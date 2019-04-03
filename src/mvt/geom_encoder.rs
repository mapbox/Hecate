//! Encode geometries according to MVT spec
//! https://github.com/mapbox/proto-tile-spec/tree/master/2.1

use crate::mvt::geom;
use crate::mvt::grid;
use crate::mvt::screen;
use crate::mvt::proto;
use postgis::ewkb;

/// Command to be executed and the number of times that the command will be executed
/// https://github.com/mapbox/proto-tile-spec/tree/master/2.1#431-command-integers
struct CommandInteger(u32);

enum Command {
    MoveTo = 1,
    LineTo = 2,
    ClosePath = 7,
}

impl CommandInteger {
    fn new(id: Command, count: u32) -> CommandInteger {
        CommandInteger(((id as u32) & 0x7) | (count << 3))
    }
    #[cfg(test)]
    fn id(&self) -> u32 {
        self.0 & 0x7
    }
    #[cfg(test)]
    fn count(&self) -> u32 {
        self.0 >> 3
    }
}

#[test]
fn test_commands() {
    assert_eq!(CommandInteger(9).id(), Command::MoveTo as u32);
    assert_eq!(CommandInteger(9).count(), 1);

    assert_eq!(CommandInteger::new(Command::MoveTo, 1).0, 9);
    assert_eq!(CommandInteger::new(Command::LineTo, 3).0, 26);
    assert_eq!(CommandInteger::new(Command::ClosePath, 1).0, 15);
}


/// Commands requiring parameters are followed by a ParameterInteger for each parameter required by that command
/// https://github.com/mapbox/proto-tile-spec/tree/master/2.1#432-parameter-integers
struct ParameterInteger(u32);

impl ParameterInteger {
    fn new(value: i32) -> ParameterInteger {
        ParameterInteger(((value << 1) ^ (value >> 31)) as u32)
    }
    #[cfg(test)]
    fn value(&self) -> i32 {
        ((self.0 >> 1) as i32) ^ (-((self.0 & 1) as i32))
    }
}

#[test]
fn test_paremeters() {
    assert_eq!(ParameterInteger(50).value(), 25);
    assert_eq!(ParameterInteger::new(25).value(), 25);
}


pub struct CommandSequence(pub Vec<u32>);

impl CommandSequence {
    fn new() -> CommandSequence {
        CommandSequence(Vec::new())
    }
    pub fn vec(&self) -> Vec<u32> {
        self.0.clone() // FIXME: ref
    }
    #[cfg(test)]
    fn append(&mut self, other: &mut CommandSequence) {
        self.0.append(&mut other.0);
    }
    fn push(&mut self, value: u32) {
        self.0.push(value);
    }
}

#[test]
fn test_sequence() {
    let mut seq = CommandSequence::new();
    seq.push(CommandInteger::new(Command::MoveTo, 1).0);
    seq.push(ParameterInteger::new(25).0);
    seq.push(ParameterInteger::new(17).0);
    assert_eq!(seq.0, &[9, 50, 34]);

    let mut seq2 = CommandSequence::new();
    seq2.push(CommandInteger::new(Command::MoveTo, 1).0);
    seq.append(&mut seq2);
    assert_eq!(seq.0, &[9, 50, 34, 9]);
}

pub fn encode_geometry(bbox: &grid::Extent,
                       scale: u32,
                       reverse_y: bool,
                       geom: geom::Geometry)
                       -> CommandSequence {
    match geom {
        ewkb::GeometryT::Point(ref g) => {
            screen::Point::from_geom(bbox, scale, reverse_y, g).encode()
        }
        ewkb::GeometryT::MultiPoint(ref g) => {
            screen::MultiPoint::from_geom(bbox, scale, reverse_y, g).encode()
        }
        ewkb::GeometryT::LineString(ref g) => {
            screen::LineString::from_geom(bbox, scale, reverse_y, g).encode()
        }
        ewkb::GeometryT::MultiLineString(ref g) => {
            screen::MultiLineString::from_geom(bbox, scale, reverse_y, g).encode()
        }
        ewkb::GeometryT::Polygon(ref g) => {
            screen::Polygon::from_geom(bbox, scale, reverse_y, g).encode()
        }
        ewkb::GeometryT::MultiPolygon(ref g) => {
            screen::MultiPolygon::from_geom(bbox, scale, reverse_y, g).encode()
        }
        ewkb::GeometryT::GeometryCollection(_) => panic!("GeometryCollection not supported"),
    }
}

pub fn encode_geometry_type(g: &geom::Geometry) -> proto::Tile_GeomType {
    match g {
        &ewkb::GeometryT::Point(_) => proto::Tile_GeomType::POINT,
        &ewkb::GeometryT::LineString(_) => proto::Tile_GeomType::LINESTRING,
        &ewkb::GeometryT::Polygon(_) => proto::Tile_GeomType::POLYGON,
        &ewkb::GeometryT::MultiPoint(_) => proto::Tile_GeomType::POINT,
        &ewkb::GeometryT::MultiLineString(_) => proto::Tile_GeomType::LINESTRING,
        &ewkb::GeometryT::MultiPolygon(_) => proto::Tile_GeomType::POLYGON,
        &ewkb::GeometryT::GeometryCollection(_) => proto::Tile_GeomType::UNKNOWN,
    }
}

pub trait EncodableGeom {
    fn encode(&self) -> CommandSequence {
        let mut seq = CommandSequence::new();
        self.encode_from(&screen::Point::origin(), &mut seq);
        seq
    }
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence);
}

impl EncodableGeom for screen::Point {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        seq.push(CommandInteger::new(Command::MoveTo, 1).0);
        seq.push(ParameterInteger::new(self.x.saturating_sub(startpos.x)).0);
        seq.push(ParameterInteger::new(self.y.saturating_sub(startpos.y)).0);
    }
}

impl EncodableGeom for screen::MultiPoint {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        seq.push(CommandInteger::new(Command::MoveTo, self.points.len() as u32).0);
        let (mut posx, mut posy) = (startpos.x, startpos.y);
        for point in &self.points {
            seq.push(ParameterInteger::new(point.x.saturating_sub(posx)).0);
            seq.push(ParameterInteger::new(point.y.saturating_sub(posy)).0);
            posx = point.x;
            posy = point.y;
        }
    }
}

impl EncodableGeom for screen::LineString {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        if self.points.len() > 0 {
            self.points[0].encode_from(startpos, seq);
            seq.push(CommandInteger::new(Command::LineTo, (self.points.len() - 1) as u32).0);
            for i in 1..self.points.len() {
                let ref pos = &self.points[i - 1];
                let ref point = &self.points[i];
                seq.push(ParameterInteger::new(point.x.saturating_sub(pos.x)).0);
                seq.push(ParameterInteger::new(point.y.saturating_sub(pos.y)).0);
            }
        }
    }
}
impl screen::LineString {
    fn encode_ring_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        // almost same as LineString.encode_from, with ClosePath instead of last point
        if self.points.len() > 0 {
            self.points[0].encode_from(startpos, seq);
            seq.push(CommandInteger::new(Command::LineTo, (self.points.len() - 2) as u32).0);
            for i in 1..self.points.len() - 1 {
                let ref pos = &self.points[i - 1];
                let ref point = &self.points[i];
                seq.push(ParameterInteger::new(point.x.saturating_sub(pos.x)).0);
                seq.push(ParameterInteger::new(point.y.saturating_sub(pos.y)).0);
            }
            seq.push(CommandInteger::new(Command::ClosePath, 1).0);
        }
    }
}

impl EncodableGeom for screen::MultiLineString {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        let mut pos = startpos;
        for line in &self.lines {
            if line.points.len() > 0 {
                line.encode_from(&pos, seq);
                pos = &line.points[line.points.len() - 1];
            }
        }
    }
}

impl EncodableGeom for screen::Polygon {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        let mut pos = startpos;
        for line in &self.rings {
            if line.points.len() > 1 {
                line.encode_ring_from(&pos, seq);
                pos = &line.points[line.points.len() - 2];
            }
        }
    }
}

impl EncodableGeom for screen::MultiPolygon {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        let mut pos = startpos;
        for polygon in &self.polygons {
            for line in &polygon.rings {
                if line.points.len() > 1 {
                    line.encode_ring_from(&pos, seq);
                    pos = &line.points[line.points.len() - 2];
                }
            }
        }
    }
}

pub trait ScreenGeom<T> {
    /// Convert geometry into screen coordinates
    fn from_geom(bbox: &grid::Extent, scale: u32, reverse_y: bool, geom: &T) -> Self;
}

impl ScreenGeom<geom::Point> for screen::Point {
    fn from_geom(bbox: &grid::Extent, scale: u32, reverse_y: bool, point: &geom::Point) -> Self {
        let x_span = bbox.maxx - bbox.minx;
        let y_span = bbox.maxy - bbox.miny;
        let mut screen_geom = screen::Point {
            x: ((point.x - bbox.minx) * scale as f64 / x_span) as i32,
            y: ((point.y - bbox.miny) * scale as f64 / y_span) as i32,
        };
        if reverse_y {
            screen_geom.y = (scale as i32).saturating_sub(screen_geom.y)
        };
        screen_geom
    }
}

impl ScreenGeom<geom::MultiPoint> for screen::MultiPoint {
    fn from_geom(bbox: &grid::Extent,
                 scale: u32,
                 reverse_y: bool,
                 multipoint: &geom::MultiPoint)
                 -> Self {
        let mut screen_geom = screen::MultiPoint { points: Vec::new() };
        for point in &multipoint.points {
            screen_geom.points.push(screen::Point::from_geom(bbox, scale, reverse_y, point));
        }
        screen_geom
    }
}

impl ScreenGeom<geom::LineString> for screen::LineString {
    fn from_geom(bbox: &grid::Extent,
                 scale: u32,
                 reverse_y: bool,
                 line: &geom::LineString)
                 -> Self {
        let mut screen_geom = screen::LineString { points: Vec::new() };
        for point in &line.points {
            screen_geom.points.push(screen::Point::from_geom(bbox, scale, reverse_y, point));
        }
        screen_geom
    }
}

impl ScreenGeom<geom::MultiLineString> for screen::MultiLineString {
    fn from_geom(bbox: &grid::Extent,
                 scale: u32,
                 reverse_y: bool,
                 multiline: &geom::MultiLineString)
                 -> Self {
        let mut screen_geom = screen::MultiLineString { lines: Vec::new() };
        for line in &multiline.lines {
            screen_geom.lines.push(screen::LineString::from_geom(bbox, scale, reverse_y, line));
        }
        screen_geom
    }
}

impl ScreenGeom<geom::Polygon> for screen::Polygon {
    fn from_geom(bbox: &grid::Extent,
                 scale: u32,
                 reverse_y: bool,
                 polygon: &geom::Polygon)
                 -> Self {
        let mut screen_geom = screen::Polygon { rings: Vec::new() };
        for line in &polygon.rings {
            screen_geom.rings.push(screen::LineString::from_geom(bbox, scale, reverse_y, line));
        }
        screen_geom
    }
}

impl ScreenGeom<geom::MultiPolygon> for screen::MultiPolygon {
    fn from_geom(bbox: &grid::Extent,
                 scale: u32,
                 reverse_y: bool,
                 multipolygon: &geom::MultiPolygon)
                 -> Self {
        let mut screen_geom = screen::MultiPolygon { polygons: Vec::new() };
        for polygon in &multipolygon.polygons {
            screen_geom.polygons.push(screen::Polygon::from_geom(bbox, scale, reverse_y, polygon));
        }
        screen_geom
    }
}
