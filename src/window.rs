use glium::{Display, Frame, Surface};
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{SystemTime, Instant, Duration};
use image::{DynamicImage, GenericImageView};
use winit::window::{WindowBuilder, Icon};
use crate::shader::ShaderManager;
use crate::font::FontManager;
use crate::texture::TextureManager;
use crate::render::Canvas;
use winit::event::{Event, WindowEvent, KeyboardInput, MouseButton, MouseScrollDelta, ElementState, StartCause};
use winit::event_loop::{EventLoop, ControlFlow};
use glium::backend::glutin::glutin::ContextBuilder;
use winit::monitor::MonitorHandle;
use std::collections::VecDeque;
use winit::platform::desktop::EventLoopExtDesktop;
#[cfg(windows)]
use winit::platform::windows::EventLoopExtWindows;
#[cfg(not(windows))]
use winit::platform::unix::EventLoopExtUnix;
use winit::dpi::{LogicalSize, PhysicalSize, Position, LogicalPosition};

pub struct Window;

impl Window {
    pub fn show<L, S, T>(size: S, title: T, icon: Option<DynamicImage>,
                         decorated: bool, resizable: bool, top: bool, vsync: bool, listener: &mut L, tps: u32)
        where L: WindowListener, S: Into<(u32, u32)>, T: Into<String> {

        let (window_w, window_h) = size.into();

        let mut event_loop = EventLoop::new_any_thread();
        let mut wb = WindowBuilder::new()
            .with_decorations(decorated)
            .with_title(title)
            .with_resizable(resizable) //Stupid winit warning about Xfce bug
            .with_always_on_top(top)
            .with_visible(false)
            .with_inner_size(LogicalSize::new(window_w, window_h));

        if cfg!(not(windows)) && !resizable {
            wb = wb.with_min_inner_size(LogicalSize::new(window_w, window_h))
                .with_max_inner_size(LogicalSize::new(window_w, window_h));
        }

        if let Some(icon) = icon {
            let (icon_w, icon_h) = icon.dimensions();
            wb = wb.with_window_icon(Some(
                Icon::from_rgba(icon.to_rgba().into_raw(), icon_w, icon_h)
                    .expect("Bad icon")
            ));
        }
        let cb = ContextBuilder::new()
            .with_depth_buffer(24)
            .with_vsync(vsync);

        let display = Display::new(wb, cb, &event_loop)
            .expect("Display creation failed");

        let monitor: MonitorHandle = display.gl_window().window().current_monitor();
        let (monitor_w, monitor_h) = Into::<(f64, f64)>::into(monitor.size());

        let shaders = Rc::new(RefCell::new(ShaderManager::new(&display)));
        let fonts = Rc::new(RefCell::new(FontManager::new(&display)));
        let textures = Rc::new(RefCell::new(TextureManager::new(&display)));

        {
            let gl_window = display.gl_window();
            let window: &winit::window::Window = gl_window.window();
            listener.load_resources(&display, shaders.clone(), fonts.clone(), textures.clone());
            window.set_visible(true);
            window.set_outer_position(LogicalPosition::new(
                monitor_w / 2.0 - window_w as f64 / 2.0,
                monitor_h / 2.0 - window_h as f64 / 2.0
            ));
        }

        let mut events = VecDeque::new();
        let mut next_frame_time = Instant::now();
        let mut last_frame_time = SystemTime::now();
        let mut mouse = (0f32, 0f32);

        listener.on_created(&display);

        event_loop.run_return(move |event: Event<()>, _, control_flow| {
            if listener.is_closed(&display) {
                *control_flow = ControlFlow::Exit;
                return;
            }
            let new_frame = match event {
                Event::NewEvents(cause) => {
                    match cause {
                        StartCause::ResumeTimeReached { .. } | StartCause::Init => true,
                        _ => false
                    }
                },
                other => {
                    if let Some(event) = other.to_static() {
                        events.push_back(event);
                    }
                    false
                }
            };
            if new_frame {
                let frame = display.draw();

                let (w, h) = frame.get_dimensions();

                let elapsed = SystemTime::now().duration_since(last_frame_time).expect("Error calculating frame time");
                let partial_ticks = (elapsed.as_millis() as f64 / tps as f64) as f32;

                listener.on_frame_update(&display, (w as f32, h as f32), mouse, partial_ticks);

                let mut canvas = Canvas::new(
                    display.clone(), shaders.clone(), fonts.clone(), textures.clone(), frame
                );

                listener.on_frame_draw(&mut canvas, mouse, partial_ticks);

                let dimensions = canvas.dimensions();

                canvas.into_inner().finish().expect("Frame finishing failed");

                while let Some(event) = events.pop_front() {
                    match event {
                        Event::WindowEvent { event, .. } => {
                            match event {
                                WindowEvent::CloseRequested =>
                                    listener.on_close_requested(&display, dimensions),
                                WindowEvent::Focused(focused) =>
                                    listener.on_focused(&display, dimensions, focused),
                                WindowEvent::KeyboardInput { input, .. } =>
                                    listener.on_keyboard_key(&display, dimensions, input),
                                WindowEvent::ReceivedCharacter(ch) =>
                                    listener.on_keyboard_char(&display, dimensions, ch),
                                WindowEvent::MouseInput { state: e_state, button, .. } =>
                                    listener.on_mouse_button(&display, dimensions, button, e_state, mouse),
                                WindowEvent::MouseWheel { delta, .. } =>
                                    listener.on_mouse_wheel(&display, dimensions, delta),
                                WindowEvent::CursorMoved { position, .. } => {
                                    let (mouse_x, mouse_y): (f64, f64) = position.into();
                                    mouse = (mouse_x as f32, mouse_y as f32);
                                    listener.on_mouse_move(&display, dimensions, mouse);
                                }
                                _ => ()
                            }
                        },
                        _ => ()
                    }
                }

                last_frame_time = SystemTime::now();
                next_frame_time = Instant::now() + Duration::from_secs_f32(1.0 / 60.0);
            }
            *control_flow = ControlFlow::WaitUntil(next_frame_time);
        });
    }
}

pub trait WindowListener {
    fn is_closed(&self, display: &Display) -> bool;
    fn load_resources(&self, display: &Display, shaders: Rc<RefCell<ShaderManager>>, fonts: Rc<RefCell<FontManager>>, textures: Rc<RefCell<TextureManager>>) {}
    fn on_created(&mut self, display: &Display) {}
    fn on_frame_update(&mut self, display: &Display, dimensions: (f32, f32), mouse: (f32, f32), partial_ticks: f32) {}
    fn on_frame_draw(&self, canvas: &mut Canvas<Frame>, mouse_pos: (f32, f32), partial_ticks: f32);
    fn on_close_requested(&mut self, display: &Display, dimensions: (f32, f32)) {}
    fn on_focused(&mut self, display: &Display, dimensions: (f32, f32), focused: bool) {}
    fn on_keyboard_char(&mut self, display: &Display, dimensions: (f32, f32), ch: char) {}
    fn on_keyboard_key(&mut self, display: &Display, dimensions: (f32, f32), input: KeyboardInput) {}
    fn on_mouse_button(&mut self, display: &Display, dimensions: (f32, f32), button: MouseButton, state: ElementState, pos: (f32, f32)) {}
    fn on_mouse_wheel(&mut self, display: &Display, dimensions: (f32, f32), delta: MouseScrollDelta) {}
    fn on_mouse_move(&mut self, display: &Display, dimensions: (f32, f32), pos: (f32, f32)) {}
}