mod bounds;
mod dim;
mod grid;

pub use self::{
    bounds::{Bounds, IterBounds},
    dim::{Dim, Dim2, Dim3},
    grid::{Grid, Error},
};

pub type Tiles<Tile> = Grid<Tile, Dim2>;
pub type Voxels<Voxel> = Grid<Voxel, Dim3>;
