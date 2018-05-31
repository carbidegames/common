use {
    cgmath::{Point2, Point3},
    grid::{Dim, Dim2, Dim3},
};

#[derive(Clone, Copy)]
pub struct Range<D: Dim> {
    pub start: D::Point,
    /// Inclusive
    pub end: D::Point,
}

impl<D: Dim> Range<D> {
    pub fn new(start: D::Point, end: D::Point) -> Self {
        Range { start, end }
    }

    pub fn iter(self) -> IterRange<D> {
        let start = self.start;
        IterRange {
            range: self,
            next_position: Some(start),
        }
    }
}

impl Range<Dim2> {
    pub fn new_dim2(
        start_x: i32, start_y: i32, end_x: i32, end_y: i32
    ) -> Self {
        Range::new(Point2::new(start_x, start_y), Point2::new(end_x, end_y))
    }
}

impl Range<Dim3> {
    pub fn new_dim3(
        start_x: i32, start_y: i32, start_z: i32, end_x: i32, end_y: i32, end_z: i32
    ) -> Self {
        Range::new(Point3::new(start_x, start_y, start_z), Point3::new(end_x, end_y, end_z))
    }
}

pub struct IterRange<D: Dim> {
    range: Range<D>,
    next_position: Option<D::Point>,
}

impl<D: Dim> Iterator for IterRange<D> {
    type Item = D::Point;

    fn next(&mut self) -> Option<D::Point> {
        let position = self.next_position;

        if let Some(position) = position {
            self.next_position = D::next(position, self.range.start, self.range.end);
        }

        position
    }
}
