use log::*;

extern crate env_logger;
extern crate wgpu;

use either::Either;
use euclid::{Point2D, TypedPoint2D};
use lyon::tessellation::*;

use crate::graphics;

use euclid::*;

use lyon::path::Path;
use palette::{Gradient, Lch, Srgb};

/// Return a color scale from cold at 0 to warm at 1. This will draw attention towards higher
/// values.
///
/// This scale has high distinguishability.
pub fn scale_temperature(mut scalar: f32, n_chunks: f32) -> [f32; 4] {
    scalar = (scalar * n_chunks).floor() / (n_chunks - 1.0);
    let lightness = 70.0;
    let chroma = 90.0;
    match Srgb::from(
        Gradient::new(vec![
            Lch::new(lightness, chroma, 280.0),
            Lch::new(lightness, chroma, 60.0),
        ])
        .get(scalar),
    )
    .into_components()
    {
        (r, g, b) => [r, g, b, 1.0],
    }
}

/// Return a color scale from drab at 0 to colorful at 1. This will strongly draw attention towards
/// higher values.
///
/// This color scale has medium distinguishability.
pub fn scale_chroma(mut scalar: f32, n_chunks: f32) -> [f32; 4] {
    // Quantize the colors
    scalar = (scalar * n_chunks).floor() / (n_chunks - 1.0);
    let lightness = 70.0;
    match Srgb::from(
        Gradient::new(vec![
            Lch::new(lightness, 0.0, 60.0),
            Lch::new(lightness, 90.0, 60.0),
        ])
        .get(scalar),
    )
    .into_components()
    {
        (r, g, b) => [r, g, b, 1.0],
    }
}

/// This unit refers to "data space," i.e. the most raw version of the coordinates
pub enum DataUnit {}

pub type Point2DData = TypedPoint2D<f32, DataUnit>;
pub type Box2DData = TypedBox2D<f32, DataUnit>;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex {
    _pos: [f32; 2],
    _color: [f32; 4],
}

enum MyPath {
    Filled(Path),
    Stroked { path: Path, width: f32 },
}

// First version of a trait for rendering
pub trait Render {
    fn styled_geoms(&self) -> Vec<StyledGeom>;

    fn texts(&self) -> Vec<Text>;
}

// This improved rendering trait is aware of the viewport and is responsible for clipping
pub trait Render2 {
    // Eh too much complexity, plus this stuff needs to fit into the graphics card memory anyways
    // type IterGeoms: Iterator<Item=StyledGeom>;
    // type IterTexts: Iterator<Item=Text>;

    fn styled_geoms(&self, viewport: Box2DData) -> Vec<StyledGeom>;

    fn texts(&self, viewport: Box2DData) -> Vec<Text>;
}

impl<L: Render, R: Render> Render for Either<L, R> {
    fn styled_geoms(&self) -> Vec<StyledGeom> {
        match self {
            Either::Left(l) => l.styled_geoms(),
            Either::Right(r) => r.styled_geoms(),
        }
    }

    fn texts(&self) -> Vec<Text> {
        match self {
            Either::Left(l) => l.texts(),
            Either::Right(r) => r.texts(),
        }
    }
}

impl<L: Render2, R: Render2> Render2 for Either<L, R> {
    fn styled_geoms(&self, viewport: Box2DData) -> Vec<StyledGeom> {
        match self {
            Either::Left(l) => l.styled_geoms(viewport),
            Either::Right(r) => r.styled_geoms(viewport),
        }
    }

    fn texts(&self, viewport: Box2DData) -> Vec<Text> {
        match self {
            Either::Left(l) => l.texts(viewport),
            Either::Right(r) => r.texts(viewport),
        }
    }
}

/// Cells are implicitly based around the origin
pub struct FnGrid<F, G> {
    /// This is not necessary with Render2
    pub viewport: Option<Box2DData>,

    /// The side length of a cell, in data space
    pub cell_size: f32,

    /// Given the center of a grid cell, return the color to paint this grid cell
    pub color_fn: F,

    /// Given the center of a grid cell, return the label of this grid cell
    pub label_fn: G,
}

impl<F: Fn(Point2DData) -> [f32; 4], G: Fn(Point2DData) -> String> Render for FnGrid<F, G> {
    fn styled_geoms(&self) -> Vec<StyledGeom> {
        let viewport = self.viewport.unwrap();
        let min_x = (viewport.min.x / self.cell_size).floor() as isize;
        let min_y = (viewport.min.y / self.cell_size).floor() as isize;
        let max_x = (viewport.max.x / self.cell_size).floor() as isize;
        let max_y = (viewport.max.y / self.cell_size).floor() as isize;

        let mut cells = vec![];
        for x in min_x..=max_x {
            let cell_x_min = x as f32 * self.cell_size;
            for y in min_y..=max_y {
                let cell_y_min = y as f32 * self.cell_size;
                cells.push(StyledGeom {
                    geom: Geom::Polygon(vec![
                        Point2DData::new(cell_x_min, cell_y_min),
                        Point2DData::new(cell_x_min + self.cell_size, cell_y_min),
                        Point2DData::new(cell_x_min + self.cell_size, cell_y_min + self.cell_size),
                        Point2DData::new(cell_x_min, cell_y_min + self.cell_size),
                    ]),
                    color: (self.color_fn)(Point2DData::new(
                        cell_x_min + self.cell_size * 0.5,
                        cell_y_min + self.cell_size * 0.5,
                    )),
                })
            }
        }
        cells
    }

    fn texts(&self) -> Vec<Text> {
        let viewport = self.viewport.unwrap();
        let min_x = (viewport.min.x / self.cell_size).floor() as isize;
        let min_y = (viewport.min.y / self.cell_size).floor() as isize;
        let max_x = (viewport.max.x / self.cell_size).floor() as isize;
        let max_y = (viewport.max.y / self.cell_size).floor() as isize;

        let mut cells = vec![];
        for x in min_x..=max_x {
            let cell_x_min = x as f32 * self.cell_size;
            for y in min_y..=max_y {
                let cell_y_min = y as f32 * self.cell_size;
                let location = Point2DData::new(
                    cell_x_min + self.cell_size * 0.5,
                    cell_y_min + self.cell_size * 0.5,
                );
                cells.push(Text {
                    text: (self.label_fn)(location),
                    location,
                })
            }
        }
        cells
    }
}

impl<F: Fn(Point2DData) -> [f32; 4], G: Fn(Point2DData) -> String> Render2 for FnGrid<F, G> {
    fn styled_geoms(&self, viewport: Box2DData) -> Vec<StyledGeom> {
        assert_eq!(self.viewport, None);
        let min_x = (viewport.min.x / self.cell_size).floor() as isize;
        let min_y = (viewport.min.y / self.cell_size).floor() as isize;
        let max_x = (viewport.max.x / self.cell_size).floor() as isize;
        let max_y = (viewport.max.y / self.cell_size).floor() as isize;

        let mut cells = vec![];
        for x in min_x..=max_x {
            let cell_x_min = x as f32 * self.cell_size;
            for y in min_y..=max_y {
                let cell_y_min = y as f32 * self.cell_size;
                cells.push(StyledGeom {
                    geom: Geom::Polygon(vec![
                        Point2DData::new(cell_x_min, cell_y_min),
                        Point2DData::new(cell_x_min + self.cell_size, cell_y_min),
                        Point2DData::new(cell_x_min + self.cell_size, cell_y_min + self.cell_size),
                        Point2DData::new(cell_x_min, cell_y_min + self.cell_size),
                    ]),
                    color: (self.color_fn)(Point2DData::new(
                        cell_x_min + self.cell_size * 0.5,
                        cell_y_min + self.cell_size * 0.5,
                    )),
                })
            }
        }
        cells
    }

    fn texts(&self, viewport: Box2DData) -> Vec<Text> {
        assert_eq!(self.viewport, None);
        let min_x = (viewport.min.x / self.cell_size).floor() as isize;
        let min_y = (viewport.min.y / self.cell_size).floor() as isize;
        let max_x = (viewport.max.x / self.cell_size).floor() as isize;
        let max_y = (viewport.max.y / self.cell_size).floor() as isize;

        let mut cells = vec![];
        for x in min_x..=max_x {
            let cell_x_min = x as f32 * self.cell_size;
            for y in min_y..=max_y {
                let cell_y_min = y as f32 * self.cell_size;
                let location = Point2DData::new(
                    cell_x_min + self.cell_size * 0.5,
                    cell_y_min + self.cell_size * 0.5,
                );
                cells.push(Text {
                    text: (self.label_fn)(location),
                    location,
                })
            }
        }
        cells
    }
}

#[derive(Clone, Debug)]
pub struct StyledGeom {
    pub geom: Geom,
    pub color: [f32; 4],
}

impl Render for StyledGeom {
    fn styled_geoms(&self) -> Vec<StyledGeom> {
        vec![self.clone()]
    }

    fn texts(&self) -> Vec<Text> {
        vec![]
    }
}

//impl Render2 for StyledGeom {
//    fn styled_geoms(&self, viewport: Box2DData) -> Vec<StyledGeom> {
//        if self.geom.is_in(viewport) {
//            vec![self.clone()]
//        } else {
//            vec![]
//        }
//    }
//
//    fn texts(&self, viewport: Box2DData) -> Vec<Text> {
//        vec![]
//    }
//}
//
#[derive(Clone, Debug)]
pub struct Text {
    pub text: String,
    pub location: Point2DData,
}

impl Render for Text {
    fn styled_geoms(&self) -> Vec<StyledGeom> {
        vec![]
    }

    fn texts(&self) -> Vec<Text> {
        vec![self.clone()]
    }
}

impl Render2 for Text {
    fn styled_geoms(&self, viewport: Box2DData) -> Vec<StyledGeom> {
        vec![]
    }

    fn texts(&self, viewport: Box2DData) -> Vec<Text> {
        // TODO this is wrong
        if viewport.contains(&self.location) {
            vec![self.clone()]
        } else {
            vec![]
        }
    }
}

impl<R: Render> Render for Vec<R> {
    fn styled_geoms(&self) -> Vec<StyledGeom> {
        self.iter().flat_map(|r| r.styled_geoms()).collect()
    }

    fn texts(&self) -> Vec<Text> {
        self.iter().flat_map(|r| r.texts()).collect()
    }
}

/// This attempts to represent the underlying data
#[derive(Clone, Debug)]
pub enum Geom {
    Point(Point2DData),
    Lines {
        points: Vec<Point2DData>,
        width: f32,
    },
    Polygon(Vec<Point2DData>), // don't repeat the first point
                               //    Text(String), // This seems def. not a geom in the tidy data sense
}

pub enum PointStyle {
    Circle { radius: f32 },
}

/// Transform this point from data space into drawing space coordinates
fn transform_viewport(point: &Point2DData, viewport: &Box2DData) -> Point2D<f32> {
    Point2D::new(
        2.0 * (point.x - viewport.min.x) / (viewport.max.x - viewport.min.x) - 1.0,
        2.0 * (point.y - viewport.min.y) / (viewport.max.y - viewport.min.y) - 1.0,
    )
}

fn transform_viewport_1d(len: f32, viewport: &Box2DData) -> f32 {
    2.0 * len / (viewport.max.y - viewport.min.y)
}

fn geom_to_path(geom: Geom, viewport: Box2DData, screen: Vector2D<usize>) -> MyPath {
    let mut builder = Path::builder();

    match geom {
        Geom::Point(point) => {
            // 3px diameter is good
            let radius_px = 10.0;
            let point = transform_viewport(&point, &viewport);
            builder.move_to(point + Vector2D::new(radius_px / screen.x as f32, 0.0));
            builder.arc(
                point,
                Vector2D::new(radius_px / screen.x as f32, radius_px / screen.y as f32),
                Angle::two_pi(),
                Angle::zero(),
            );
            builder.close();
            MyPath::Filled(builder.build())
        }
        Geom::Lines { points, width } => {
            debug_assert!(points.len() >= 2);
            builder.move_to(transform_viewport(&points[0], &viewport));
            for point in &points[1..] {
                builder.line_to(transform_viewport(&point, &viewport));
            }
            MyPath::Stroked {
                path: builder.build(),
                width: transform_viewport_1d(width, &viewport),
            }
        }
        Geom::Polygon(points) => {
            debug_assert!(points.len() >= 3);
            builder.move_to(transform_viewport(&points[0], &viewport));
            for point in &points[1..] {
                builder.line_to(transform_viewport(&point, &viewport));
            }
            builder.close();
            MyPath::Filled(builder.build())
        }
    }
}

fn create_vertices(
    styled_geoms: Vec<StyledGeom>,
    screen: Vector2D<usize>,
    viewport: Box2DData,
) -> (Vec<Vertex>, Vec<u32>) {
    // Will contain the result of the tessellation.
    let mut geometry: VertexBuffers<Vertex, u32> = VertexBuffers::new();

    let mut fill_tessellator = FillTessellator::new();
    let mut stroke_tessellator = StrokeTessellator::new();

    let tolerance = 0.0001;
    let fill_options = FillOptions::DEFAULT
        .with_normals(false)
        .with_tolerance(tolerance);
    let stroke_options = StrokeOptions::DEFAULT.with_tolerance(tolerance);

    for styled_geom in styled_geoms.iter() {
        match geom_to_path(styled_geom.geom.clone(), viewport, screen) {
            MyPath::Filled(path) => {
                fill_tessellator
                    .tessellate_path(
                        path.into_iter(),
                        &fill_options,
                        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| Vertex {
                            _pos: [vertex.position.x, vertex.position.y],
                            _color: styled_geom.color,
                        }),
                    )
                    .unwrap();
            }
            MyPath::Stroked { path, width } => {
                stroke_tessellator
                    .tessellate_path(
                        path.into_iter(),
                        &stroke_options.with_line_width(width),
                        &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| Vertex {
                            _pos: [vertex.position.x, vertex.position.y],
                            _color: styled_geom.color,
                        }),
                    )
                    .unwrap();
            }
        }
    }

    info!(
        "{} vertices, {} indices",
        geometry.vertices.len(),
        geometry.indices.len()
    );

    (geometry.vertices, geometry.indices)
}

use self::wgpu::TextureFormat;
use log::info;
use wgpu_glyph::{GlyphBrushBuilder, HorizontalAlign, Layout, Scale, Section, VerticalAlign};

/// Render to a PNG image with the given path
pub fn capture<R: Render>(render: R, viewport: Box2DData, path: std::path::PathBuf) {
    // Number of pixels per side in the rendered image
    let size = 2048u32;

    debug!("Initializing WGPU...");
    let instance = wgpu::Instance::new();

    let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
        power_preference: wgpu::PowerPreference::LowPower,
    });

    let mut device = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    });

    let vs_bytes = Vec::from(include_bytes!("spirv/vert.spirv") as &[u8]);
    let fs_bytes = Vec::from(include_bytes!("spirv/frag.spirv") as &[u8]);

    let vs_module = device.create_shader_module(&vs_bytes);
    let fs_module = device.create_shader_module(&fs_bytes);

    let vertex_size = std::mem::size_of::<Vertex>();

    let (vertex_data, index_data) = create_vertices(
        render.styled_geoms(),
        Vector2D::new(size as usize, size as usize),
        viewport,
    );
    debug!("{} {}", vertex_data.len(), index_data.len());

    let vertex_buf = device
        .create_buffer_mapped(vertex_data.len(), wgpu::BufferUsage::VERTEX)
        .fill_from_slice(&vertex_data);

    let index_buf = device
        .create_buffer_mapped(index_data.len(), wgpu::BufferUsage::INDEX)
        .fill_from_slice(&index_data);

    let bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { bindings: &[] });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::PipelineStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::PipelineStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        },
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8Unorm,
            color_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        index_format: wgpu::IndexFormat::Uint32,
        vertex_buffers: &[wgpu::VertexBufferDescriptor {
            stride: vertex_size as u64,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    format: wgpu::VertexFormat::Float4,
                    offset: 8, // Because this is preceded by two 4-byte floats?
                    shader_location: 1,
                },
            ],
        }],
        sample_count: 1,
    });

    // The output buffer lets us retrieve the data as an array
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: (size * size * std::mem::size_of::<u32>() as u32) as u64,
        usage: wgpu::BufferUsage::MAP_READ,
    });

    let texture_extent = wgpu::Extent3d {
        width: size,
        height: size,
        depth: 1,
    };

    // The render pipeline renders data into this texture
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsage::STORAGE,
    });
    let texture_view = texture.create_default_view();

    // Transform from (-1..1) to pixels
    let transform_window = |location: Point2D<f32>| {
        Point2D::new(
            ((location.x + 1.0) * 0.5) * size as f32,
            ((location.y + 1.0) * 0.5) * size as f32,
        )
    };

    // Prepare glyph_brush
    let font: &[u8] =
        include_bytes!("font/cooper-hewitt-fixed-for-windows-master/CooperHewitt-Semibold.ttf");
    let mut glyph_brush =
        GlyphBrushBuilder::using_font_bytes(font).build(&mut device, TextureFormat::Bgra8Unorm);

    let command_buffer = {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        // Intentionally throw the render pass into a scope so that we drop it early, I think
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &texture_view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::WHITE,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&render_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.set_index_buffer(&index_buf, 0);
            rpass.set_vertex_buffers(&[(&vertex_buf, 0)]);
            rpass.draw_indexed(0..(index_data.len() as u32), 0, 0..1);
        }

        for text in render.texts() {
            glyph_brush.queue(Section {
                text: &text.text,
                screen_position: transform_window(transform_viewport(&text.location, &viewport))
                    .to_tuple(),
                color: [0.0, 0.0, 0.0, 1.0],
                scale: Scale { x: 40.0, y: 40.0 },
                bounds: (size as f32, size as f32),
                layout: Layout::default_single_line()
                    .h_align(HorizontalAlign::Center)
                    .v_align(VerticalAlign::Center),
                ..Section::default()
            });
        }

        // Draw queued texts
        glyph_brush
            .draw_queued(&mut device, &mut encoder, &texture_view, size, size)
            .unwrap();

        // Copy the data from the texture to the buffer
        encoder.copy_texture_to_buffer(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::BufferCopyView {
                buffer: &output_buffer,
                offset: 0,
                row_pitch: std::mem::size_of::<u32>() as u32 * size,
                image_height: size,
            },
            texture_extent,
        );

        encoder
    }
    .finish();

    device.get_queue().submit(&[command_buffer]);

    // Dump the image into a PNG
    output_buffer.map_read_async(
        0,
        (std::mem::size_of::<u32>() as u32 * size * size) as u64,
        move |result: wgpu::BufferMapAsyncResult<&[u8]>| {
            let png_encoder = image::png::PNGEncoder::new(std::fs::File::create(path).unwrap());
            png_encoder
                .encode(&result.unwrap().data, size, size, image::ColorType::RGBA(8))
                .unwrap();
        },
    );
}

#[cfg(test)]
mod tests {
    use crate::graphics::*;

    use crate::graphics;
    use crate::graphics::*;

    use log::*;
    use std::path::PathBuf;

    #[test]
    fn test_fn_grid() {
        let viewport = Box2DData::new(Point2DData::new(-2.0, -2.0), Point2DData::new(2.0, 2.0));
        graphics::capture(
            vec![FnGrid {
                viewport: Some(viewport),
                cell_size: 1.0,
                color_fn: |point: Point2DData| {
                    [0.0, (point.x + 2.0) / 4.0, (point.y + 2.0) / 4.0, 1.0]
                },
                label_fn: |point: Point2DData| format!("{}", point),
            }],
            viewport,
            PathBuf::from("output/fn_grid.png"),
        );
    }

    /// This grid is designed to be too be large to naively render on my graphics card
    #[test]
    #[ignore]
    fn test_fn_grid_many() {
        let viewport = Box2DData::new(Point2DData::new(-1.0, -1.0), Point2DData::new(1.0, 1.0));
        graphics::capture(
            vec![FnGrid {
                viewport: Some(viewport),
                cell_size: 0.0005,
                color_fn: |point: Point2DData| [0.0, point.x, point.y, 1.0],
                label_fn: |point: Point2DData| String::from(""),
            }],
            viewport,
            PathBuf::from("output/fn_grid_many.png"),
        );
    }

    #[test]
    fn test_layers() {
        let viewport = Box2DData::new(Point2DData::new(-2.0, -2.0), Point2DData::new(2.0, 2.0));
        let fn_grid = FnGrid {
            viewport: Some(viewport),
            cell_size: 0.95,
            color_fn: |point: Point2DData| [0.0, (point.x + 2.0) / 4.0, (point.y + 2.0) / 4.0, 1.0],
            label_fn: |point: Point2DData| format!("{:?}", point),
        };

        graphics::capture(
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
            PathBuf::from("output/layers.png"),
        );
    }

    #[test]
    fn test_line_width() {
        // This should show two lines that are exactly as wide as they are long, i.e. squares
        graphics::capture(
            vec![
                StyledGeom {
                    geom: Geom::Lines {
                        points: vec![Point2DData::new(-1.0, -0.5), Point2DData::new(0.0, -0.5)],
                        width: 1.0,
                    },
                    color: [1.0, 0.0, 1.0, 1.0],
                },
                StyledGeom {
                    geom: Geom::Lines {
                        points: vec![Point2DData::new(0.5, 0.0), Point2DData::new(0.5, 1.0)],
                        width: 1.0,
                    },
                    color: [1.0, 0.0, 1.0, 1.0],
                },
            ],
            Box2DData::new(Point2DData::new(-1.0, -1.0), Point2DData::new(1.0, 1.0)),
            PathBuf::from("output/line_width.png"),
        );
    }

    #[test]
    fn test_lines() {
        // This should show a filled circle that fades angularly along the palette gradient
        // This seems to be able to handle 100,000 lines but not 1,000,000
        let n = 1000;
        capture(
            (0..n)
                .map(|i| {
                    let ratio = (i as f32) / (n as f32);
                    let angle = ratio * 2.0 * std::f32::consts::PI;
                    StyledGeom {
                        geom: Geom::Lines {
                            points: vec![
                                Point2DData::new(0.0, 0.0),
                                Point2DData::new(angle.cos(), angle.sin()),
                            ],
                            width: 0.002,
                        },
                        color: scale_temperature(ratio, 16.0),
                    }
                })
                .collect::<Vec<_>>(),
            Box2DData::new(Point2DData::new(-1.0, -1.0), Point2DData::new(1.0, 1.0)),
            PathBuf::from("output/lines.png"),
        );
    }

    #[test]
    fn test_points() {
        // This should render a square that's half the height of the screen, right in the middle of the
        // screen.
        graphics::capture(
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
            PathBuf::from("output/points.png"),
        );
    }

    // This should render a black transparent square that's half the height of the screen, right in
    // the middle of the screen.
    #[test]
    fn test_square() {
        graphics::capture(
            vec![StyledGeom {
                geom: Geom::Polygon(vec![
                    Point2DData::new(0.25, 0.25),
                    Point2DData::new(0.75, 0.25),
                    Point2DData::new(0.75, 0.75),
                    Point2DData::new(0.25, 0.75),
                ]),
                color: [0.0, 0.0, 0.0, 0.5],
            }],
            Box2DData::new(Point2DData::new(0.0, 0.0), Point2DData::new(1.0, 1.0)),
            PathBuf::from("output/square.png"),
        );
    }

    /// This should render some text in the center of the screen
    #[test]
    fn test_text() {
        capture(
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
            PathBuf::from("output/text.png"),
        );
    }

}
