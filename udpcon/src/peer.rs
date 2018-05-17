use {
    std::{
        thread::{self, JoinHandle},
        net::{SocketAddr},
        sync::mpsc::{self, Sender, Receiver},
    },

    crc::{crc32},
    byteorder::{WriteBytesExt, LittleEndian, ByteOrder},

    worker::{self, WorkerMessage},
    Error,
};

pub struct Peer {
    protocol_id: u32,
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
            worker::worker(mode, worker_outgoing, worker_incoming);
        });

        Peer {
            protocol_id,
            _worker_thread: worker_thread,
            incoming,
            outgoing,
        }
    }

    pub fn send(&self, target: SocketAddr, mut data: Vec<u8>) -> Result<(), Error> {
        // Append the protocol ID so the receiver can verify its validness.
        // It's appended at the end because we will know the length anyways, so our header doesn't
        // have to be at the start. This way we can avoid having to copy data to put the header at
        // the start.
        data.write_u32::<LittleEndian>(self.protocol_id).unwrap();

        // It's recommended to limit UDP packets to 512 bytes, and the read side only allocates
        // that much data in the buffer for receiving. Therefore, prevent any packets that are too
        // large.
        if data.len() > 512 {
            return Err(Error::DataTooLarge)
        }

        self.outgoing.send((target, data)).unwrap();
        Ok(())
    }

    pub fn try_recv(&self) -> Option<(SocketAddr, Vec<u8>)> {
        // Keep trying until we've got valid data or we run out
        while let Some((address, mut data)) = self.incoming.try_recv().ok() {
            let header_start = data.len()-4;

            // Verify the protocol ID, if it's not right, skip this packet
            let client_protocol_id = LittleEndian::read_u32(&data[header_start..]);
            if client_protocol_id != self.protocol_id { continue }

            // Hide the header
            data.resize(header_start, 0);

            return Some((address, data))
        }

        None
    }
}

pub enum PeerMode {
    Server { address: SocketAddr },
    Client { server: SocketAddr },
}
