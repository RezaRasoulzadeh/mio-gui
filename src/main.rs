// main.rs
#[cfg(test)]
mod raster;
mod renderer;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use renderer::Renderer;

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    redraws_remaining: u8,
    // Most recent size from a `Resized` event that has not yet been applied
    // to the surface. A burst of `Resized` events (interactive drag, an
    // animated maximize) overwrites this rather than each one triggering
    // its own `surface.configure`, so the expensive reconfigure only runs
    // once per drawn frame, against the latest known size.
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
        self.redraws_remaining = 3;
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
                // Record the size but don't touch the surface here. During a
                // rapid resize or an animated maximize, winit can deliver
                // many `Resized` events before a single frame is drawn; if
                // each one called `surface.configure` synchronously, the
                // event loop would stall behind repeated swapchain
                // recreation. Storing the latest size and requesting a
                // redraw lets multiple queued `Resized` events collapse
                // into one reconfigure, applied against the newest size,
                // right before the next frame renders.
                self.pending_size = Some(size);
                self.redraws_remaining = 3;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                renderer.scale_factor_changed(scale_factor);
                self.redraws_remaining = 3;
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