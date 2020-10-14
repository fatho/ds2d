use glutin::{
    dpi::LogicalSize,
    event::{ElementState, Event, WindowEvent},
    event_loop::ControlFlow,
    event_loop::EventLoop,
};
use log::error;
use std::rc::Rc;

use crate::graphics::Color;

pub(crate) mod graphics;
pub(crate) mod keyboard;
pub(crate) mod mouse;
pub(crate) mod timer;

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
    debug: bool,
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            size: LogicalSize {
                width: 800.0,
                height: 600.0,
            },
            debug: cfg!(debug_assertions),
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

    /// Enable additional debug checks and output.
    /// Defaults to `cfg!(debug_assertions)`.
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
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
            .with_gl_debug_flag(self.debug)
            .build_windowed(window_builder, &event_loop)?;
        // The window is dropped in case of an error
        let windowed_context = unsafe { windowed_context.make_current().map_err(|(_, err)| err)? };
        let mut context = Context::new(windowed_context);
        if self.debug {
            context.graphics.init_debug();
        }

        Ok((event_loop, context))
    }
}

/// Run the game and never return.
pub fn run(
    event_loop: EventLoop<()>,
    mut context: Context,
    mut game: impl super::Game + 'static,
) -> ! {
    event_loop.run(move |event, _target, control_flow| {
        context.handle_event(event, control_flow, &mut game);
    })
}

/// A collection of the various systems of the game engine.
/// This will be passed to each call into the actual game.
#[derive(Debug)]
pub struct Context {
    pub(crate) keyboard: keyboard::KeyboardContext,
    pub(crate) mouse: mouse::MouseContext,
    pub(crate) timer: timer::TimerContext,
    pub(crate) graphics: graphics::GraphicsContext,
}

impl Context {
    pub(crate) fn new(windowed_context: glutin::WindowedContext<glutin::PossiblyCurrent>) -> Self {
        let window = Rc::new(windowed_context);
        Self {
            timer: timer::TimerContext::new(),
            mouse: mouse::MouseContext::default(),
            keyboard: keyboard::KeyboardContext::default(),
            graphics: graphics::GraphicsContext::new(window),
        }
    }

    pub(crate) fn handle_event(
        &mut self,
        event: glutin::event::Event<()>,
        control_flow: &mut ControlFlow,
        game: &mut impl super::Game,
    ) {
        match event {
            Event::NewEvents(_) => {}
            Event::WindowEvent {
                window_id: _,
                event,
            } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(new_size) => {
                        self.graphics.windowed_context.resize(new_size);
                        self.graphics.screen_size = new_size;
                        log::debug!("Window resized: {:?}", new_size);
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        self.graphics.scale_factor = scale_factor;
                        log::debug!("Window scale factor changed: {:?}", scale_factor);
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
                if let Err(err) = game.update(self) {
                    error!("Game::update failed: {}", err);
                    *control_flow = ControlFlow::Exit;
                }

                // Clear transient event state
                self.keyboard.unicode_text.clear();
                self.mouse.scroll_x = 0.0;
                self.mouse.scroll_y = 0.0;

                // Keep the animation running
                self.graphics.windowed_context.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Clear the screen in a hideous magenta so that its clear if the Game forgot to clear it
                crate::graphics::clear(self, Color::MAGENTA);
                if let Err(err) = game.draw(self) {
                    error!("Game::draw failed: {}", err);
                    *control_flow = ControlFlow::Exit;
                }
                self.graphics.windowed_context.swap_buffers().unwrap();
            }
            Event::RedrawEventsCleared => {
                *control_flow = ControlFlow::Poll;
            }
            Event::LoopDestroyed => {
                if let Err(err) = game.exit(self) {
                    error!(
                        "Game::exit failed, any state might not have been persisted: {}",
                        err
                    );
                    // Can't do much more here, let's hope for the best
                }
            }
        }
    }
}
