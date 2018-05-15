extern crate ggez;
extern crate metrohash;
extern crate rivr;

mod cache;
mod renderer;

pub use self::{
    cache::{GgezRivrCache},
    renderer::{GgezRivrRenderer},
};

use {
    ggez::{GameError},
    rivr::{Error, RenderingError},
};

/// Converts a ggez error to a rivr error.
pub fn egtr(e: GameError) -> Error {
    Error::Rendering(RenderingError::Other(Box::new(e)))
}

/// Converts a rivr error to a ggez error.
pub fn ertg(e: Error) -> GameError {
    GameError::UnknownError(format!("{:#?}", e))
}
