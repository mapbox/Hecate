#[derive(Debug, PartialEq)]
pub struct Extent {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

/// Min and max grid cell numbers
#[derive(Debug, PartialEq)]
pub struct ExtentInt {
    pub minx: u32,
    pub miny: u32,
    pub maxx: u32,
    pub maxy: u32,
}

#[derive(Debug, PartialEq)]
pub enum Origin {
    TopLeft,
    BottomLeft, // TopRight, BottomRight
}

#[derive(Debug, PartialEq)]
pub enum Unit {
    M,
    DD,
    Ft,
}

// Credits: MapCache by Thomas Bonfort (http://mapserver.org/mapcache/)
#[derive(Debug)]
pub struct Grid {
    /// The width and height of an individual tile, in pixels.
    width: u16,
    height: u16,
    /// The geographical extent covered by the grid, in ground units (e.g. meters, degrees, feet, etc.).
    /// Must be specified as 4 floating point numbers ordered as minx, miny, maxx, maxy.
    /// The (minx,miny) point defines the origin of the grid, i.e. the pixel at the bottom left of the
    /// bottom-left most tile is always placed on the (minx,miny) geographical point.
    /// The (maxx,maxy) point is used to determine how many tiles there are for each zoom level.
    pub extent: Extent,
    /// Spatial reference system (PostGIS SRID).
    pub srid: i32,
    /// Grid units
    pub units: Unit,
    /// This is a list of resolutions for each of the zoom levels defined by the grid.
    /// This must be supplied as a list of positive floating point values, ordered from largest to smallest.
    /// The largest value will correspond to the grid’s zoom level 0. Resolutions
    /// are expressed in “units-per-pixel”,
    /// depending on the unit used by the grid (e.g. resolutions are in meters per
    /// pixel for most grids used in webmapping).
    resolutions: Vec<f64>,
    /// Grid origin
    pub origin: Origin,
    /// Whether y tiles go from South->North (normal) or North->South (reversed)
    pub reverse_y: bool,
}

impl Grid {
    /// WGS84 grid
    pub fn wgs84() -> Grid {
        Grid {
            width: 256,
            height: 256,
            extent: Extent {
                minx: -180.0,
                miny: -90.0,
                maxx: 180.0,
                maxy: 90.0,
            },
            srid: 4326,
            units: Unit::DD,
            resolutions: vec![0.703_125_000_000_000,
                              0.351_562_500_000_000,
                              0.175_781_250_000_000,
                              8.789_062_500_000_00e-2,
                              4.394_531_250_000_00e-2,
                              2.197_265_625_000_00e-2,
                              1.098_632_812_500_00e-2,
                              5.493_164_062_500_00e-3,
                              2.746_582_031_250_00e-3,
                              1.373_291_015_625_00e-3,
                              6.866_455_078_125_00e-4,
                              3.433_227_539_062_50e-4,
                              1.716_613_769_531_25e-4,
                              8.583_068_847_656_25e-5,
                              4.291_534_423_828_12e-5,
                              2.145_767_211_914_06e-5,
                              1.072_883_605_957_03e-5,
                              5.364_418_029_785_16e-6],
            origin: Origin::BottomLeft,
            reverse_y: false,
        }
    }

    /// Web Mercator grid (Google maps compatible)
    pub fn web_mercator() -> Grid {
        Grid {
            width: 256,
            height: 256,
            extent: Extent {
                minx: -20_037_508.342_789_248_0,
                miny: -20_037_508.342_789_248_0,
                maxx: 20_037_508.342_789_248_0,
                maxy: 20_037_508.342_789_248_0
            },
            srid: 3857,
            units: Unit::M,
            resolutions: vec![156_543.033_928_041_0,
                              78_271.516_964_020_48,
                              39_135.758_482_010_23,
                              19_567.879_241_005_12,
                              9_783.939_620_502_561,
                              4_891.969_810_251_280,
                              2_445.984_905_125_640,
                              1_222.992_452_562_820,
                              611.496_226_281_410_0,
                              305.748_113_140_704_8,
                              152.874_056_570_352_5,
                              76.437_028_285_176_24,
                              38.218_514_142_588_13,
                              19.109_257_071_294_06,
                              9.554_628_535_647_032,
                              4.777_314_267_823_516,
                              2.388_657_133_911_758,
                              1.194_328_566_955_879,
                              0.597_164_283_477_939_5,
                              0.298_582_141_738_970_0,
                              0.149_291_070_869_485_0,
                              0.074_645_535_434_742_4,
                              0.037_322_767_717_371_2],
            origin: Origin::BottomLeft,
            reverse_y: true,
        }
    }

    pub fn nlevels(&self) -> u8 {
        self.resolutions.len() as u8
    }
    pub fn maxzoom(&self) -> u8 {
        self.nlevels() - 1
    }
    pub fn pixel_width(&self, zoom: u8) -> f64 {
        self.resolutions[zoom as usize] //TODO: assumes grid unit 'm'
    }
    pub fn scale_denominator(&self, zoom: u8) -> f64 {
        let pixel_screen_width = 0.0254 / 96.0; //FIXME: assumes 96dpi - check with mapnik
        self.pixel_width(zoom) / pixel_screen_width
    }
    /// Extent of a given tile in the grid given its x, y, and z in TMS adressing scheme
    pub fn tile_extent(&self, z: u8, x: u32, y: u32) -> Extent {
        let zoom = z;
        let xtile = x;
        let ytile = if self.reverse_y {
            let res = self.resolutions[zoom as usize];
            let unitheight = self.height as f64 * res;
            let maxy = ((self.extent.maxy - self.extent.minx - 0.01 * unitheight) / unitheight)
                .ceil() as u32;
            maxy.saturating_sub(y).saturating_sub(1)
        } else {
            y
        };

        // based on mapcache_grid_get_tile_extent
        let res = self.resolutions[zoom as usize];
        let tile_sx = self.width as f64;
        let tile_sy = self.height as f64;
        match self.origin {
            Origin::BottomLeft => {
                Extent {
                    minx: self.extent.minx + (res * xtile as f64 * tile_sx),
                    miny: self.extent.miny + (res * ytile as f64 * tile_sy),
                    maxx: self.extent.minx + (res * (xtile + 1) as f64 * tile_sx),
                    maxy: self.extent.miny + (res * (ytile + 1) as f64 * tile_sy),
                }
            }
            Origin::TopLeft => {
                Extent {
                    minx: self.extent.minx + (res * xtile as f64 * tile_sx),
                    miny: self.extent.maxy - (res * (ytile + 1) as f64 * tile_sy),
                    maxx: self.extent.minx + (res * (xtile + 1) as f64 * tile_sx),
                    maxy: self.extent.maxy - (res * ytile as f64 * tile_sy),
                }
            }
        }
    }
    /// (maxx, maxy) of grid level
    pub fn level_limit(&self, zoom: u8) -> (u32, u32) {
        let res = self.resolutions[zoom as usize];
        let unitheight = self.height as f64 * res;
        let unitwidth = self.width as f64 * res;

        let maxy = ((self.extent.maxy - self.extent.miny - 0.01 * unitheight) / unitheight)
            .ceil() as u32;
        let maxx = ((self.extent.maxx - self.extent.miny - 0.01 * unitwidth) / unitwidth)
            .ceil() as u32;
        (maxx, maxy)
    }
    /// Tile index limits covering extent
    pub fn tile_limits(&self, extent: Extent, tolerance: i32) -> Vec<ExtentInt> {
        // Based on mapcache_grid_compute_limits
        const EPSILON: f64 = 0.000_000_1;
        let nlevels = self.resolutions.len() as u8;
        (0..nlevels)
            .map(|i| {
                let res = self.resolutions[i as usize];
                let unitheight = self.height as f64 * res;
                let unitwidth = self.width as f64 * res;
                let (level_maxx, level_maxy) = self.level_limit(i);

                let (mut minx, mut maxx, mut miny, mut maxy) =
                    match self.origin {
                        Origin::BottomLeft => {
                            ((((extent.minx - self.extent.minx) / unitwidth + EPSILON)
                                .floor() as i32) - tolerance,
                             (((extent.maxx - self.extent.minx) / unitwidth - EPSILON)
                                .ceil() as i32) + tolerance,
                             (((extent.miny - self.extent.miny) / unitheight + EPSILON)
                                .floor() as i32) - tolerance,
                             (((extent.maxy - self.extent.miny) / unitheight - EPSILON)
                                .ceil() as i32) + tolerance)
                        }
                        Origin::TopLeft => {
                            ((((extent.minx - self.extent.minx) / unitwidth + EPSILON)
                                .floor() as i32) - tolerance,
                             (((extent.maxx - self.extent.minx) / unitwidth - EPSILON)
                                .ceil() as i32) + tolerance,
                             (((self.extent.maxy - extent.maxy) / unitheight + EPSILON)
                                .floor() as i32) - tolerance,
                             (((self.extent.maxy - extent.miny) / unitheight - EPSILON)
                                .ceil() as i32) + tolerance)
                        }
                    };

                // to avoid requesting out-of-range tiles
                if minx < 0 {
                    minx = 0;
                }
                if maxx > level_maxx as i32 {
                    maxx = level_maxx as i32
                };
                if miny < 0 {
                    miny = 0
                };
                if maxy > level_maxy as i32 {
                    maxy = level_maxy as i32
                };

                ExtentInt {
                    minx: minx as u32,
                    maxx: maxx as u32,
                    miny: miny as u32,
                    maxy: maxy as u32,
                }
            })
            .collect()
    }
}
