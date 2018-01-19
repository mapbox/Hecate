//! Grid and Geometry types in screen coordinates

#[derive(Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
}

#[derive(Debug, PartialEq)]
pub struct MultiPoint {
    pub points: Vec<Point>,
}

#[derive(Debug, PartialEq)]
pub struct LineString {
    pub points: Vec<Point>,
}

#[derive(Debug, PartialEq)]
pub struct MultiLineString {
    pub lines: Vec<LineString>,
}

#[derive(Debug, PartialEq)]
pub struct Polygon {
    pub rings: Vec<LineString>,
}

#[derive(Debug, PartialEq)]
pub struct MultiPolygon {
    pub polygons: Vec<Polygon>,
}
