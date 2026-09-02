#![cfg_attr(windows, windows_subsystem = "windows")]

//! CindyDrift — the cotton-candy drift as a Windows screensaver.
//!
//! Everything is CPU-composited into a plain u32 framebuffer (winit +
//! softbuffer), so there is no GPU driver, no webview, and no network use.

mod raster;
mod scene;
mod sprites;

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Fullscreen, Window, WindowId, WindowLevel};

const FRAME: Duration = Duration::from_micros(16_667); // ~60fps
/// Cumulative pointer travel, in pixels, that counts as "the user is back".
const MOUSE_EXIT_PX: f64 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `/s`, or a bare double-click: run the drift fullscreen.
    Run,
    /// `/c[:hwnd]` (configure) and `/p <hwnd>` (thumbnail preview): exit 0.
    Exit,
}

/// Screensaver flags are case-insensitive and may use `/` or `-`, and `/c` may
/// carry its window handle as `/c:12345`.
pub fn parse_args<I, S>(args: I) -> Mode
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    for arg in args {
        let lower = arg.as_ref().trim().to_ascii_lowercase();
        let flag = lower
            .strip_prefix('/')
            .or_else(|| lower.strip_prefix('-'))
            .unwrap_or("");
        match flag.chars().next() {
            Some('c') | Some('p') => return Mode::Exit,
            Some('s') => return Mode::Run,
            _ => {}
        }
    }
    // No recognised flag (including no args at all — people double-click the
    // .scr to preview it): run the drift.
    Mode::Run
}

fn main() {
    if parse_args(std::env::args().skip(1)) == Mode::Exit {
        return;
    }
    if let Ok(event_loop) = EventLoop::new() {
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = App::default();
        let _ = event_loop.run_app(&mut app);
    }
}

#[derive(Default)]
struct App {
    window: Option<Rc<Window>>,
    context: Option<softbuffer::Context<Rc<Window>>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    scene: Option<scene::Scene>,
    size: (u32, u32),
    start: Option<Instant>,
    last: Option<Instant>,
    next_frame: Option<Instant>,
    cursor_origin: Option<(f64, f64)>,
    mouse_travel: f64,
}

impl App {
    fn quit(&self, el: &ActiveEventLoop) {
        el.exit();
    }

    fn ensure_scene(&mut self, w: u32, h: u32) {
        if self.scene.is_none() || self.size != (w, h) {
            self.size = (w, h);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos() ^ d.as_secs() as u32)
                .unwrap_or(0x5eed);
            let seed = nanos ^ w.wrapping_shl(8) ^ h ^ 0x0cd1_5a11;
            self.scene = Some(scene::Scene::new(w as usize, h as usize, seed));
        }
    }

    fn draw(&mut self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let inner = window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(inner.width), NonZeroU32::new(inner.height))
        else {
            return;
        };
        self.ensure_scene(inner.width, inner.height);

        let Some(surface) = self.surface.as_mut() else {
            return;
        };
        if surface.resize(w, h).is_err() {
            return;
        }

        let now = Instant::now();
        let start = *self.start.get_or_insert(now);
        let last = self.last.replace(now).unwrap_or(now);
        let t = now.duration_since(start).as_secs_f32();
        let dt = now.duration_since(last).as_secs_f32().min(0.1);

        if let (Some(scene), Ok(mut buffer)) = (self.scene.as_mut(), surface.buffer_mut()) {
            scene.update(t, dt);
            scene.render(&mut buffer, t);
            let _ = buffer.present();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let monitor = el.primary_monitor().or_else(|| el.available_monitors().next());
        let attrs = Window::default_attributes()
            .with_title("CindyDrift")
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_fullscreen(Some(Fullscreen::Borderless(monitor)));
        let Ok(window) = el.create_window(attrs) else {
            el.exit();
            return;
        };
        window.set_cursor_visible(false);
        let window = Rc::new(window);

        let Ok(context) = softbuffer::Context::new(window.clone()) else {
            el.exit();
            return;
        };
        let Ok(surface) = softbuffer::Surface::new(&context, window.clone()) else {
            el.exit();
            return;
        };
        self.context = Some(context);
        self.surface = Some(surface);
        self.window = Some(window);
        self.next_frame = Some(Instant::now());
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.quit(el),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    self.quit(el);
                }
            }
            WindowEvent::MouseInput { .. } | WindowEvent::Touch(_) => self.quit(el),
            WindowEvent::CursorMoved { position, .. } => {
                let p = (position.x, position.y);
                match self.cursor_origin {
                    None => self.cursor_origin = Some(p),
                    Some(o) => {
                        let d = ((p.0 - o.0).powi(2) + (p.1 - o.1).powi(2)).sqrt();
                        if d > MOUSE_EXIT_PX {
                            self.quit(el);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.draw();
                // Pace from the deadline we just met, not from now, so a slow
                // frame does not compound into a slower cadence. If we fall
                // more than a frame behind, drop the backlog.
                let now = Instant::now();
                let next = self.next_frame.unwrap_or(now) + FRAME;
                self.next_frame = Some(if next < now { now + FRAME } else { next });
            }
            _ => {}
        }
    }

    fn device_event(&mut self, el: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        // Raw motion catches the case where the pointer is already pinned to a
        // screen edge and CursorMoved stops reporting.
        if let DeviceEvent::MouseMotion { delta } = event {
            self.mouse_travel += (delta.0 * delta.0 + delta.1 * delta.1).sqrt();
            if self.mouse_travel > MOUSE_EXIT_PX {
                self.quit(el);
            }
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        let next = self.next_frame.unwrap_or_else(Instant::now);
        if Instant::now() >= next {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        el.set_control_flow(ControlFlow::WaitUntil(next));
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Mode};

    #[test]
    fn no_args_runs_fullscreen() {
        let empty: [&str; 0] = [];
        assert_eq!(parse_args(empty), Mode::Run);
    }

    #[test]
    fn show_flag_is_case_insensitive_and_accepts_dashes() {
        assert_eq!(parse_args(["/s"]), Mode::Run);
        assert_eq!(parse_args(["/S"]), Mode::Run);
        assert_eq!(parse_args(["-s"]), Mode::Run);
    }

    #[test]
    fn preview_and_config_exit_immediately() {
        assert_eq!(parse_args(["/p", "1234"]), Mode::Exit);
        assert_eq!(parse_args(["/P", "1234"]), Mode::Exit);
        assert_eq!(parse_args(["/c"]), Mode::Exit);
        assert_eq!(parse_args(["/c:98765"]), Mode::Exit);
        assert_eq!(parse_args(["-C:98765"]), Mode::Exit);
    }

    #[test]
    fn bare_window_handle_is_not_a_flag() {
        assert_eq!(parse_args(["1234"]), Mode::Run);
    }
}
