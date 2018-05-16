extern crate mio;

mod client;
mod server;

pub use {
    client::{Client},
    server::{Server},
};

// Remember, recommended UDP packet size is: 512
