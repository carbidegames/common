use {
    std::{
        thread::{self, JoinHandle},
        net::{SocketAddr},
        sync::mpsc::{self, Sender, Receiver},
    },

    mio::{
        net::{UdpSocket},
        Events, Ready, Poll, PollOpt, Token,
    },
};

type WorkerMessage = (SocketAddr, [u8; 4]);

pub struct Peer {
    _worker_thread: JoinHandle<()>,
    incoming: Receiver<WorkerMessage>,
    outgoing: Sender<WorkerMessage>,
}

impl Peer {
    pub fn start(mode: PeerMode) -> Self {
        let (worker_incoming, incoming) = mpsc::channel();
        let (outgoing, worker_outgoing) = mpsc::channel();
        let worker_thread = thread::spawn(move || {
            worker(mode, worker_outgoing, worker_incoming);
        });

        Peer {
            _worker_thread: worker_thread,
            incoming,
            outgoing,
        }
    }

    pub fn send(&self, target: SocketAddr, data: [u8; 4]) {
        self.outgoing.send((target, data)).unwrap();
    }

    pub fn try_recv(&self) -> Option<(SocketAddr, [u8; 4])> {
        self.incoming.try_recv().ok()
    }
}

pub enum PeerMode {
    Server { address: SocketAddr },
    Client { server: SocketAddr },
}

fn worker(
    mode: PeerMode,
    worker_outgoing: Receiver<WorkerMessage>, worker_incoming: Sender<WorkerMessage>
) {
    const WRITE: Token = Token(0);
    const READ: Token = Token(1);

    // Set up the connection, depending on if we're a client or server
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
                WRITE => {
                    while let Some(data) = worker_outgoing.try_recv().ok() {
                        write_socket.send_to(&data.1, &data.0).unwrap();
                    }
                },
                READ => {
                    let mut buffer = [0; 4];
                    let from = read_socket.recv_from(&mut buffer).unwrap().1;
                    worker_incoming.send((from, buffer)).unwrap()
                }
                _ => unreachable!()
            }
        }
    }
}
