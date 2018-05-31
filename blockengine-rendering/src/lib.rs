extern crate cgmath;
extern crate ggez;
#[macro_use] extern crate gfx;
extern crate gfx_device_gl;
extern crate image;
extern crate lagato;
extern crate blockengine;

mod mesh;
mod renderer;
mod texture;

pub use self::{
    renderer::{Renderer},
    mesh::{Mesh, triangulate_voxels},
    texture::{Texture},
};

use {
    cgmath::{Point3},
};

pub struct Object {
    pub position: Point3<f32>,
    pub mesh: Mesh,
}
