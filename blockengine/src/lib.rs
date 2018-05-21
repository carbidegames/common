extern crate alga;
extern crate nalgebra;
extern crate lagato;

use {
    nalgebra::{Point3},
    lagato::grid::{Voxels},
};

pub struct Chunk<D> {
    pub position: Point3<i32>,
    pub voxels: Voxels<bool>,
    /// Additional game/side specific chunk data.
    pub data: D,
}
