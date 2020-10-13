use glow::HasContext;
use glutin::{
    dpi::LogicalSize,
    event::{ElementState, Event, WindowEvent},
    event_loop::ControlFlow,
    event_loop::EventLoop,
};
use std::rc::Rc;

pub mod graphics;
pub mod keyboard;
pub mod mouse;
pub mod timer;
pub mod window;

/// The types of errors that can occur when initializing the context.
#[derive(Debug)]
pub enum InitError {
    WindowCreation(glutin::CreationError),
    Context(glutin::ContextError),
}

impl From<glutin::CreationError> for InitError {
    fn from(err: glutin::CreationError) -> Self {
        InitError::WindowCreation(err)
    }
}
impl From<glutin::ContextError> for InitError {
    fn from(err: glutin::ContextError) -> Self {
        InitError::Context(err)
    }
}

/// The entry point of the `ds2d` library that sets up the various subsystems,
/// and in particular a window with a graphics context.
pub struct ContextBuilder {
    title: String,
    size: LogicalSize<f64>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            size: LogicalSize {
                width: 800.0,
                height: 600.0,
            },
        }
    }

    /// Set the title of the game window.
    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = title.into();
        self
    }

    /// Set the logical size of the game window (before being scaled by the DPI factor).
    /// TODO: is there a way of providing a physical size for initialization?
    pub fn logical_size(mut self, size: LogicalSize<f64>) -> Self {
        self.size = size;
        self
    }

    /// Create a window with an OpenGL context, and the corresponding event loop.
    /// The returned `ds2d::Context` can be used for initializing the Game state
    /// before starting the game loop.
    pub fn build(self) -> Result<(EventLoop<()>, Context), InitError> {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title(self.title)
            .with_resizable(true)
            .with_inner_size(self.size);
        let windowed_context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)?;
        // The window is dropped in case of an error
        let windowed_context = unsafe { windowed_context.make_current().map_err(|(_, err)| err)? };

        Ok((event_loop, Context::new(windowed_context)))
    }
}

/// Run the game and never return.
pub fn run(event_loop: EventLoop<()>, mut context: Context) -> ! {
    event_loop.run(move |event, _target, control_flow| {
        context.handle_event(event, control_flow);
    })
}

/// A collection of the various systems of the game engine.
/// This will be passed to each call into the actual game.
#[derive(Debug)]
pub struct Context {
    pub(crate) keyboard: keyboard::KeyboardContext,
    pub(crate) mouse: mouse::MouseContext,
    pub(crate) timer: timer::TimerContext,
    pub(crate) window: window::WindowContext,
    pub(crate) graphics: graphics::GraphicsContext,
}

impl Context {
    pub(crate) fn new(windowed_context: glutin::WindowedContext<glutin::PossiblyCurrent>) -> Self {
        let window = Rc::new(windowed_context);
        Self {
            timer: timer::TimerContext::new(),
            mouse: mouse::MouseContext::default(),
            keyboard: keyboard::KeyboardContext::default(),
            window: window::WindowContext::new(window.clone()),
            graphics: graphics::GraphicsContext::new(window),
        }
    }

    pub(crate) fn handle_event(
        &mut self,
        event: glutin::event::Event<()>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            Event::NewEvents(_) => {}
            Event::WindowEvent {
                window_id: _,
                event,
            } => {
                match event {
                    WindowEvent::CloseRequested => {
                        // TODO: let the game handle close event
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    WindowEvent::Resized(new_size) => self.window.windowed_context.resize(new_size),
                    WindowEvent::ScaleFactorChanged { .. } => {
                        // TODO: what to do with changing DPI?
                    }
                    WindowEvent::Destroyed => {}
                    WindowEvent::ReceivedCharacter(ch) => {
                        self.keyboard.unicode_text.push(ch);
                    }
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        input,
                        is_synthetic: _,
                    } => {
                        // TODO: preferibly use scan codes, but how to find those?
                        if let Some(vk) = input.virtual_keycode {
                            if input.state == ElementState::Pressed {
                                self.keyboard.pressed_keys.insert(vk);
                            } else {
                                self.keyboard.pressed_keys.remove(&vk);
                            }
                        }
                    }
                    WindowEvent::ModifiersChanged(_) => {
                        // TODO: is ModifiersChanged needed?
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.mouse.position = position;
                    }
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        glutin::event::MouseScrollDelta::LineDelta(dx, dy) => {
                            self.mouse.scroll_x += dx;
                            self.mouse.scroll_y += dy;
                        }
                        glutin::event::MouseScrollDelta::PixelDelta(_) => {
                            // TODO: is this also needed? my mouse at home only emits LineDelta
                        }
                    },
                    WindowEvent::MouseInput { state, button, .. } => {
                        if state == ElementState::Pressed {
                            self.mouse.pressed_buttons.insert(button);
                        } else {
                            self.mouse.pressed_buttons.remove(&button);
                        }
                    }
                    WindowEvent::Touch(_) => {
                        // TODO: support Touch eventually
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent { .. } => {
                // TODO: might only need Added and Removed events to detect controller changes
            }
            Event::UserEvent(_) => {}
            Event::Suspended => {}
            Event::Resumed => {}
            Event::MainEventsCleared => {
                self.timer.tick();
                // TODO: call update() here

                // Clear transient event state
                self.keyboard.unicode_text.clear();
                self.mouse.scroll_x = 0.0;
                self.mouse.scroll_y = 0.0;

                // Keep the animation running
                self.window.windowed_context.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                unsafe {
                    self.graphics
                        .gl
                        .clear_color(100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0);
                    self.graphics.gl.clear(glow::COLOR_BUFFER_BIT);
                }
                // TODO: call draw() here
                self.window.windowed_context.swap_buffers().unwrap();
            }
            Event::RedrawEventsCleared => {
                *control_flow = glutin::event_loop::ControlFlow::Poll;
            }
            Event::LoopDestroyed => {}
        }
    }
}
