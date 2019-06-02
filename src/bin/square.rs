extern crate env_logger;

use openstreetmap::graphics;
use openstreetmap::graphics::*;

use euclid::*;
use log::*;

fn main() {
    // A no-op viewport
    // This should render a pink square that's half the height of the screen, right in the middle
    // of the screen.
    graphics::leggo(
        vec![StyledGeom {
            geom: Geom::Polygon(vec![
                Point2D::new(0.25, 0.25),
                Point2D::new(0.75, 0.25),
                Point2D::new(0.75, 0.75),
                Point2D::new(0.25, 0.75),
            ]),
            color: [0.9, 0.1, 0.5],
        }],
        Box2D::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)),
    );
}
