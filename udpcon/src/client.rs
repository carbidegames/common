use {
    std::net::{SocketAddr},

    mio::{
        net::{UdpSocket},
        Events, Ready, Poll, PollOpt, Token,
    },
};

pub struct Client {
}

impl Client {
    pub fn connect(server: SocketAddr) -> Self {
        const WRITE: Token = Token(1);

        let write_socket = UdpSocket::bind(&"0.0.0.0:0".parse().unwrap()).unwrap();
        write_socket.connect(server).unwrap();

        let poll = Poll::new().unwrap();
        poll.register(&write_socket, WRITE, Ready::writable(), PollOpt::edge()).unwrap();

        let mut events = Events::with_capacity(128);
        'breakme: loop {
            poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match event.token() {
                    WRITE => {
                        write_socket.send(&[0, 1, 2, 3]).unwrap();
                        break 'breakme
                    }
                    _ => unreachable!()
                }
            }
        }

        Client {
        }
    }
}
