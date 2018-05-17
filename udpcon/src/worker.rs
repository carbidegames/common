use {
    std::{
        net::{SocketAddr},
        sync::mpsc::{Sender, Receiver},
    },

    mio::{
        net::{UdpSocket},
        Events, Ready, Poll, PollOpt, Token,
    },
};

pub type WorkerMessage = (SocketAddr, Vec<u8>);

pub fn worker(
    address: Option<SocketAddr>,
    worker_outgoing: Receiver<WorkerMessage>, worker_incoming: Sender<WorkerMessage>
) {
    const WRITE: Token = Token(0);
    const READ: Token = Token(1);

    let (write_socket, read_socket) = initialize_sockets(address);

    // Set up what socket events we're looking for
    let poll = Poll::new().unwrap();
    poll.register(&write_socket, WRITE, Ready::writable(), PollOpt::edge()).unwrap();
    poll.register(&read_socket, READ, Ready::readable(), PollOpt::edge()).unwrap();

    // Loop to handle events when they come up
    // IMPORTANT: It's best to do as little work as possible on this thread, since we have to work
    // with timed IO resources access.
    let mut events = Events::with_capacity(128);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                WRITE => write(&write_socket, &worker_outgoing),
                READ => read(&read_socket, &worker_incoming),
                _ => unreachable!()
            }
        }
    }
}

fn initialize_sockets(address: Option<SocketAddr>) -> (UdpSocket, UdpSocket) {
    let write_socket = UdpSocket::bind(&"0.0.0.0:0".parse().unwrap()).unwrap();
    let read_socket = UdpSocket::bind(
        &address.unwrap_or_else(|| "0.0.0.0:0".parse().unwrap())
    ).unwrap();

    (write_socket, read_socket)
}

fn write(write_socket: &UdpSocket, worker_outgoing: &Receiver<WorkerMessage>) {
    if let Some((target, data)) = worker_outgoing.try_recv().ok() {
        // This is verified by the send function, but just in case something went wrong
        assert!(data.len() <= 512);

        write_socket.send_to(&data, &target).unwrap();
    }
}

fn read(read_socket: &UdpSocket, worker_incoming: &Sender<WorkerMessage>) {
    let mut buffer = vec![0; 512];
    let (length, from) = read_socket.recv_from(&mut buffer).unwrap();

    // If the packet is too small to have our header, don't even bother sending it
    // Doing this here prevents us from clogging the channel with empty packets in case of a DoS
    // attack
    if length < 4 { return }

    // Resize the vector to hide waste data, then send it over
    buffer.resize(length, 0);
    worker_incoming.send((from, buffer.to_vec())).unwrap()
}
