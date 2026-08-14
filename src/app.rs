// app.rs
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Ime, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, ModifiersState};
use winit::window::{Window, WindowId};

use crate::{Renderer, SystemClipboard, TextAlign, TextDraw, TextEditState, TextStyle};

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
    text_edit: TextEditState,
    clipboard: Option<SystemClipboard>,
    modifiers: ModifiersState,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
            return;
        }

        let attrs = Window::default_attributes().with_title("Mio-GUI");
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("Mio-GUI window creation failed: {error}");
                event_loop.exit();
                return;
            }
        };
        window.set_ime_allowed(true);
        let mut renderer = match pollster::block_on(Renderer::new(window.clone())) {
            Ok(renderer) => renderer,
            Err(error) => {
                eprintln!("Mio-GUI renderer initialization failed: {error}");
                event_loop.exit();
                return;
            }
        };
        set_text_fixture(&mut renderer, window.inner_size());
        self.clipboard = match SystemClipboard::new() {
            Ok(clipboard) => Some(clipboard),
            Err(error) => {
                eprintln!("Mio-GUI clipboard initialization failed: {error}");
                None
            }
        };
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
        self.clipboard = None;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                self.clipboard = None;
                event_loop.exit();
            }
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
            WindowEvent::Ime(ime) => match ime {
                Ime::Enabled => {}
                Ime::Preedit(text, selection) => {
                    self.text_edit.update_composition_with_selection(
                        &text,
                        selection.map(|(start, end)| start..end),
                    );
                }
                Ime::Commit(text) => {
                    if self.text_edit.composition_range().is_some() {
                        self.text_edit.update_composition(&text);
                        self.text_edit.commit_composition();
                    } else {
                        self.text_edit.paste(&text);
                    }
                }
                Ime::Disabled => self.text_edit.commit_composition(),
            },
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                self.handle_clipboard_shortcut(&event.logical_key);
            }
            WindowEvent::RedrawRequested => {
                if let Some(size) = self.pending_size.take() {
                    renderer.resize(size);
                    set_text_fixture(renderer, size);
                }
                if let Err(error) = renderer.render() {
                    eprintln!("Mio-GUI render error: {error}");
                }
                self.redraws_remaining = self.redraws_remaining.saturating_sub(1);
                if let (true, Some(window)) = (self.redraws_remaining > 0, self.window.as_ref()) {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn handle_clipboard_shortcut(&mut self, key: &Key) {
        let Some(clipboard) = self.clipboard.as_mut() else {
            return;
        };
        if !primary_modifier(self.modifiers) {
            return;
        }
        let Key::Character(character) = key else {
            return;
        };

        let result = if character.eq_ignore_ascii_case("c") {
            self.text_edit.copy_to(clipboard)
        } else if character.eq_ignore_ascii_case("x") {
            self.text_edit.cut_to(clipboard)
        } else if character.eq_ignore_ascii_case("v") {
            self.text_edit.paste_from(clipboard)
        } else {
            return;
        };

        if let Err(error) = result {
            eprintln!("Mio-GUI clipboard error: {error}");
        }
    }
}

fn primary_modifier(modifiers: ModifiersState) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    }
}

fn set_text_fixture(renderer: &mut Renderer, size: PhysicalSize<u32>) {
    let scale_factor = renderer.scale_factor();
    let center = [
        size.width as f32 / scale_factor * 0.5,
        size.height as f32 / scale_factor * 0.5,
    ];
    let samples = [
        ("رابط کاربری راست‌به‌چپ", -38.0),
        ("Mio-GUI left-to-right", 0.0),
        ("نسخه Mio-GUI 2", 38.0),
    ];
    let draws = samples.map(|(text, offset)| TextDraw {
        text: text.to_owned(),
        style: TextStyle {
            font_size: 20.0,
            line_height: 28.0,
            ..TextStyle::default()
        },
        baseline: [center[0], center[1] + offset],
        align: TextAlign::Center,
        color: [0.08, 0.07, 0.05, 1.0],
    });
    let _ = renderer.set_text_draws(&draws);
}

pub fn run() {
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(error) => {
            eprintln!("Mio-GUI event loop creation failed: {error}");
            return;
        }
    };
    let mut app = App::default();
    if let Err(error) = event_loop.run_app(&mut app) {
        eprintln!("Mio-GUI event loop failed: {error}");
    }
}
