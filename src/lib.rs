// For profiling:
//#![feature(alloc_system)]
//extern crate alloc_system;

use crate::protos::DenseNode;
use geo::algorithm::bearing::Bearing;
use geo::haversine_distance::HaversineDistance;
use geo_types::Point;

pub mod graphics;
pub mod protos;

use graphics::Point2DData;

#[derive(Clone, Copy, Debug)]
pub struct MyNode {
    pub x_y_meters: [f64; 2],
    pub node_id: i64,
}

impl MyNode {
    pub fn to_point2d(&self) -> Point2DData {
        Point2DData::new(self.x_y_meters[0] as f32, self.x_y_meters[1] as f32)
    }
}

impl PartialEq for MyNode {
    fn eq(&self, other: &Self) -> bool {
        self.x_y_meters == other.x_y_meters
    }
}

impl rstar::Point for MyNode {
    type Scalar = f64;
    const DIMENSIONS: usize = 2;

    fn generate(generator: impl Fn(usize) -> Self::Scalar) -> Self {
        MyNode {
            x_y_meters: <[f64; 2]>::generate(generator),
            node_id: -9999,
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        self.x_y_meters.nth(index)
    }

    fn nth_mut(&mut self, _index: usize) -> &mut Self::Scalar {
        unimplemented!()
    }
}

const DEG_TO_RAD: f32 = 2.0 * std::f32::consts::PI / 360.0;

pub fn lat_lon_to_x_y(centroid: &Point<f32>, lat_lon: (f32, f32)) -> Point2DData {
    // The geo library takes (lon, lat) since it's like (x, y); x is lon, y is lat
    let centroid_lon_lat = Point::new(centroid.y(), centroid.x());
    let distance_from_location =
        centroid_lon_lat.haversine_distance(&Point::new(lat_lon.1, lat_lon.0));
    let bearing_from_location =
        centroid_lon_lat.bearing(Point::new(lat_lon.1, lat_lon.0)) * DEG_TO_RAD;
    Point2DData::new(
        distance_from_location * bearing_from_location.sin(),
        -distance_from_location * bearing_from_location.cos(),
    )
}

pub fn dense_node_to_x_y(node: &DenseNode, centroid: Point<f32>) -> Point2DData {
    let lat = node.lat as f32 / 10000000.0;
    let lon = node.lon as f32 / 10000000.0;
    lat_lon_to_x_y(&centroid, (lat, lon))
}
