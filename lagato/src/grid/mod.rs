mod dim;
mod grid;
mod range;

pub use self::{
    dim::{Dim, Dim2, Dim3},
    grid::{Grid, Error},
    range::{Range, IterRange},
};

pub type Tiles<Tile> = Grid<Tile, Dim2>;
pub type Voxels<Voxel> = Grid<Voxel, Dim3>;
