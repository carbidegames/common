use {
    std::{
        collections::{VecDeque},
        net::{SocketAddr},
        sync::mpsc::{Sender, Receiver},
    },

    mio::{
        net::{UdpSocket},
        Events, Ready, Poll, PollOpt, Token, Registration, SetReadiness
    },

    header::{Header},
    MTU_ESTIMATE,
};

pub type WorkerMessage = (SocketAddr, Vec<u8>);

pub fn worker(
    bind_address: Option<SocketAddr>,
    worker_outgoing: Receiver<WorkerMessage>, worker_incoming: Sender<WorkerMessage>,
    registration: Registration, worker_set: SetReadiness,
) {
    const SOCKET: Token = Token(0);
    const CHANNEL: Token = Token(1);

    let socket = UdpSocket::bind(
        &bind_address.unwrap_or_else(|| "0.0.0.0:0".parse().unwrap())
    ).unwrap();

    // Set up what socket events we're looking for
    let poll = Poll::new().unwrap();
    poll.register(&socket, SOCKET, Ready::readable(), PollOpt::edge()).unwrap();
    poll.register(&registration, CHANNEL, Ready::readable(), PollOpt::edge()).unwrap();

    // Loop to handle events when they come up
    // IMPORTANT: It's best to do as little work as possible on this thread, since we have to work
    // with timed IO resources access.
    let mut events = Events::with_capacity(128);
    let mut waiting_sends = VecDeque::new();
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                SOCKET => {
                    if event.readiness().is_writable() {
                        write(&socket, &mut waiting_sends);

                        // If we don't have anything left we don't need to wait for writes anymore
                        if waiting_sends.len() == 0 {
                            poll.reregister(
                                &socket, SOCKET, Ready::readable(), PollOpt::edge()
                            ).unwrap();
                        }
                    }

                    if event.readiness().is_readable() {
                        read(&socket, &worker_incoming);
                    }
                },
                CHANNEL => {
                    worker_set.set_readiness(Ready::empty()).unwrap();

                    while let Some(message) = worker_outgoing.try_recv().ok() {
                        waiting_sends.push_back(message);
                    }

                    // Make sure we're listening to write events now so we can send out the data
                    let both = Ready::readable() | Ready::writable();
                    poll.reregister(&socket, SOCKET, both, PollOpt::edge()).unwrap();
                },
                _ => unreachable!()
            }
        }
    }
}

fn write(socket: &UdpSocket, waiting_sends: &mut VecDeque<WorkerMessage>) {
    while let Some((target, data)) = waiting_sends.pop_front() {
        // This is verified by the send function, but just in case something went wrong
        assert!(data.len() <= MTU_ESTIMATE);

        let result = socket.send_to(&data, &target);

        // If we hit this it probably means we hit a WouldBlock, so just wait till we can write
        // again. Remember to put it back in the front of the list.
        if result.is_err() {
            waiting_sends.push_front((target, data));
            return
        }
    }
}

fn read(socket: &UdpSocket, worker_incoming: &Sender<WorkerMessage>) {
    let mut buffer = vec![0; MTU_ESTIMATE];

    while let Ok((length, from)) = socket.recv_from(&mut buffer) {
        // If the packet is too small to have our header, don't even bother sending it
        // Doing this here prevents us from clogging the channel with empty packets in case of a
        // DoS attack
        if length < Header::DATA_SIZE { return }

        // Resize the vector to hide waste data, then send it over
        buffer.resize(length, 0);
        worker_incoming.send((from, buffer.to_vec())).unwrap()
    }
}
