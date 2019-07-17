use log::*;

extern crate env_logger;
extern crate wgpu;

use either::Either;
use euclid::{Point2D, TypedPoint2D};
use lyon::tessellation::*;

use crate::graphics;
use shaderc;

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

pub trait Render {
    fn styled_geoms(&self) -> Vec<StyledGeom>;

    fn texts(&self) -> Vec<Text>;
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

/// Cells are implicitly based around the origin
pub struct FnGrid<F, G> {
    pub viewport: Box2DData,

    /// The side length of a cell, in data space
    pub cell_size: f32,

    /// Given the center of a grid cell, return the color to paint this grid cell
    pub color_fn: F,

    /// Given the center of a grid cell, return the label of this grid cell
    pub label_fn: G,
}

impl<F: Fn(Point2DData) -> [f32; 4], G: Fn(Point2DData) -> String> Render for FnGrid<F, G> {
    fn styled_geoms(&self) -> Vec<StyledGeom> {
        let min_x = (self.viewport.min.x / self.cell_size).floor() as isize;
        let min_y = (self.viewport.min.y / self.cell_size).floor() as isize;
        let max_x = (self.viewport.max.x / self.cell_size).floor() as isize;
        let max_y = (self.viewport.max.y / self.cell_size).floor() as isize;

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
        let min_x = (self.viewport.min.x / self.cell_size).floor() as isize;
        let min_y = (self.viewport.min.y / self.cell_size).floor() as isize;
        let max_x = (self.viewport.max.x / self.cell_size).floor() as isize;
        let max_y = (self.viewport.max.y / self.cell_size).floor() as isize;

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

#[allow(dead_code)]
pub fn cast_slice<T>(data: &[T]) -> &[u8] {
    use std::mem::size_of;
    use std::slice::from_raw_parts;

    unsafe { from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<T>()) }
}

pub fn glsl_to_spirv(name: &str, source: &str, kind: shaderc::ShaderKind) -> Vec<u8> {
    let mut compiler = shaderc::Compiler::new().unwrap();
    Vec::from(
        compiler
            .compile_into_spirv(source, kind, name, "main", None)
            .unwrap()
            .as_binary_u8(),
    )
}

pub trait Example {
    fn init(sc_desc: &wgpu::SwapChainDescriptor, device: &mut wgpu::Device) -> Self;
    fn resize(&mut self, sc_desc: &wgpu::SwapChainDescriptor, device: &mut wgpu::Device);
    fn update(&mut self, event: wgpu::winit::WindowEvent);
    fn render(&mut self, frame: &wgpu::SwapChainOutput, device: &mut wgpu::Device);
}

pub fn leggo<R: Render>(render: R, viewport: Box2DData) {
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

    debug!("building shaders...");
    let vs_bytes = graphics::glsl_to_spirv(
        "graphics.vert",
        include_str!("shader/graphics.vert"),
        shaderc::ShaderKind::Vertex,
    );
    let fs_bytes = graphics::glsl_to_spirv(
        "graphics.frag",
        include_str!("shader/graphics.frag"),
        shaderc::ShaderKind::Fragment,
    );
    let vs_module = device.create_shader_module(&vs_bytes);
    let fs_module = device.create_shader_module(&fs_bytes);

    let vertex_size = std::mem::size_of::<Vertex>();

    // Ways to get dimensions:
    // - from actual window size when previewing
    // - from an intended px dimension
    // - from intended real world dimension + DPI
    let screen = Vector2D::new(2880, 1800);

    let (vertex_data, index_data) = create_vertices(render.styled_geoms(), screen, viewport);
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
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
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

    use wgpu::winit::{
        ControlFlow, ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window,
        WindowEvent,
    };

    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();
    window.set_fullscreen(Some(window.get_current_monitor()));
    let size = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    // The vertex shader requires this
    assert_eq!(size.width as f32 / size.height as f32, 1.6);

    // Transform from (-1..1) to pixels
    let transform_window = |location: Point2D<f32>| {
        Point2D::new(
            ((location.x + 1.0) * 0.5) * size.height as f32
                + (size.width - size.height) as f32 * 0.5,
            ((location.y + 1.0) * 0.5) * size.height as f32,
        )
    };

    let surface = instance.create_surface(&window);
    let mut swap_chain = device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: (size.width.round() as u32) * 4,
            height: (size.height.round() as u32) * 4,
        },
    );

    // Prepare glyph_brush
    let inconsolata: &[u8] = include_bytes!("font/Inconsolata-Regular.ttf");
    let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(inconsolata)
        .build(&mut device, TextureFormat::Bgra8Unorm);

    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(code),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match code {
                    VirtualKeyCode::Escape => return ControlFlow::Break,
                    _ => {}
                },
                WindowEvent::CloseRequested => return ControlFlow::Break,
                _ => {}
            },
            _ => {}
        }

        let frame = swap_chain.get_next_texture();
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
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
                bounds: (size.width as f32, size.height as f32),
                layout: Layout::default_single_line()
                    .h_align(HorizontalAlign::Center)
                    .v_align(VerticalAlign::Center),
                ..Section::default()
            });
        }

        // Draw queued texts
        glyph_brush
            .draw_queued(
                &mut device,
                &mut encoder,
                &frame.view,
                size.width.round() as u32,
                size.height.round() as u32,
            )
            .unwrap();

        device.get_queue().submit(&[encoder.finish()]);

        ControlFlow::Continue
    });
}
