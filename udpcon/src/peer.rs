use {
    std::{
        thread::{self, JoinHandle},
        net::{SocketAddr},
        sync::mpsc::{self, Sender, Receiver},
    },

    crc::{crc32},

    worker::{self, WorkerMessage},
    Error,
};

const HEADER_SIZE: usize = 4;

pub struct Peer {
    _worker_thread: JoinHandle<()>,
    incoming: Receiver<WorkerMessage>,
    outgoing: Sender<WorkerMessage>,
}

impl Peer {
    pub fn start(mode: PeerMode, protocol: &'static str) -> Self {
        // Get our protocol identifier from the caller-friendly string
        let protocol_id = crc32::checksum_ieee(protocol.as_bytes());

        // Set up the worker that manages the sockets
        let (worker_incoming, incoming) = mpsc::channel();
        let (outgoing, worker_outgoing) = mpsc::channel();
        let worker_thread = thread::spawn(move || {
            worker::worker(mode, protocol_id, worker_outgoing, worker_incoming);
        });

        Peer {
            _worker_thread: worker_thread,
            incoming,
            outgoing,
        }
    }

    pub fn send(&self, target: SocketAddr, data: Vec<u8>) -> Result<(), Error> {
        // It's recommended to limit UDP packets to 512 bytes, and the read side only allocates
        // that much data in the buffer for receiving. Therefore, prevent any packets that are too
        // large.
        if data.len() > 512 - HEADER_SIZE {
            return Err(Error::DataTooLarge)
        }

        self.outgoing.send((target, data)).unwrap();
        Ok(())
    }

    pub fn try_recv(&self) -> Option<(SocketAddr, Vec<u8>)> {
        self.incoming.try_recv().ok()
    }
}

pub enum PeerMode {
    Server { address: SocketAddr },
    Client { server: SocketAddr },
}
