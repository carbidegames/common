extern crate cgmath;
extern crate serde;
#[macro_use] extern crate serde_derive;

pub mod camera;
pub mod grid;
mod event;

pub use self::{
    event::{Event},
};

use {
    cgmath::{Vector2, Rad, Angle},
};

pub struct DirectionalInput {
    pub backward: bool,
    pub forward: bool,
    pub left: bool,
    pub right: bool,
}

impl DirectionalInput {
    pub fn new() -> Self {
        DirectionalInput {
            backward: false,
            forward: false,
            left: false,
            right: false,
        }
    }

    pub fn to_vector(&self) -> Vector2<f32> {
        let mut input = Vector2::new(0.0, 0.0);
        if self.backward { input.y += 1.0; }
        if self.forward { input.y -= 1.0; }
        if self.left { input.x -= 1.0; }
        if self.right { input.x += 1.0; }

        input
    }
}

pub fn rotate_vector(mut value: Vector2<f32>, radians: Rad<f32>) -> Vector2<f32> {
    let sin = radians.sin();
    let cos = radians.cos();

    let tx = value.x;
    let ty = value.y;

    value.x = (cos * tx) - (sin * ty);
    value.y = (sin * tx) + (cos * ty);

    value
}
