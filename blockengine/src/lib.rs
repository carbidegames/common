extern crate alga;
extern crate nalgebra;
extern crate lagato;

use {
    nalgebra::{Point2},
    lagato::grid::{Voxels},
};

pub struct Chunk {
    pub position: Point2<i32>,
    pub voxels: Voxels<bool>,
}
