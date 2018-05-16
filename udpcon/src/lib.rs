extern crate mio;
extern crate crc;
extern crate byteorder;

mod peer;

pub use {
    peer::{Peer, PeerMode},
};

#[derive(Debug)]
pub enum Error {
    DataTooLarge,
}
