use {
    grid::{Dim},
};

#[derive(Clone, Copy)]
pub struct Bounds<D: Dim> {
    pub start: D::Point,
    pub end: D::Point,
}

impl<D: Dim> Bounds<D> {
    pub fn iter(self) -> IterBounds<D> {
        let start = self.start;
        IterBounds {
            bounds: self,
            next_position: Some(start),
        }
    }
}

pub struct IterBounds<D: Dim> {
    bounds: Bounds<D>,
    next_position: Option<D::Point>,
}

impl<D: Dim> Iterator for IterBounds<D> {
    type Item = D::Point;

    fn next(&mut self) -> Option<D::Point> {
        let position = self.next_position;

        if let Some(position) = position {
            self.next_position = D::next(position, self.bounds.start, self.bounds.end);
        }

        position
    }
}
