extern crate env_logger;

use glx::graphics;
use glx::graphics::*;

use euclid::*;
use log::*;

fn main() {
    let viewport = Box2DData::new(Point2DData::new(-2.0, -2.0), Point2DData::new(2.0, 2.0));
    let fn_grid = FnGrid {
        viewport,
        cell_size: 0.95,
        color_fn: |point: Point2DData| [0.0, (point.x + 2.0) / 4.0, (point.y + 2.0) / 4.0, 1.0],
        label_fn: |point: Point2DData| format!("{:?}", point),
    };

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
    );
}
