extern crate env_logger;

use glx::graphics::*;

use euclid::*;

fn main() {
    leggo(
        Text {
            text: "hello world".to_string(),
            location: Point2D::new(0.0, 0.0),
        },
        Box2D::new(Point2D::new(-1.0, -1.0), Point2D::new(1.0, 1.0)),
    );
}
