// main.rs
#[cfg(test)]
mod raster;
mod renderer;

use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use renderer::Renderer;

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    redraw_until: Option<Instant>,
}

impl App {
    fn begin_resize_redraw(&mut self, event_loop: &ActiveEventLoop) {
        self.redraw_until = Some(Instant::now() + Duration::from_millis(200));
        event_loop.set_control_flow(ControlFlow::Poll);
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
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
        window.request_redraw();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.renderer = None;
        self.window = None;
        self.redraw_until = None;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                renderer.resize(size);
                self.begin_resize_redraw(event_loop);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                renderer.scale_factor_changed(scale_factor);
                self.begin_resize_redraw(event_loop);
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = renderer.render() {
                    eprintln!("Mio-GUI render error: {error}");
                }
                if self
                    .redraw_until
                    .is_some_and(|deadline| Instant::now() < deadline)
                {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                } else {
                    self.redraw_until = None;
                    event_loop.set_control_flow(ControlFlow::Wait);
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
