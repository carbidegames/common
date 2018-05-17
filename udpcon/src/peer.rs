use {
    std::{
        collections::{VecDeque, HashSet},
        thread::{self, JoinHandle},
        net::{SocketAddr},
        sync::mpsc::{self, Sender, Receiver},
    },

    crc::{crc32},
    byteorder::{WriteBytesExt, LittleEndian, ByteOrder},

    worker::{self, WorkerMessage},
    Error,
    MTU_ESTIMATE,
};

pub struct Peer {
    protocol_id: u32,

    _worker_thread: JoinHandle<()>,
    incoming: Receiver<WorkerMessage>,
    outgoing: Sender<WorkerMessage>,

    queued_events: VecDeque<Event>,
    peers: HashSet<SocketAddr>,
}

impl Peer {
    pub fn start(address: Option<SocketAddr>, protocol: &'static str) -> Self {
        // Get our protocol identifier from the caller-friendly string
        let protocol_id = crc32::checksum_ieee(protocol.as_bytes());

        // Set up the worker that manages the sockets
        let (worker_incoming, incoming) = mpsc::channel();
        let (outgoing, worker_outgoing) = mpsc::channel();
        let worker_thread = thread::spawn(move || {
            worker::worker(address, worker_outgoing, worker_incoming);
        });

        Peer {
            protocol_id,

            _worker_thread: worker_thread,
            incoming,
            outgoing,

            queued_events: VecDeque::new(),
            peers: HashSet::new(),
        }
    }

    pub fn send(&self, target: SocketAddr, mut data: Vec<u8>) -> Result<(), Error> {
        // Append the protocol ID so the receiver can verify its validness.
        // It's appended at the end because we will know the length anyways, so our header doesn't
        // have to be at the start. This way we can avoid having to copy data to put the header at
        // the start.
        data.write_u32::<LittleEndian>(self.protocol_id).unwrap();

        // Limit packet sizes to at most the MTU, anything more might get dropped
        // TODO: Support automatically splitting large packets
        if data.len() > MTU_ESTIMATE {
            return Err(Error::DataTooLarge)
        }

        self.outgoing.send((target, data)).unwrap();
        Ok(())
    }

    /// Checks for incoming packets and network events.
    /// This is combined into one event queue to make sure network events can be handled
    /// sequentially, for example you can be sure a Packet event won't be received before a
    /// NewPeer event.
    pub fn poll(&mut self) -> EventsIter {
        while let Some((source, mut data)) = self.incoming.try_recv().ok() {
            let header_start = data.len()-4;

            // Verify the protocol ID, if it's not right, skip this packet
            let client_protocol_id = LittleEndian::read_u32(&data[header_start..]);
            if client_protocol_id != self.protocol_id { continue }

            // Hide the header
            data.resize(header_start, 0);

            // Check if we haven't seen this peer before, if that's the case we have to raise a new
            // peer event before the packet event
            if !self.peers.contains(&source) {
                self.peers.insert(source);
                self.queued_events.push_back(Event::NewPeer { address: source })
            }

            self.queued_events.push_back(Event::Packet { source, data });
        }

        EventsIter { queued_events: &mut self.queued_events }
    }
}

pub struct EventsIter<'a> {
    queued_events: &'a mut VecDeque<Event>,
}

impl<'a> Iterator for EventsIter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        self.queued_events.pop_front()
    }
}

#[derive(Debug)]
pub enum Event {
    Packet { source: SocketAddr, data: Vec<u8>, },
    NewPeer { address: SocketAddr, },
}
