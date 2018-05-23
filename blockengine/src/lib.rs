extern crate alga;
extern crate nalgebra;
extern crate lagato;

use {
    nalgebra::{Point3},
    lagato::grid::{Voxels},
};

pub struct Chunk {
    pub position: Point3<i32>,
    pub voxels: Voxels<bool>,
}
