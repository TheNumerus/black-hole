use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor,
    PossiblyCurrentContext, Version,
};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface};

use glutin_winit::DisplayBuilder;

use raw_window_handle::HasRawWindowHandle;

use flume::{Receiver, Sender};

use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

use cgmath::{Deg, InnerSpace, Matrix3};

use thiserror::Error;

use winit::dpi::{PhysicalPosition, PhysicalSize, Size};
use winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use blackhole::framebuffer::FrameBuffer;
use blackhole::scene::Scene;

use blackhole_common::scene_loader::SceneLoader;

use gl_wrapper::geometry::{GeometryBuilder, VertexAttribute};
use gl_wrapper::program::ProgramBuilder;
use gl_wrapper::renderer::GlRenderer;
use gl_wrapper::texture::{Texture2D, TextureFilter, TextureFormats};
use gl_wrapper::QUAD;

use crate::renderer::{InteractiveRenderer, RenderInMsg, RenderOutMsg};

pub struct App {
    event_loop: EventLoop<()>,
    render_thread: Option<JoinHandle<()>>,
    gl_context: PossiblyCurrentContext,
    gl_window: GlWindow,
    scene_loader: SceneLoader,
    tx_in: Sender<RenderInMsg>,
    rx_out: Receiver<RenderOutMsg>,
    cpu_framebuffer: Arc<RwLock<FrameBuffer>>,
}

impl App {
    pub fn new(
        mut renderer: InteractiveRenderer,
        scene_loader: SceneLoader,
    ) -> Result<Self, AppError> {
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

        let (tx_in, rx_in) = flume::unbounded();
        let (tx_out, rx_out) = flume::unbounded();

        let cpu_framebuffer = Arc::new(RwLock::new(FrameBuffer::default()));
        let fb_clone = Arc::clone(&cpu_framebuffer);

        let render_thread = Some(std::thread::spawn(move || {
            renderer.render(fb_clone, tx_out, rx_in);
        }));

        let app = Self {
            event_loop,
            render_thread,
            gl_context,
            gl_window,
            scene_loader,
            tx_in,
            rx_out,
            cpu_framebuffer,
        };

        Ok(app)
    }

    pub fn run(mut self) -> ! {
        self.tx_in.send(RenderInMsg::Restart).unwrap();

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

        self.tx_in.send(RenderInMsg::Restart).unwrap();

        let texture = {
            let read_lock = self.cpu_framebuffer.read().unwrap();

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

        let mut last_pos = PhysicalPosition::new(0.0, 0.0);
        let mut lmb_pressed = false;
        let mut rmb_pressed = false;
        let mut scene: Option<Scene> = None;

        let mut keys = ActiveKeys::default();

        self.event_loop
            .run(move |event, _window_target, control_flow| {
                *control_flow = ControlFlow::Wait;
                match event {
                    Event::RedrawEventsCleared => {
                        if let Some(msg) = self.rx_out.try_iter().next() {
                            match msg {
                                RenderOutMsg::Update(scale) => {
                                    let read_lock = self.cpu_framebuffer.read().unwrap();

                                    let (w, h) = (
                                        read_lock.width() as u32 / scale.scale(),
                                        read_lock.height() as u32 / scale.scale(),
                                    );

                                    texture
                                        .update(
                                            w,
                                            h,
                                            unsafe { read_lock.as_f32_slice() },
                                            TextureFormats::RgbaF32,
                                        )
                                        .unwrap();
                                }
                            }
                        }

                        if let Some(scene) = &mut scene {
                            let camera_delta = {
                                let mut x = 0.0;
                                let mut y = 0.0;
                                let mut z = 0.0;
                                if keys.a {
                                    x -= 1.0;
                                }

                                if keys.d {
                                    x += 1.0;
                                }

                                if keys.w {
                                    y += 1.0;
                                }

                                if keys.s {
                                    y -= 1.0;
                                }

                                if keys.q {
                                    z -= 1.0;
                                }

                                if keys.e {
                                    z += 1.0;
                                }

                                scene.camera.side() * (x / 50.0)
                                    + scene.camera.forward() * (y / 50.0)
                                    + scene.camera.up() * (z / 50.0)
                            };

                            scene.camera.location += camera_delta;

                            if camera_delta.magnitude2() != 0.0 {
                                self.tx_in
                                    .send(RenderInMsg::SceneChange(scene.clone()))
                                    .unwrap();
                            }
                        }

                        self.gl_window.window.request_redraw();
                        self.gl_window
                            .surface
                            .swap_buffers(&self.gl_context)
                            .unwrap();
                    }
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::Resized(size) => {
                            if size.width != 0 && size.height != 0 {
                                self.gl_window.surface.resize(
                                    &self.gl_context,
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
                                self.tx_in
                                    .send(RenderInMsg::Resize(size.width, size.height))
                                    .unwrap();
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let delta = (last_pos.x - position.x, last_pos.y - position.y);

                            if let Some(scene) = &mut scene {
                                if rmb_pressed {
                                    let rot = Matrix3::from_angle_y(Deg(delta.0 / 10.0))
                                        * Matrix3::from_axis_angle(
                                            scene.camera.side(),
                                            Deg(delta.1 / 10.0),
                                        );

                                    scene.camera.rot_mat = rot * scene.camera.rot_mat;
                                    self.tx_in
                                        .send(RenderInMsg::SceneChange(scene.clone()))
                                        .unwrap();
                                }
                            }

                            last_pos = position;
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            if let MouseButton::Left = button {
                                lmb_pressed = state == ElementState::Pressed
                            }
                            if let MouseButton::Right = button {
                                rmb_pressed = state == ElementState::Pressed
                            }
                        }
                        WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                            Some(VirtualKeyCode::W) => {
                                keys.w = input.state == ElementState::Pressed
                            }
                            Some(VirtualKeyCode::A) => {
                                keys.a = input.state == ElementState::Pressed
                            }
                            Some(VirtualKeyCode::S) => {
                                keys.s = input.state == ElementState::Pressed
                            }
                            Some(VirtualKeyCode::D) => {
                                keys.d = input.state == ElementState::Pressed
                            }
                            Some(VirtualKeyCode::Q) => {
                                keys.q = input.state == ElementState::Pressed
                            }
                            Some(VirtualKeyCode::E) => {
                                keys.e = input.state == ElementState::Pressed
                            }
                            _ => {}
                        },
                        WindowEvent::DroppedFile(path) => {
                            let scene_res = self.scene_loader.load_path(&path);

                            scene = match scene_res {
                                Ok(s) => {
                                    eprintln!("Read scene file from {:?}", path);
                                    self.tx_in
                                        .send(RenderInMsg::SceneChange(s.clone()))
                                        .unwrap();
                                    Some(s)
                                }
                                Err(e) => {
                                    eprintln!("Could not read scene description: {e}");
                                    return;
                                }
                            };
                        }
                        WindowEvent::CloseRequested => {
                            control_flow.set_exit();
                            self.tx_in.send(RenderInMsg::Exit).unwrap();
                            self.render_thread.take().unwrap().join().unwrap();
                        }
                        _ => (),
                    },
                    Event::RedrawRequested(_) => {
                        gl_fb.bind();

                        gl_renderer.clear_color(0.0, 0.0, 0.0);

                        texture.bind(0);
                        gl_renderer.draw(&quad, &program_copy);

                        gl_wrapper::framebuffer::FrameBuffer::bind_default();

                        gl_renderer.clear_color(0.0, 0.0, 0.0);

                        texture_fb.bind(0);
                        gl_renderer.draw(&quad, &program);
                    }
                    _ => (),
                }
            })
    }
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

#[derive(Debug, Error)]
pub enum AppError {}

pub struct ActiveKeys {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    q: bool,
    e: bool,
}

impl Default for ActiveKeys {
    fn default() -> Self {
        Self {
            w: false,
            a: false,
            s: false,
            d: false,
            q: false,
            e: false,
        }
    }
}
