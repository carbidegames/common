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

type Data = [u8; 4];

pub struct Client {
    _worker_thread: JoinHandle<()>,
    sender: Sender<Data>,
}

impl Client {
    pub fn connect(server: SocketAddr) -> Self {
        let (sender, receiver) = mpsc::channel();
        let worker_thread = thread::spawn(move || client_worker(server, receiver));

        Client {
            _worker_thread: worker_thread,
            sender,
        }
    }

    pub fn send(&self, data: Data) {
        self.sender.send(data).unwrap();
    }
}

fn client_worker(server: SocketAddr, receiver: Receiver<Data>) {
    const WRITE: Token = Token(1);

    let write_socket = UdpSocket::bind(&"0.0.0.0:0".parse().unwrap()).unwrap();
    write_socket.connect(server).unwrap();

    let poll = Poll::new().unwrap();
    poll.register(&write_socket, WRITE, Ready::writable(), PollOpt::edge()).unwrap();

    let mut events = Events::with_capacity(128);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                WRITE => {
                    while let Some(data) = receiver.try_recv().ok() {
                        write_socket.send(&data).unwrap();
                    }
                }
                _ => unreachable!()
            }
        }
    }
}
