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

type Data = ([u8; 4], SocketAddr);

pub struct Server {
    worker_thread: JoinHandle<()>,
    receiver: Receiver<Data>,
}

impl Server {
    pub fn start(bind: SocketAddr) -> Self {
        let (sender, receiver) = mpsc::channel();

        let worker_thread = thread::spawn(move || server_worker(bind, sender));

        Server {
            worker_thread,
            receiver,
        }
    }

    pub fn try_recv(&self) -> Option<Data> {
        self.receiver.try_recv().ok()
    }
}

fn server_worker(bind: SocketAddr, sender: Sender<Data>) {
    const READ: Token = Token(1);

    let read_socket = UdpSocket::bind(&bind).unwrap();

    let poll = Poll::new().unwrap();
    poll.register(&read_socket, READ, Ready::readable(), PollOpt::edge()).unwrap();

    let mut buffer = [0; 4];

    let mut events = Events::with_capacity(128);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                READ => {
                    let from = read_socket.recv_from(&mut buffer).unwrap().1;
                    sender.send((buffer, from)).unwrap()
                }
                _ => unreachable!()
            }
        }
    }
}
