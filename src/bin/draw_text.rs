extern crate env_logger;

use glx::graphics::*;

use euclid::*;

fn main() {
    leggo(
        vec![StyledGeom {
            geom: Geom::Lines {
                points: vec![Point2D::new(-1.0, -0.5), Point2D::new(0.0, -0.5)],
                width: 0.1,
            },
            color: [1.0, 0.0, 1.0],
        }],
        Box2D::new(Point2D::new(-1.0, -1.0), Point2D::new(1.0, 1.0)),
    );
}
