use {
    std::{
        collections::{VecDeque, HashMap},
        thread::{self, JoinHandle},
        net::{SocketAddr},
        sync::mpsc::{self, Sender, Receiver},
        time::{Instant, Duration},
    },

    mio::{Registration, SetReadiness, Ready},
    crc::{crc32},

    header::{Header, PacketClass},
    worker::{self, WorkerMessage},
    Error, MTU_ESTIMATE,
};

pub struct Peer {
    protocol_id: u32,

    _worker_thread: JoinHandle<()>,
    incoming: Receiver<WorkerMessage>,
    outgoing: Sender<WorkerMessage>,
    outgoing_set: SetReadiness,

    queued_events: VecDeque<Event>,
    connections: HashMap<SocketAddr, PeerConnection>,
}

impl Peer {
    /// Starts a new open UDP peer. `bind_address` is the address and port this peer will listen on
    /// for incoming connections if applicable.
    pub fn start(bind_address: Option<SocketAddr>, protocol: &'static str) -> Self {
        // Get our protocol identifier from the caller-friendly string
        let protocol_id = crc32::checksum_ieee(protocol.as_bytes());

        // Set up the worker that manages the sockets
        let (worker_incoming, incoming) = mpsc::channel();
        let (outgoing, worker_outgoing) = mpsc::channel();
        let (registration, outgoing_set) = Registration::new2();
        let worker_set = outgoing_set.clone();
        let worker_thread = thread::spawn(move || {
            worker::worker(
                bind_address, worker_outgoing, worker_incoming, registration, worker_set
            );
        });

        Peer {
            protocol_id,

            _worker_thread: worker_thread,
            incoming,
            outgoing,
            outgoing_set,

            queued_events: VecDeque::new(),
            connections: HashMap::new(),
        }
    }

    /// Sends an outgoing message to a target.
    pub fn send(&mut self, target: SocketAddr, mut data: Vec<u8>) -> Result<(), Error> {
        let header = Header {
            class: PacketClass::Message,
        };
        header.write_to(&mut data, self.protocol_id);

        self.send_packet(target, data)
    }

    /// Checks for incoming packets and network events.
    /// This is combined into one event queue to make sure network events can be handled
    /// sequentially, for example you can be sure a Message event won't be received before a
    /// NewPeer event.
    pub fn poll(&mut self) -> EventsIter {
        let now = Instant::now();

        while let Some((source, data)) = self.incoming.try_recv().ok() {
            // The header extraction makes sure we're not being sent garbage
            if let Some((header, data)) = Header::extract(data, self.protocol_id) {
                // Update when the last time we got a packet was
                self.update_last_packet(source, now);

                if header.class == PacketClass::Message {
                    self.queued_events.push_back(Event::Message { source, data });
                }
            }
        }

        // Check if any connections have timed out
        self.check_timeouts(now);

        // Check if we have to send heartbeats to any connection
        self.send_heartbeats(now);

        EventsIter { queued_events: &mut self.queued_events }
    }

    fn update_last_packet(&mut self, source: SocketAddr, now: Instant) {
        if self.connections.contains_key(&source) {
            // We currently have a connection with this peer, update the last time we saw it
            self.connections.get_mut(&source).unwrap().last_received = now;
        } else {
            // We haven't seen this peer yet, so add it now and raise an event for it
            self.connections.insert(source, PeerConnection {
                last_received: now,
                last_sent: now - Duration::new(10, 0),
            });
            self.queued_events.push_back(Event::NewPeer { address: source })
        }
    }

    fn check_timeouts(&mut self, now: Instant) {
        let timeout = Duration::new(5, 0);
        let queued_events = &mut self.queued_events;
        self.connections.retain(|address, peer| {
            let timed_out = now.duration_since(peer.last_received) >= timeout;
            if timed_out {
                queued_events.push_back(Event::PeerTimedOut { address: *address });
            }
            !timed_out
        });
    }

    fn send_heartbeats(&mut self, now: Instant) {
        let treshold = Duration::new(1, 0);

        let mut needs_heartbeat = Vec::new();
        for (address, connection) in &self.connections {
            if now.duration_since(connection.last_sent) > treshold {
                needs_heartbeat.push(*address);
            }
        }

        for address in needs_heartbeat {
            let mut data = Vec::new();
            let header = Header {
                class: PacketClass::Heartbeat,
            };
            header.write_to(&mut data, self.protocol_id);

            self.send_packet(address, data).unwrap();
        }
    }

    fn send_packet(&mut self, target: SocketAddr, data: Vec<u8>) -> Result<(), Error> {
        // Limit packet sizes to at most the MTU, anything more might get dropped
        // TODO: Support automatically splitting large packets
        if data.len() > MTU_ESTIMATE {
            return Err(Error::DataTooLarge)
        }

        // Keep track of that we sent a packet at this time, we use this to check if we need to
        // send empty heartbeats to keep the connection alive
        if let Some(connection) = self.connections.get_mut(&target) {
            connection.last_sent = Instant::now();
        }

        self.outgoing.send((target, data)).unwrap();
        self.outgoing_set.set_readiness(Ready::readable()).unwrap();
        Ok(())
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
    Message { source: SocketAddr, data: Vec<u8> },
    NewPeer { address: SocketAddr },
    PeerTimedOut { address: SocketAddr },
}

struct PeerConnection {
    last_received: Instant,
    last_sent: Instant,
}
