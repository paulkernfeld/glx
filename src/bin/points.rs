extern crate env_logger;

use glx::graphics;
use glx::graphics::*;

use euclid::*;
use log::*;

fn main() {
    // This should render a square that's half the height of the screen, right in the middle of the
    // screen.
    graphics::leggo(
        vec![
            StyledGeom {
                geom: Geom::Point(Point2D::new(-0.5, -0.5)),
                color: [1.0, 1.0, 0.0],
            },
            StyledGeom {
                geom: Geom::Point(Point2D::new(0.5, -0.5)),
                color: [1.0, 0.0, 1.0],
            },
            StyledGeom {
                geom: Geom::Point(Point2D::new(0.5, 0.5)),
                color: [0.0, 1.0, 1.0],
            },
            StyledGeom {
                geom: Geom::Point(Point2D::new(-0.5, 0.5)),
                color: [0.5, 0.5, 0.5],
            },
        ],
        Box2D::new(Point2D::new(-1.0, -1.0), Point2D::new(1.0, 1.0)),
    );
}
