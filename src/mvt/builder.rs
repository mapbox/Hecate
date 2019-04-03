use crate::mvt::geom_encoder::{encode_geometry, encode_geometry_type};
use crate::mvt::geom::Geometry;
use crate::mvt::grid;
use crate::mvt::proto;
use std::borrow::Cow;

#[derive(Debug)]
pub struct Tile<'a> {
    bbox: &'a grid::Extent,
    layers: Vec<Layer<'a>>,
}

impl<'a> Tile<'a> {
    pub fn new<'b: 'a>(bbox: &'b grid::Extent) -> Tile<'a> {
        Tile {
            bbox: bbox,
            layers: Vec::new(),
        }
    }

    pub fn add_layer<'b: 'a>(&mut self, layer: Layer<'b>) {
        self.layers.push(layer);
    }

    pub fn layers(&self) -> &[Layer<'a>] {
        self.layers.as_slice()
    }

    pub fn encode(self, grid: &grid::Grid) -> proto::Tile {
        let mut vec_tile = proto::Tile::new();
        for layer in self.layers.into_iter() {
            let mut vec_layer = proto::Tile_Layer::new();
            vec_layer.set_version(2);
            vec_layer.set_name(String::from(layer.name));
            vec_layer.set_extent(4096);
            for feature in layer.features.into_iter() {
                let mut vec_feature = proto::Tile_Feature::new();
                for property in feature.props.into_iter() {
                    let mut vec_value = proto::Tile_Value::new();
                    match property.value {
                        Value::String(ref v) => vec_value.set_string_value(v.clone()),
                        Value::F32(v) => vec_value.set_float_value(v),
                        Value::F64(v) => vec_value.set_double_value(v),
                        Value::I64(v) => vec_value.set_int_value(v),
                        Value::U64(v) => vec_value.set_uint_value(v),
                        Value::Bool(v) => vec_value.set_bool_value(v),
                    }
                    let key = match vec_layer.get_keys().iter().position(|k| *k == property.key) {
                        None => {
                            vec_layer.mut_keys().push(String::from(property.key));
                            vec_layer.get_keys().len() - 1
                        }
                        Some(idx) => idx,
                    };
                    vec_feature.mut_tags().push(key as u32);
                    let value = match vec_layer.get_values().iter().position(|v| *v == vec_value) {
                        None => {
                            vec_layer.mut_values().push(vec_value);
                            vec_layer.get_values().len() - 1
                        }
                        Some(idx) => idx,
                    };
                    vec_feature.mut_tags().push(value as u32);
                }
                vec_feature.set_field_type(encode_geometry_type(&feature.geom));
                vec_feature.set_geometry(encode_geometry(self.bbox, 4096, grid.reverse_y, feature.geom).vec());
                vec_layer.mut_features().push(vec_feature);
            }
            vec_tile.mut_layers().push(vec_layer);
        }
        vec_tile
    }
}

#[derive(Debug)]
pub struct Layer<'a> {
    name: Cow<'a, str>,
    features: Vec<Feature<'a>>,
}

impl<'a> Layer<'a> {
    pub fn new<S>(name: S) -> Layer<'a>
        where S: Into<Cow<'a, str>>
    {
        Layer {
            name: name.into(),
            features: Vec::new(),
        }
    }

    pub fn add_feature<'b: 'a>(&mut self, feature: Feature<'b>) {
        self.features.push(feature);
    }

    pub fn features(&self) -> &[Feature<'a>] {
        self.features.as_slice()
    }
}

#[derive(Debug)]
pub struct Property<'a> {
    pub key: Cow<'a, str>,
    pub value: Value,
}

#[derive(Debug)]
pub struct Feature<'a> {
    id: Option<u64>,
    geom: Geometry,
    props: Vec<Property<'a>>,
}

impl<'a> Feature<'a> {
    pub fn new(geom: Geometry) -> Feature<'a> {
        Feature {
            id: None,
            geom: geom,
            props: Vec::new(),
        }
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = Some(id);
    }

    pub fn add_property<S>(&mut self, key: S, value: Value)
        where S: Into<Cow<'a, str>>
    {
        self.props.push(Property {
            key: key.into(),
            value: value,
        })
    }

    pub fn geometry(&self) -> &Geometry {
        &self.geom
    }

    pub fn properties(&self) -> &[Property] {
        self.props.as_slice()
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    F32(f32),
    F64(f64),
    I64(i64),
    U64(u64),
    Bool(bool),
}
