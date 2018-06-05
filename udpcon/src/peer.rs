use {
    std::{
        collections::{VecDeque, HashMap},
        net::{SocketAddr},
        time::{Instant, Duration},
    },

    crc::{crc32},

    header::{Header, PacketClass, SequencedHeader},
    worker::{PacketWorker},
    Error, MTU_ESTIMATE,
};

pub struct Peer {
    protocol_id: u32,
    worker: PacketWorker,

    queued_events: VecDeque<Event>,
    connections: HashMap<SocketAddr, PeerConnection>,
    next_packet_number: u16,
}

impl Peer {
    /// Starts a new open UDP peer. `bind_address` is the address and port this peer will listen on
    /// for incoming connections if applicable.
    pub fn start(bind_address: Option<SocketAddr>, protocol: &'static str) -> Self {
        // Get our protocol identifier from the caller-friendly string
        let protocol_id = crc32::checksum_ieee(protocol.as_bytes());

        let worker = PacketWorker::start(bind_address);

        Peer {
            protocol_id,
            worker,

            queued_events: VecDeque::new(),
            connections: HashMap::new(),
            next_packet_number: 1,
        }
    }

    pub fn stop(self) {
        self.worker.stop()
    }

    pub fn connect(&mut self, target: SocketAddr) {
        self.send_heartbeat(target);
    }

    /// Sends an outgoing message to a target.
    pub fn send(
        &mut self, target: SocketAddr, mut data: Vec<u8>, reliability: Reliability,
    ) -> Result<(), Error> {
        // Headers are attached after data and eachother in sequence, the header at the end is used
        // to interpret what headers should be read in before it.

        let class = match reliability {
            Reliability::Unreliable => PacketClass::UnreliableMessage,
            Reliability::Sequenced => {
                let sequenced_header = SequencedHeader { packet_number: self.next_packet_number };
                self.next_packet_number = self.next_packet_number.wrapping_add(1);
                sequenced_header.write_to(&mut data);

                PacketClass::SequencedMessage
            },
        };

        let header = Header { class };
        header.write_to(&mut data, self.protocol_id);

        self.send_packet(target, data)
    }

    /// Checks for incoming packets and network events, and sends out heartbeat messages.
    /// Network events are combined into one event queue to make sure they can be handled
    /// sequentially, for example you can be sure a Message event won't be received before a
    /// NewPeer event.
    pub fn update(&mut self) -> EventsIter {
        let now = Instant::now();

        while let Some((source, data)) = self.worker.try_recv() {
            // The header extraction makes sure we're not being sent garbage
            if let Some((header, data)) = Header::extract(data, self.protocol_id) {
                // Update when the last time we got a packet was
                self.update_last_packet(source, now);

                match header.class {
                    PacketClass::UnreliableMessage =>
                        self.queued_events.push_back(Event::Message { source, data }),
                    PacketClass::SequencedMessage => {
                        // Make sure we have enough remaining data for this header
                        if data.len() < SequencedHeader::START_OFFSET {
                            continue
                        }

                        // This means we also need to extract the sequenced header
                        let (sequenced_header, data) = SequencedHeader::extract(data);

                        // Check if we should drop this packet
                        let connection = self.connections.get_mut(&source).unwrap();
                        if !sequence_greater_than(
                            sequenced_header.packet_number,
                            connection.last_received_packet_number,
                        ) {
                            continue
                        }
                        connection.last_received_packet_number = sequenced_header.packet_number;

                        self.queued_events.push_back(Event::Message { source, data });
                    },
                    _ => {},
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
                last_received_packet_number: 0,
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
            self.send_heartbeat(address);
        }
    }

    fn send_heartbeat(&mut self, target: SocketAddr) {
        let mut data = Vec::new();
        let header = Header {
            class: PacketClass::Heartbeat,
        };
        header.write_to(&mut data, self.protocol_id);

        self.send_packet(target, data).unwrap();
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

        self.worker.send(target, data);
        Ok(())
    }
}

pub enum Reliability {
    /// This message:
    /// - May not arrive
    /// - May not arrive in order
    Unreliable,
    /// This message:
    /// - May not arrive
    /// - Is dropped if arriving later than other messages
    Sequenced,
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
    NewPeer { address: SocketAddr },
    PeerTimedOut { address: SocketAddr },
    Message { source: SocketAddr, data: Vec<u8> },
}

struct PeerConnection {
    last_received: Instant,
    last_sent: Instant,
    last_received_packet_number: u16,
}

fn sequence_greater_than(previous: u16, next: u16) -> bool {
    ( ( previous > next ) && ( previous - next <= 32768 ) ) ||
    ( ( previous < next ) && ( next - previous  > 32768 ) )
}
