extern crate mio;
extern crate crc;
extern crate byteorder;

mod peer;
mod worker;

pub use {
    peer::{Peer, EventsIter, Event},
};

#[derive(Debug)]
pub enum Error {
    DataTooLarge,
}

// This number for Maximum Transmission Unit is frequently used in the games industry as a good
// rule of thumb for what's likely to be safe in most real-world situations
const MTU_ESTIMATE: usize = 1024;
