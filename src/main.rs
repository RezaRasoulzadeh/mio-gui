// main.rs
mod raster;
mod renderer;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use renderer::Renderer;

#[cfg(target_os = "macos")]
const SETTLE_REDRAWS: u8 = 3;
#[cfg(not(target_os = "macos"))]
const SETTLE_REDRAWS: u8 = 1;

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    redraws_remaining: u8,
    pending_size: Option<PhysicalSize<u32>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
            return;
        }

        let attrs = Window::default_attributes().with_title("Mio-GUI");
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let renderer = pollster::block_on(Renderer::new(window.clone()));
        self.window = Some(window.clone());
        self.renderer = Some(renderer);
        self.redraws_remaining = SETTLE_REDRAWS;
        window.request_redraw();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.renderer = None;
        self.window = None;
        self.redraws_remaining = 0;
        self.pending_size = None;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.pending_size = Some(size);
                self.redraws_remaining = SETTLE_REDRAWS;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                renderer.scale_factor_changed(scale_factor);
                self.redraws_remaining = SETTLE_REDRAWS;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(size) = self.pending_size.take() {
                    renderer.resize(size);
                }
                if let Err(error) = renderer.render() {
                    eprintln!("Mio-GUI render error: {error}");
                }
                self.redraws_remaining = self.redraws_remaining.saturating_sub(1);
                if self.redraws_remaining > 0
                    && let Some(window) = &self.window
                {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
