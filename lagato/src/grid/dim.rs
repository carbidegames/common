use {
    nalgebra::{Vector2, Point2, Vector3, Point3},
    serde::{Serialize, de::DeserializeOwned},
};

pub trait Dim {
    type Vector: Serialize + DeserializeOwned + Copy;
    type Point: Serialize + DeserializeOwned + Copy;

    fn start() -> Self::Point;
    fn end(size: Self::Vector) -> Self::Point;

    fn area(size: Self::Vector) -> usize;
    fn is_in_bounds(position: Self::Point, size: Self::Vector) -> bool;

    fn index(position: Self::Point, size: Self::Vector) -> usize;
    fn next(position: Self::Point, start: Self::Point, end: Self::Point) -> Option<Self::Point>;
}

pub enum Dim2 {}

impl Dim for Dim2 {
    type Vector = Vector2<i32>;
    type Point = Point2<i32>;

    fn start() -> Self::Point {
        Point2::new(0, 0)
    }

    fn end(size: Self::Vector) -> Self::Point {
        Point2::new(size.x - 1, size.y - 1)
    }

    fn area(size: Vector2<i32>) -> usize {
        (size.x * size.y) as usize
    }

    fn is_in_bounds(position: Point2<i32>, size: Vector2<i32>) -> bool {
        position.x >= 0 && position.y >= 0
            && position.x < size.x && position.y < size.y
    }

    fn index(position: Point2<i32>, size: Vector2<i32>) -> usize {
        (position.x + (position.y * size.x)) as usize
    }

    fn next(
        mut position: Point2<i32>, start: Self::Point, end: Self::Point
    ) -> Option<Point2<i32>> {
        // Increment for next
        position.x += 1;
        if position.x > end.x {
            position.x = start.x;
            position.y += 1;
        }

        // See if it's beyond the end
        if position.y > end.y {
            None
        } else {
            Some(position)
        }
    }
}

pub enum Dim3 {}

impl Dim for Dim3 {
    type Vector = Vector3<i32>;
    type Point = Point3<i32>;

    fn start() -> Self::Point {
        Point3::new(0, 0, 0)
    }

    fn end(size: Self::Vector) -> Self::Point {
        Point3::new(size.x - 1, size.y - 1, size.z - 1)
    }

    fn area(size: Vector3<i32>) -> usize {
        (size.x * size.y * size.z) as usize
    }

    fn is_in_bounds(position: Point3<i32>, size: Vector3<i32>) -> bool {
        position.x >= 0 && position.y >= 0 && position.z >= 0
            && position.x < size.x && position.y < size.y && position.z < size.z
    }

    fn index(position: Point3<i32>, size: Vector3<i32>) -> usize {
        (position.x + (position.y * size.x) + (position.z * size.x * size.y)) as usize
    }

    fn next(
        mut position: Point3<i32>, start: Self::Point, end: Self::Point
    ) -> Option<Point3<i32>> {
        // Increment for next
        position.x += 1;
        if position.x > end.x {
            position.x = start.x;
            position.y += 1;

            if position.y > end.y {
                position.y = start.y;
                position.z += 1;
            }
        }

        // See if it's beyond the end
        if position.z > end.z {
            None
        } else {
            Some(position)
        }
    }
}
