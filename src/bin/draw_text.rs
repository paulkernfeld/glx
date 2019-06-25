extern crate env_logger;

use glx::graphics::*;

fn main() {
    leggo(
        Text {
            text: "hello world".to_string(),
            location: Point2DData::new(0.0, 0.0),
        },
        Box2DData::new(Point2DData::new(-1.0, -1.0), Point2DData::new(1.0, 1.0)),
    );
}
