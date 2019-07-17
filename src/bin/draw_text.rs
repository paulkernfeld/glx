extern crate env_logger;

use glx::graphics::*;

/// This should render some text in the center of the screen
fn main() {
    leggo(
        vec![
            Text {
                text: "center".to_string(),
                location: Point2DData::new(0.0, 0.0),
            },
            Text {
                text: "top left".to_string(),
                location: Point2DData::new(-1.0, -1.0),
            },
            Text {
                text: "bottom right".to_string(),
                location: Point2DData::new(1.0, 1.0),
            },
        ],
        Box2DData::new(Point2DData::new(-1.0, -1.0), Point2DData::new(1.0, 1.0)),
    );
}
