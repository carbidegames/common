use {
    nalgebra::{Point2},

    grid::{Dim, Dim2, Bounds, IterBounds},
    Event,
};

#[derive(Deserialize, Serialize)]
pub struct Grid<Cell, D: Dim> {
    cells: Vec<Cell>,
    size: D::Vector,

    pub changed: Event,
}

impl<Cell: Default, D: Dim> Grid<Cell, D> {
    pub fn empty(size: D::Vector) -> Self {
        let amount = D::area(size);
        let mut cells = Vec::with_capacity(amount);
        for _ in 0..amount { cells.push(Cell::default()) }

        Grid {
            cells,
            size,

            changed: Event::new(),
        }
    }
}

impl<Cell, D: Dim> Grid<Cell, D> {
    pub fn size(&self) -> D::Vector {
        self.size
    }

    pub fn get(&self, position: D::Point) -> Result<&Cell, Error> {
        if D::is_in_bounds(position, self.size) {
            Ok(&self.cells[D::index(position, self.size)])
        } else {
            Err(Error::OutOfBounds)
        }
    }

    pub fn get_mut(&mut self, position: D::Point) -> Result<&mut Cell, Error> {
        if D::is_in_bounds(position, self.size) {
            let index = D::index(position, self.size);
            Ok(&mut self.cells[index])
        } else {
            Err(Error::OutOfBounds)
        }
    }

    pub fn is_in_bounds(&self, position: D::Point) -> bool {
        D::is_in_bounds(position, self.size)
    }

    pub fn iter_pos(&self) -> IterBounds<D> {
        Bounds {
            start: D::start(),
            end: D::end(self.size),
        }.iter()
    }
}

impl<Cell> Grid<Cell, Dim2> {
    pub fn bounds(&self, start: Point2<f32>, end: Point2<f32>) -> Bounds<Dim2> {
        let start_x = (start.x.floor() as i32).max(0);
        let start_y = (start.y.floor() as i32).max(0);
        let end_x = (end.x.ceil() as i32).min(self.size.x);
        let end_y = (end.y.ceil() as i32).min(self.size.y);

        Bounds {
            start: Point2::new(start_x, start_y),
            end: Point2::new(end_x, end_y),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    OutOfBounds,
}

#[cfg(test)]
mod tests {
    use {
        nalgebra::{Vector2, Vector3},

        grid::{Grid, Dim2, Dim3},
    };

    #[test]
    fn iter_goes_over_all_entries_dim2() {
        let grid: Grid<bool, Dim2> = Grid::empty(Vector2::new(10, 10));
        let mut got_min = false;
        let mut got_max = false;
        let mut amount_iterated = 0;

        for pos in grid.iter_pos() {
            if pos.x == 0 && pos.y == 0 {
                got_min = true;
            }
            if pos.x == 9 && pos.y == 9 {
                got_max = true;
            }

            amount_iterated += 1;
        }

        assert!(got_min);
        assert!(got_max);
        assert_eq!(amount_iterated, 10 * 10);
    }

    #[test]
    fn iter_goes_over_all_entries_dim3() {
        let grid: Grid<bool, Dim3> = Grid::empty(Vector3::new(10, 10, 10));
        let mut got_min = false;
        let mut got_max = false;
        let mut amount_iterated = 0;

        for pos in grid.iter_pos() {
            if pos.x == 0 && pos.y == 0 && pos.z == 0 {
                got_min = true;
            }
            if pos.x == 9 && pos.y == 9 && pos.z == 9 {
                got_max = true;
            }

            amount_iterated += 1;
        }

        assert!(got_min);
        assert!(got_max);
        assert_eq!(amount_iterated, 10 * 10 * 10);
    }
}
