use crate::graphics::*;
use std::path::PathBuf;

pub struct Series {
    pub title: String,
    pub color: [f32; 4],
}

pub struct Legend {
    pub title: String,
    pub series: Vec<Series>,
    /// In data space, unfortunately. Should this eventually be in screen space or -1..1 space?
    pub area: Box2DData,
}

impl Legend {
    pub fn render(&self) -> impl Render {
        let mut styled_geoms = vec![StyledGeom {
            geom: Geom::from_box2d(&self.area),
            color: [0.5, 0.5, 1.0, 1.0],
        }];

        let serieses = Box2DData::new(
            Point2DData::new(
                self.area.min.x,
                self.area.min.y * 0.75 + self.area.max.y * 0.25,
            ),
            Point2DData::new(self.area.max.x, self.area.max.y),
        );

        for (series, series_box) in self
            .series
            .iter()
            .zip(slice_box2d(serieses, self.series.len()))
            {
                styled_geoms.push(StyledGeom {
                    geom: Geom::from_box2d(&series_box),
                    color: series.color,
                });
            }

        let mut texts = vec![Text {
            text: self.title.clone(),
            location: Point2DData::new(
                self.area.min.x * 0.5 + self.area.max.x * 0.5,
                self.area.min.y * 0.9 + self.area.max.y * 0.1,
            ),
        }];

        let serieses = Box2DData::new(
            Point2DData::new(
                self.area.min.x,
                self.area.min.y * 0.75 + self.area.max.y * 0.25,
            ),
            Point2DData::new(self.area.max.x, self.area.max.y),
        );

        for (series, series_box) in self
            .series
            .iter()
            .zip(slice_box2d(serieses, self.series.len()))
            {
                texts.push(Text {
                    text: series.title.clone(),
                    location: series_box.center(),
                })
            }

        let x: Layers<Box<dyn Render>> = Layers(vec![Box::new(Layer(styled_geoms)), Box::new(Layer(texts))]);
        x
    }
}


mod tests {
    use super::*;

    #[test]
    fn test_layers_legend() {
        let viewport = Box2DData::new(Point2DData::new(-1.0, -1.0), Point2DData::new(1.0, 1.0));
        let render: Layers<Box<dyn Render>> = Layers(vec![
            Box::new(StyledGeom {
                geom: Geom::from_box2d(&viewport),
                color: [1.0, 0.0, 0.0, 1.0],
            }),
            Box::new(Legend {
                title: String::from("Background"),
                series: (0..10)
                    .map(|i| Series {
                        title: format!("Background {}", i),
                        color: [0.0, 0.0, i as f32 / 9.0, 1.0],
                    })
                    .collect(),
                area: Box2DData::new(Point2DData::new(-1.0, -1.0), Point2DData::new(0.5, 0.5)),
            }.render()),
            Box::new(Legend {
                title: String::from("Foreground"),
                series: (0..10)
                    .map(|i| Series {
                        title: format!("Foreground {}", i),
                        color: [0.0, i as f32 / 9.0, 0.0, 1.0],
                    })
                    .collect(),
                area: Box2DData::new(Point2DData::new(-0.5, -0.5), Point2DData::new(1.0, 1.0)),
            }.render()),
        ]);
        capture(
            render,
            viewport,
            PathBuf::from("output/layers_legend.png"),
            TEST_SIZE,
        );
    }

    #[test]
    fn test_legend() {
        capture(
            Legend {
                title: String::from("Legend"),
                series: (0..10)
                    .map(|i| Series {
                        title: format!("Series {}", i),
                        color: [0.0, 0.0, i as f32 / 9.0, 1.0],
                    })
                    .collect(),
                area: Box2DData::new(Point2DData::new(-0.5, -0.5), Point2DData::new(0.5, 0.5)),
            }.render(),
            Box2DData::new(Point2DData::new(-1.0, -1.0), Point2DData::new(1.0, 1.0)),
            PathBuf::from("output/legend.png"),
            TEST_SIZE,
        );
    }
}