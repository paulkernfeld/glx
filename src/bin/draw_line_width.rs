extern crate env_logger;

use glx::graphics;
use glx::graphics::*;

fn main() {
    // This should show two lines that are exactly as wide as they are long, i.e. squares
    graphics::leggo(
        vec![
            StyledGeom {
                geom: Geom::Lines {
                    points: vec![Point2DData::new(-1.0, -0.5), Point2DData::new(0.0, -0.5)],
                    width: 1.0,
                },
                color: [1.0, 0.0, 1.0],
            },
            StyledGeom {
                geom: Geom::Lines {
                    points: vec![Point2DData::new(0.5, 0.0), Point2DData::new(0.5, 1.0)],
                    width: 1.0,
                },
                color: [1.0, 0.0, 1.0],
            },
        ],
        Box2DData::new(Point2DData::new(-1.0, -1.0), Point2DData::new(1.0, 1.0)),
    );
}
