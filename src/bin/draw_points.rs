extern crate env_logger;

use glx::graphics;
use glx::graphics::*;

use euclid::*;
use log::*;
use std::path::PathBuf;

fn main() {
    // This should render a square that's half the height of the screen, right in the middle of the
    // screen.
    graphics::leggo(
        vec![
            StyledGeom {
                geom: Geom::Point(Point2DData::new(-0.5, -0.5)),
                color: [1.0, 1.0, 0.0, 1.0],
            },
            StyledGeom {
                geom: Geom::Point(Point2DData::new(0.5, -0.5)),
                color: [1.0, 0.0, 1.0, 1.0],
            },
            StyledGeom {
                geom: Geom::Point(Point2DData::new(0.5, 0.5)),
                color: [0.0, 1.0, 1.0, 1.0],
            },
            StyledGeom {
                geom: Geom::Point(Point2DData::new(-0.5, 0.5)),
                color: [0.5, 0.5, 0.5, 1.0],
            },
        ],
        Box2DData::new(Point2DData::new(-1.0, -1.0), Point2DData::new(1.0, 1.0)),
        PathBuf::from("points.png")
    );
}
