use glx::graphics;
use glx::graphics::*;

use euclid::*;
use log::*;

fn main() {
    // This should render a 40x40 grid with black in the bottom left and white in one corner. The
    // grid should be slightly falling off the screen.
    let viewport = Box2DData::new(Point2DData::new(-2.0, -2.0), Point2DData::new(2.0, 2.0));
    graphics::leggo(
        vec![FnGrid {
            viewport,
            cell_size: 1.0,
            color_fn: |point: Point2DData| [0.0, (point.x + 2.0) / 4.0, (point.y + 2.0) / 4.0, 1.0],
            label_fn: |point: Point2DData| format!("{}", point),
        }],
        viewport,
    );
}
