use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use thiserror::Error;
use tracing::info;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

/// Configuration required to create the application's native window.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowConfig {
    title: String,
}

impl WindowConfig {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
        }
    }
}

/// An owned, cloneable handle to the native application window.
#[derive(Clone, Debug)]
pub struct NativeWindow(Arc<Window>, Arc<AtomicBool>);

impl NativeWindow {
    pub fn request_redraw(&self) {
        self.0.request_redraw();
    }

    pub fn size(&self) -> WindowSize {
        let size = self.0.inner_size();
        WindowSize {
            width: size.width,
            height: size.height,
        }
    }

    pub fn set_cursor_captured(&self, captured: bool) -> Result<(), PlatformError> {
        let result = if captured {
            self.0
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| self.0.set_cursor_grab(CursorGrabMode::Locked))
        } else {
            self.0.set_cursor_grab(CursorGrabMode::None)
        };
        result.map_err(|error| PlatformError::CursorGrab(error.to_string()))?;
        self.1.store(captured, Ordering::Release);
        self.0.set_cursor_visible(!captured);
        info!(captured, "cursor capture changed");
        Ok(())
    }

    pub fn cursor_captured(&self) -> bool {
        self.1.load(Ordering::Acquire)
    }
}

impl HasWindowHandle for NativeWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.0.window_handle()
    }
}

impl HasDisplayHandle for NativeWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        self.0.display_handle()
    }
}

/// Drawable window dimensions in physical pixels.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    pub fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Engine-owned keyboard keys currently consumed by the runtime.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Key {
    W,
    A,
    S,
    D,
    V,
    Escape,
    F3,
    F5,
    F9,
    Tab,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    PageUp,
    PageDown,
    BracketLeft,
    BracketRight,
    Minus,
    Equal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyInput {
    pub key: Key,
    pub pressed: bool,
}

/// Engine-owned events emitted by the native platform layer.
pub enum PlatformEvent<'a> {
    Resumed(&'a NativeWindow),
    CloseRequested,
    Resized(WindowSize),
    KeyboardInput(KeyInput),
    MouseLookButton { pressed: bool },
    MouseMotion { delta_x: f32, delta_y: f32 },
    Focused(bool),
    RedrawRequested,
    AboutToWait,
    Exiting,
}

/// An instruction returned to the native event loop after an event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoopControl {
    Continue,
    Exit,
}

/// Receives native events without exposing winit to the runtime.
pub trait EventHandler {
    fn on_event(&mut self, event: PlatformEvent<'_>) -> LoopControl;
}

/// An engine-owned description of a native platform failure.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum PlatformError {
    #[error("native event loop failed: {0}")]
    EventLoop(String),
    #[error("failed to create native window: {0}")]
    Window(String),
    #[error("failed to change cursor capture: {0}")]
    CursorGrab(String),
}

/// Runs the native event loop and dispatches engine-owned events.
pub fn run_event_loop(
    config: WindowConfig,
    handler: &mut dyn EventHandler,
) -> Result<(), PlatformError> {
    let event_loop =
        EventLoop::new().map_err(|error| PlatformError::EventLoop(error.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut application = NativeApplication {
        config,
        window: None,
        error: None,
        last_cursor_position: None,
        handler,
    };

    event_loop
        .run_app(&mut application)
        .map_err(|error| PlatformError::EventLoop(error.to_string()))?;

    if let Some(error) = application.error {
        return Err(error);
    }

    Ok(())
}

struct NativeApplication<'a> {
    config: WindowConfig,
    window: Option<NativeWindow>,
    error: Option<PlatformError>,
    last_cursor_position: Option<(f64, f64)>,
    handler: &'a mut dyn EventHandler,
}

impl NativeApplication<'_> {
    fn dispatch(&mut self, event_loop: &ActiveEventLoop, event: PlatformEvent<'_>) {
        if self.handler.on_event(event) == LoopControl::Exit {
            event_loop.exit();
        }
    }
}

impl ApplicationHandler for NativeApplication<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes().with_title(self.config.title.clone());
        match event_loop.create_window(attributes) {
            Ok(window) => {
                info!(title = %window.title(), "native window created");
                let window = NativeWindow(Arc::new(window), Arc::new(AtomicBool::new(false)));
                self.window = Some(window.clone());
                self.dispatch(event_loop, PlatformEvent::Resumed(&window));
            }
            Err(error) => {
                self.error = Some(PlatformError::Window(error.to_string()));
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let belongs_to_window = self
            .window
            .as_ref()
            .is_some_and(|window| window.0.id() == window_id);
        if !belongs_to_window {
            return;
        }

        if matches!(&event, WindowEvent::Focused(false)) {
            self.last_cursor_position = None;
        }
        let event = match event {
            WindowEvent::CloseRequested => {
                info!("native window close requested");
                PlatformEvent::CloseRequested
            }
            WindowEvent::Resized(size) => PlatformEvent::Resized(WindowSize {
                width: size.width,
                height: size.height,
            }),
            WindowEvent::KeyboardInput { event, .. } => {
                let Some(key) = map_key(event.physical_key) else {
                    return;
                };
                PlatformEvent::KeyboardInput(KeyInput {
                    key,
                    pressed: event.state == ElementState::Pressed,
                })
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => PlatformEvent::MouseLookButton {
                pressed: state == ElementState::Pressed,
            },
            WindowEvent::CursorMoved { position, .. } => {
                if self
                    .window
                    .as_ref()
                    .is_some_and(NativeWindow::cursor_captured)
                {
                    return;
                }
                let (delta_x, delta_y) =
                    cursor_delta(&mut self.last_cursor_position, (position.x, position.y));
                PlatformEvent::MouseMotion { delta_x, delta_y }
            }
            WindowEvent::Focused(focused) => PlatformEvent::Focused(focused),
            WindowEvent::RedrawRequested => PlatformEvent::RedrawRequested,
            _ => return,
        };

        self.dispatch(event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event
            && self
                .window
                .as_ref()
                .is_some_and(NativeWindow::cursor_captured)
        {
            self.dispatch(
                event_loop,
                PlatformEvent::MouseMotion {
                    delta_x: delta.0 as f32,
                    delta_y: delta.1 as f32,
                },
            );
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.dispatch(event_loop, PlatformEvent::AboutToWait);
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.dispatch(event_loop, PlatformEvent::Exiting);
        self.window.take();
        info!("native event loop exited");
    }
}

fn cursor_delta(previous: &mut Option<(f64, f64)>, current: (f64, f64)) -> (f32, f32) {
    let delta = previous
        .map(|previous| (current.0 - previous.0, current.1 - previous.1))
        .unwrap_or((0.0, 0.0));
    *previous = Some(current);
    (delta.0 as f32, delta.1 as f32)
}

fn map_key(key: PhysicalKey) -> Option<Key> {
    match key {
        PhysicalKey::Code(KeyCode::KeyW) => Some(Key::W),
        PhysicalKey::Code(KeyCode::KeyA) => Some(Key::A),
        PhysicalKey::Code(KeyCode::KeyS) => Some(Key::S),
        PhysicalKey::Code(KeyCode::KeyD) => Some(Key::D),
        PhysicalKey::Code(KeyCode::KeyV) => Some(Key::V),
        PhysicalKey::Code(KeyCode::Escape) => Some(Key::Escape),
        PhysicalKey::Code(KeyCode::F3) => Some(Key::F3),
        PhysicalKey::Code(KeyCode::F5) => Some(Key::F5),
        PhysicalKey::Code(KeyCode::F9) => Some(Key::F9),
        PhysicalKey::Code(KeyCode::Tab) => Some(Key::Tab),
        PhysicalKey::Code(KeyCode::ArrowLeft) => Some(Key::ArrowLeft),
        PhysicalKey::Code(KeyCode::ArrowRight) => Some(Key::ArrowRight),
        PhysicalKey::Code(KeyCode::ArrowUp) => Some(Key::ArrowUp),
        PhysicalKey::Code(KeyCode::ArrowDown) => Some(Key::ArrowDown),
        PhysicalKey::Code(KeyCode::PageUp) => Some(Key::PageUp),
        PhysicalKey::Code(KeyCode::PageDown) => Some(Key::PageDown),
        PhysicalKey::Code(KeyCode::BracketLeft) => Some(Key::BracketLeft),
        PhysicalKey::Code(KeyCode::BracketRight) => Some(Key::BracketRight),
        PhysicalKey::Code(KeyCode::Minus) => Some(Key::Minus),
        PhysicalKey::Code(KeyCode::Equal) => Some(Key::Equal),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_motion_is_relative_after_first_position() {
        let mut previous = None;
        assert_eq!(cursor_delta(&mut previous, (100.0, 100.0)), (0.0, 0.0));
        assert_eq!(cursor_delta(&mut previous, (112.0, 95.0)), (12.0, -5.0));
    }

    #[test]
    fn movement_keys_use_physical_key_codes() {
        assert_eq!(map_key(PhysicalKey::Code(KeyCode::KeyW)), Some(Key::W));
        assert_eq!(map_key(PhysicalKey::Code(KeyCode::KeyV)), Some(Key::V));
        assert_eq!(map_key(PhysicalKey::Code(KeyCode::KeyE)), None);
        assert_eq!(
            map_key(PhysicalKey::Code(KeyCode::Escape)),
            Some(Key::Escape)
        );
        assert_eq!(
            map_key(PhysicalKey::Unidentified(
                winit::keyboard::NativeKeyCode::Unidentified
            )),
            None
        );
    }
}
