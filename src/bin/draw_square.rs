extern crate env_logger;

use glx::graphics;
use glx::graphics::*;

use log::*;

fn main() {
    // A no-op viewport
    // This should render a black transparent square that's half the height of the screen, right in
    // the middle of the screen.
    graphics::leggo(
        vec![StyledGeom {
            geom: Geom::Polygon(vec![
                Point2DData::new(0.25, 0.25),
                Point2DData::new(0.75, 0.25),
                Point2DData::new(0.75, 0.75),
                Point2DData::new(0.25, 0.75),
            ]),
            color: [0.0, 0.0, 0.0, 0.5],
        }],
        Box2DData::new(Point2DData::new(0.0, 0.0), Point2DData::new(1.0, 1.0)),
    );
}
