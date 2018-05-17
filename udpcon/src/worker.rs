use {
    std::{
        net::{SocketAddr},
        sync::mpsc::{Sender, Receiver},
    },

    mio::{
        net::{UdpSocket},
        Events, Ready, Poll, PollOpt, Token,
    },

    PeerMode,
};

pub type WorkerMessage = (SocketAddr, Vec<u8>);

pub fn worker(
    mode: PeerMode,
    worker_outgoing: Receiver<WorkerMessage>, worker_incoming: Sender<WorkerMessage>
) {
    const WRITE: Token = Token(0);
    const READ: Token = Token(1);

    let (write_socket, read_socket) = initialize_sockets(mode);

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

/// Set up the sockets, depending on if we're a client or server
fn initialize_sockets(mode: PeerMode) -> (UdpSocket, UdpSocket) {
    let write_socket = UdpSocket::bind(&"0.0.0.0:0".parse().unwrap()).unwrap();
    let read_socket = match mode {
        PeerMode::Server { address } => {
            UdpSocket::bind(&address).unwrap()
        }
        PeerMode::Client { server } => {
            let socket = UdpSocket::bind(&"0.0.0.0:0".parse().unwrap()).unwrap();
            socket.connect(server).unwrap();
            socket
        },
    };

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

    // If the packet is too small to have our header, don't even bother processing it
    if length < 4 { return }

    // Resize the vector to hide waste data, then send it over
    buffer.resize(length, 0);
    worker_incoming.send((from, buffer.to_vec())).unwrap()
}
