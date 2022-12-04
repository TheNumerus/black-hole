use std::ffi::CString;
use std::fs::File;
use std::io::BufWriter;
use std::num::NonZeroU32;
use std::sync::{Arc, RwLock};

use cgmath::{InnerSpace, Vector3};

use clap::Parser;
use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use blackhole::filter::BlackmanHarrisFilter;
use blackhole::framebuffer::{FrameBuffer, Pixel};

mod args;
mod gl_wrapper;
mod renderer;
mod scene_loader;
mod shaders;

use crate::gl_wrapper::geometry::{GeometryBuilder, VertexAttribute};
use crate::gl_wrapper::program::ProgramBuilder;
use crate::gl_wrapper::renderer::GlRenderer;
use crate::gl_wrapper::texture::{Texture2D, TextureFormats};
use crate::gl_wrapper::QUAD;
use crate::renderer::{RenderInMsg, RenderOutMsg};
use args::Args;
use blackhole::frame::{Frame, Region};
use renderer::{RenderMode, Renderer};
use scene_loader::SceneLoader;

fn main() {
    // clion needs help in trait annotation
    let args = <Args as Parser>::parse();

    let fb = FrameBuffer::new(args.width, args.height);

    let loader = SceneLoader::new();

    let scene = loader.load_path(args.scene);

    let scene = match scene {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Could not read scene description: {e}");
            std::process::exit(-1);
        }
    };

    /*let region = Region::Window {
        x_min: 499,
        y_min: 700,
        x_max: 500,
        y_max: 701,
    };
    let region = Region::Window {
        x_min: 450,
        y_min: 650,
        x_max: 550,
        y_max: 750,
    };*/
    let region = Region::Whole;

    let frame = Frame {
        width: args.width,
        height: args.height,
        region,
    };

    let mut renderer = Renderer {
        mode: args.mode,
        samples: args.samples,
        threads: args.threads,
        max_steps: 2 << 16,
        max_depth: 16,
        filter: Box::new(BlackmanHarrisFilter::new(1.5)),
        frame,
        ..Default::default()
    };

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_inner_size(Size::Physical(PhysicalSize::new(1280, 720)))
        .with_title("Black-hole renderer");
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));
    let template = ConfigTemplateBuilder::new();

    let (window, gl_config) = display_builder
        .build(&event_loop, template, |mut configs| configs.next().unwrap())
        .unwrap();

    let handle = window.as_ref().map(|w| w.raw_window_handle());
    let gl_display = gl_config.display();

    let context_attr = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 5))))
        .build(handle);

    let gl_window = GlWindow::new(window.unwrap(), &gl_config);

    let gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attr)
            .unwrap()
    })
    .take()
    .unwrap()
    .make_current(&gl_window.surface)
    .unwrap();

    gl::load_with(|s| {
        gl_display
            .get_proc_address(CString::new(s).unwrap().as_c_str())
            .cast()
    });

    let quad = GeometryBuilder::new(&QUAD)
        .with_attribute(VertexAttribute::Vec2)
        .build()
        .unwrap();
    let program = ProgramBuilder::new(
        include_str!("gl_shaders/quad.glsl"),
        include_str!("gl_shaders/output.glsl"),
    )
    .build()
    .unwrap();

    let (tx_in, rx_in) = std::sync::mpsc::channel();
    let (tx_out, rx_out) = std::sync::mpsc::channel();

    let fb = Arc::new(RwLock::new(fb));
    let fb_clone = Arc::clone(&fb);

    let mut render_thread = Some(std::thread::spawn(move || {
        renderer.render_interactive(&scene, fb_clone, tx_out, rx_in);
    }));

    tx_in.send(RenderInMsg::Restart).unwrap();

    let texture = {
        let read_lock = fb.read().unwrap();

        Texture2D::new(
            args.width as u32,
            args.height as u32,
            unsafe { read_lock.as_f32_slice() },
            TextureFormats::RgbaF32,
        )
        .unwrap()
    };

    let mut gl_renderer = GlRenderer::new();

    event_loop.run(move |event, _window_target, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawEventsCleared => {
                if let Some(msg) = rx_out.try_iter().next() {
                    match msg {
                        RenderOutMsg::Update => {
                            let read_lock = fb.read().unwrap();

                            texture
                                .update(
                                    read_lock.width() as u32,
                                    read_lock.height() as u32,
                                    unsafe { read_lock.as_f32_slice() },
                                    TextureFormats::RgbaF32,
                                )
                                .unwrap();
                        }
                    }
                }

                gl_window.window.request_redraw();
                gl_window.surface.swap_buffers(&gl_context).unwrap();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        gl_window.surface.resize(
                            &gl_context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                        gl_renderer.resize(size.width, size.height);
                        tx_in
                            .send(RenderInMsg::Resize(size.width, size.height))
                            .unwrap();
                    }
                }
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                    tx_in.send(RenderInMsg::Exit).unwrap();
                    render_thread.take().unwrap().join().unwrap();
                }
                _ => (),
            },
            Event::RedrawRequested(_) => unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                texture.bind(0);
                gl_renderer.draw(&quad, &program);
            },
            _ => (),
        }
    });

    /*renderer.render(&scene, &mut fb, |src| {});

    post_process(&mut fb, &args.mode);

    write_out(fb, args.width as u32, args.height as u32);*/
}

fn post_process(fb: &mut FrameBuffer, mode: &RenderMode) {
    let luminance_base = Vector3::new(0.2126, 0.7152, 0.0722);

    match mode {
        RenderMode::Shaded => {
            for pixel in fb.buffer_mut() {
                let luminance = Vector3::new(pixel.r, pixel.g, pixel.b).dot(luminance_base);

                let new_luminance = luminance / (luminance + 1.0);

                let tonemapped = Pixel::new(
                    pixel.r * (new_luminance / luminance),
                    pixel.g * (new_luminance / luminance),
                    pixel.b * (new_luminance / luminance),
                    pixel.a,
                );

                let new_pixel = Pixel::new(
                    tonemapped.r.powf(1.0 / 2.2),
                    tonemapped.g.powf(1.0 / 2.2),
                    tonemapped.b.powf(1.0 / 2.2),
                    pixel.a,
                );

                *pixel = new_pixel;
            }
        }
        RenderMode::Samples | RenderMode::Normal => {}
    }
}

fn write_out(fb: FrameBuffer, width: u32, height: u32) {
    let buf = unsafe {
        assert_eq!(std::mem::size_of::<Pixel>(), 4 * std::mem::size_of::<f32>());

        fb.as_f32_slice()
    };

    let mapped = buf.iter().map(|e| (e * 255.0) as u8).collect::<Vec<_>>();

    let file = File::create("out.png").unwrap();
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&mapped).unwrap();
}

pub struct GlWindow {
    // XXX the surface must be dropped before the window.
    pub surface: Surface<WindowSurface>,

    pub window: Window,
}

impl GlWindow {
    pub fn new(window: Window, config: &Config) -> Self {
        let (width, height): (u32, u32) = window.inner_size().into();
        let raw_window_handle = window.raw_window_handle();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        let surface = unsafe {
            config
                .display()
                .create_window_surface(config, &attrs)
                .unwrap()
        };

        Self { window, surface }
    }
}
