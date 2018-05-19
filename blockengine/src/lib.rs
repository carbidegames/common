extern crate alga;
extern crate ggez;
#[macro_use] extern crate gfx;
extern crate gfx_device_gl;
extern crate nalgebra;
extern crate lagato;

pub mod rendering;

use {
    nalgebra::{Point2},
    lagato::grid::{Voxels},
    rendering::{VoxelsMesh}
};

pub struct Chunk {
    pub position: Point2<i32>,
    pub voxels: Voxels<bool>,
    pub mesh: VoxelsMesh,
}
