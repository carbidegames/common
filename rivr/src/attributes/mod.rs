mod panelbox;
mod size;

// Convenience re-exports so for basic usage you don't need the dependencies
pub use {
    nalgebra::{Point2, Vector2},
    palette::{Srgba},
};

pub use self::{
    size::{PanelSize, AxisSize},
    panelbox::{PanelBox},
};

pub enum Orientation {
    Horizontal,
    Vertical,
}
