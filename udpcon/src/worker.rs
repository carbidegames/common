use {
    std::{
        net::{SocketAddr},
        sync::mpsc::{Sender, Receiver},
    },

    mio::{
        net::{UdpSocket},
        Events, Ready, Poll, PollOpt, Token,
    },
    byteorder::{WriteBytesExt, LittleEndian, ByteOrder},

    PeerMode,
};

pub type WorkerMessage = (SocketAddr, Vec<u8>);

pub fn worker(
    mode: PeerMode, protocol_id: u32,
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
    let mut events = Events::with_capacity(128);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                WRITE => write(protocol_id, &write_socket, &worker_outgoing),
                READ => read(protocol_id, &read_socket, &worker_incoming),
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

fn write(protocol_id: u32, write_socket: &UdpSocket, worker_outgoing: &Receiver<WorkerMessage>) {
    if let Some((target, mut data)) = worker_outgoing.try_recv().ok() {
        // Append the protocol ID so the receiver can verify its validness.
        // It's appended at the end because we will know the length anyways, so our header doesn't
        // have to be at the start. This way we can avoid having to copy data to put the header at
        // the start.
        data.write_u32::<LittleEndian>(protocol_id).unwrap();

        // This is verified by the send function, but just in case the header changes
        assert!(data.len() <= 512);

        write_socket.send_to(&data, &target).unwrap();
    }
}

fn read(protocol_id: u32, read_socket: &UdpSocket, worker_incoming: &Sender<WorkerMessage>) {
    let mut buffer = vec![0; 512];
    let (length, from) = read_socket.recv_from(&mut buffer).unwrap();

    // If the packet is too small to have our header, just skip it
    if length < 4 { return }

    // Verify the protocol ID, if it's not right, skip this packet
    let client_protocol_id = LittleEndian::read_u32(&buffer[length-4..length]);
    if client_protocol_id != protocol_id { return }

    // Resize the vector to hide waste data, then send it over
    buffer.resize(length-4, 0);
    worker_incoming.send((from, buffer.to_vec())).unwrap()
}
