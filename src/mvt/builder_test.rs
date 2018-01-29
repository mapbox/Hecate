use mvt::builder::{Tile, Layer, Feature, Value};
use mvt::geom::{Geometry, Point};
use mvt::grid::{Grid, Extent};
use postgis::ewkb;

#[test]
fn test_build_tile() {
    let bbox = Extent {
        minx: 958826.08,
        miny: 5987771.04,
        maxx: 978393.96,
        maxy: 6007338.92,
    };
    let mut tile = Tile::new(&bbox);
    let mut layer = Layer::new("points");
    let geom: Geometry = ewkb::GeometryT::Point(Point::new(960000.0, 6002729.0, Some(3857)));
    let mut feature = Feature::new(geom);
    feature.add_property("hello", Value::String(String::from("world")));
    feature.add_property("h", Value::String(String::from("world")));
    feature.add_property("count", Value::F64(1.23));
    layer.add_feature(feature);

    let geom: Geometry = ewkb::GeometryT::Point(Point::new(960000.0, 6002729.0, Some(3857)));
    let mut feature = Feature::new(geom);
    feature.add_property("hello", Value::String(String::from("again")));
    feature.add_property("count", Value::I64(2));
    layer.add_feature(feature);

    tile.add_layer(layer);

    // Encode the tile as protobuf structs
    let grid = Grid::wgs84();
    let data = tile.encode(&grid);
    println!("{:#?}", data);
    assert_eq!(TILE_EXAMPLE, &*format!("{:#?}", data));
}

const TILE_EXAMPLE: &'static str = r#"Tile {
    layers: [
        Tile_Layer {
            version: Some(
                2
            ),
            name: Some("points"),
            features: [
                Tile_Feature {
                    id: None,
                    tags: [
                        0,
                        0,
                        1,
                        0,
                        2,
                        1
                    ],
                    field_type: Some(
                        POINT
                    ),
                    geometry: [
                        9,
                        490,
                        6262
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: CachedSize {
                        size: Cell {
                            value: 0
                        }
                    }
                },
                Tile_Feature {
                    id: None,
                    tags: [
                        0,
                        2,
                        2,
                        3
                    ],
                    field_type: Some(
                        POINT
                    ),
                    geometry: [
                        9,
                        490,
                        6262
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: CachedSize {
                        size: Cell {
                            value: 0
                        }
                    }
                }
            ],
            keys: [
                "hello",
                "h",
                "count"
            ],
            values: [
                Tile_Value {
                    string_value: Some("world"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: CachedSize {
                        size: Cell {
                            value: 0
                        }
                    }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: Some(
                        1.23
                    ),
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: CachedSize {
                        size: Cell {
                            value: 0
                        }
                    }
                },
                Tile_Value {
                    string_value: Some("again"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: CachedSize {
                        size: Cell {
                            value: 0
                        }
                    }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: None,
                    int_value: Some(
                        2
                    ),
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: CachedSize {
                        size: Cell {
                            value: 0
                        }
                    }
                }
            ],
            extent: Some(
                4096
            ),
            unknown_fields: UnknownFields {
                fields: None
            },
            cached_size: CachedSize {
                size: Cell {
                    value: 0
                }
            }
        }
    ],
    unknown_fields: UnknownFields {
        fields: None
    },
    cached_size: CachedSize {
        size: Cell {
            value: 0
        }
    }
}"#;
