extern crate env_logger;

use glx::graphics::*;

use euclid::*;

fn main() {
    // This should show a filled circle that fades angularly along the palette gradient
    // This seems to be able to handle 100,000 lines but not 1,000,000
    let n = 1000;
    leggo(
        (0..n)
            .map(|i| {
                let ratio = (i as f32) / (n as f32);
                let angle = ratio * 2.0 * std::f32::consts::PI;
                StyledGeom {
                    geom: Geom::Lines {
                        points: vec![
                            Point2D::new(0.0, 0.0),
                            Point2D::new(angle.cos(), angle.sin()),
                        ],
                        width: 0.002,
                    },
                    color: scale_temperature(ratio, 16.0),
                }
            })
            .collect(),
        Box2D::new(Point2D::new(-1.0, -1.0), Point2D::new(1.0, 1.0)),
    );
}
