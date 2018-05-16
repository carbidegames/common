extern crate mio;
extern crate crc;
extern crate byteorder;

mod peer;

pub use {
    peer::{Peer, PeerMode},
};

// Remember, recommended UDP packet size is: 512 bytes
