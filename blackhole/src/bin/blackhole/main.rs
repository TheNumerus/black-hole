use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::{Arc, RwLock};

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

use blackhole::framebuffer::FrameBuffer;

use gl_wrapper::geometry::{GeometryBuilder, VertexAttribute};
use gl_wrapper::program::ProgramBuilder;
use gl_wrapper::renderer::GlRenderer;
use gl_wrapper::texture::{Texture2D, TextureFilter, TextureFormats};
use gl_wrapper::QUAD;

mod args;
mod renderer;
mod scene_loader;
mod shaders;

use crate::args::ArgsInteractive;
use crate::renderer::{RenderInMsg, RenderOutMsg, Scaling};
use renderer::{RenderMode, Renderer};
use scene_loader::SceneLoader;

fn main() {
    // clion needs help in trait annotation
    let args = <ArgsInteractive as Parser>::parse();

    let fb = FrameBuffer::default();

    let loader = SceneLoader::new();

    let mut renderer = Renderer {
        mode: args.mode,
        samples: args.samples,
        threads: args.threads,
        scaling: Scaling::X1,
        ..Default::default()
    };

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_inner_size(Size::Physical(PhysicalSize::new(1280, 720)))
        .with_min_inner_size(Size::Physical(PhysicalSize::new(32, 32)))
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

    let program_copy = ProgramBuilder::new(
        include_str!("gl_shaders/quad.glsl"),
        include_str!("gl_shaders/copy.glsl"),
    )
    .build()
    .unwrap();

    let (tx_in, rx_in) = std::sync::mpsc::channel();
    let (tx_out, rx_out) = std::sync::mpsc::channel();

    let fb = Arc::new(RwLock::new(fb));
    let fb_clone = Arc::clone(&fb);

    let mut render_thread = Some(std::thread::spawn(move || {
        renderer.render_interactive(fb_clone, tx_out, rx_in);
    }));

    tx_in.send(RenderInMsg::Restart).unwrap();

    let texture = {
        let read_lock = fb.read().unwrap();

        Texture2D::new(
            1280,
            720,
            unsafe { read_lock.as_f32_slice() },
            TextureFormats::RgbaF32,
            TextureFilter::Nearest,
        )
        .unwrap()
    };

    let texture_fb = Texture2D::new(
        1280,
        720,
        &[0.0; 1280 * 720 * 4],
        TextureFormats::RgbaF32,
        TextureFilter::Linear,
    )
    .unwrap();

    let gl_fb = gl_wrapper::framebuffer::FrameBuffer::from_texture(&texture_fb).unwrap();

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
                        texture_fb
                            .update(
                                size.width,
                                size.height,
                                &vec![0.0; (size.width * size.height * 4) as usize],
                                TextureFormats::RgbaF32,
                            )
                            .unwrap();
                        tx_in
                            .send(RenderInMsg::Resize(size.width, size.height))
                            .unwrap();
                    }
                }
                WindowEvent::DroppedFile(path) => {
                    let scene_res = loader.load_path(&path);

                    let scene = match scene_res {
                        Ok(v) => {
                            eprintln!("Read scene file from {:?}", path);
                            v
                        }
                        Err(e) => {
                            eprintln!("Could not read scene description: {e}");
                            return;
                        }
                    };

                    tx_in.send(RenderInMsg::SceneChange(scene)).unwrap();
                }
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                    tx_in.send(RenderInMsg::Exit).unwrap();
                    render_thread.take().unwrap().join().unwrap();
                }
                _ => (),
            },
            Event::RedrawRequested(_) => unsafe {
                gl_fb.bind();

                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                texture.bind(0);
                gl_renderer.draw(&quad, &program_copy);

                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                texture_fb.bind(0);
                gl_renderer.draw(&quad, &program);
            },
            _ => (),
        }
    });
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