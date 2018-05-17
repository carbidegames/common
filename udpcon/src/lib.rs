extern crate mio;
extern crate crc;
extern crate byteorder;

mod peer;
mod worker;

pub use {
    peer::{Peer, PeerMode, EventsIter, Event},
};

#[derive(Debug)]
pub enum Error {
    DataTooLarge,
}
