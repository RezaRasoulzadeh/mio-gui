// app.rs
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Ime, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, ModifiersState};
use winit::window::{Window, WindowId};

use crate::{
    AdornmentPlacement, ColorScheme, ComponentSize, ComponentState, ComponentStyle, Direction,
    LinearColor, RectDraw, Renderer, SystemClipboard, TextAlign, TextDraw, TextEditState,
    TextStyle, ThemeController, ThemeDefinition, ThemeMode, UserPreferences, VisualVariant,
    apply_winit_theme,
};

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
    theme: ThemeController,
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
        if let Some(theme) = window.theme() {
            apply_winit_theme(&mut self.theme, theme);
        }
        let mut renderer = match pollster::block_on(Renderer::new(window.clone())) {
            Ok(renderer) => renderer,
            Err(error) => {
                eprintln!("Mio-GUI renderer initialization failed: {error}");
                event_loop.exit();
                return;
            }
        };
        set_gallery_fixture(&mut renderer, window.inner_size());
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
            WindowEvent::ThemeChanged(theme) => {
                if apply_winit_theme(&mut self.theme, theme) {
                    self.redraws_remaining = SETTLE_REDRAWS;
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
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
                    set_gallery_fixture(renderer, size);
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

fn set_gallery_fixture(renderer: &mut Renderer, size: PhysicalSize<u32>) {
    let scale_factor = renderer.scale_factor();
    let viewport = [
        size.width as f32 / scale_factor,
        size.height as f32 / scale_factor,
    ];
    let (rects, texts) = gallery_draws(viewport);
    renderer.set_rect_draws(&rects);
    let _ = renderer.set_text_draws(&texts);
}

pub(crate) fn gallery_draws(viewport: [f32; 2]) -> (Vec<RectDraw>, Vec<TextDraw>) {
    let margin = 12.0;
    let gap = 12.0;
    let panel_size = [
        ((viewport[0] - margin * 2.0 - gap) * 0.5).max(180.0),
        ((viewport[1] - margin * 2.0 - gap) * 0.5).max(240.0),
    ];
    let panels = [
        (Direction::Ltr, ColorScheme::Light),
        (Direction::Rtl, ColorScheme::Light),
        (Direction::Ltr, ColorScheme::Dark),
        (Direction::Rtl, ColorScheme::Dark),
    ];
    let mut rects = Vec::new();
    let mut texts = Vec::new();
    let definition = ThemeDefinition::default();

    for (index, (direction, scheme)) in panels.into_iter().enumerate() {
        let origin = [
            margin + (index % 2) as f32 * (panel_size[0] + gap),
            margin + (index / 2) as f32 * (panel_size[1] + gap),
        ];
        let mut controller = ThemeController::default();
        controller.set_mode(match scheme {
            ColorScheme::Light => ThemeMode::Light,
            ColorScheme::Dark => ThemeMode::Dark,
        });
        let theme = definition.resolve(controller, UserPreferences::default());
        rects.push(rect_draw(
            origin,
            panel_size,
            theme.radii.large,
            theme.colors.surface,
            theme.borders.thin,
            theme.colors.border,
        ));
        texts.push(TextDraw {
            text: format!("{direction:?} / {scheme:?}"),
            style: TextStyle {
                font_size: 16.0,
                line_height: 22.0,
                ..TextStyle::default()
            },
            baseline: [origin[0] + panel_size[0] * 0.5, origin[1] + 27.0],
            align: TextAlign::Center,
            color: theme.colors.text.to_array(),
        });

        let variants = [
            (VisualVariant::Solid, ComponentState::default()),
            (
                VisualVariant::Outline,
                ComponentState {
                    hovered: true,
                    ..ComponentState::default()
                },
            ),
            (
                VisualVariant::Soft,
                ComponentState {
                    focused: true,
                    selected: true,
                    ..ComponentState::default()
                },
            ),
            (
                VisualVariant::Ghost,
                ComponentState {
                    active: true,
                    error: true,
                    ..ComponentState::default()
                },
            ),
        ];
        let sizes = [
            ComponentSize::Small,
            ComponentSize::Medium,
            ComponentSize::Large,
        ];
        let available = panel_size[0] - 24.0;
        let column_gap = 8.0;
        let item_width = (available - column_gap * 2.0) / 3.0;

        for (row, (variant, state)) in variants.into_iter().enumerate() {
            for (column, component_size) in sizes.into_iter().enumerate() {
                let physical_column = match direction {
                    Direction::Ltr => column,
                    Direction::Rtl => sizes.len() - 1 - column,
                };
                let style = ComponentStyle {
                    size: component_size,
                    variant,
                    state,
                    adornment: AdornmentPlacement::InlineStart,
                }
                .resolve(&theme, direction);
                let item_size = [item_width, style.metrics.minimum_block_size];
                let position = [
                    origin[0] + 12.0 + physical_column as f32 * (item_width + column_gap),
                    origin[1] + 46.0 + row as f32 * 51.0,
                ];
                push_component_draws(&mut rects, position, item_size, style, theme.colors.surface);
            }
        }
    }
    (rects, texts)
}

fn push_component_draws(
    draws: &mut Vec<RectDraw>,
    position: [f32; 2],
    size: [f32; 2],
    style: crate::ResolvedComponentStyle,
    surface: LinearColor,
) {
    let appearance = style.appearance;
    if appearance.focus_ring_width > 0.0 {
        let width = appearance.focus_ring_width;
        draws.push(rect_draw(
            [position[0] - width, position[1] - width],
            [size[0] + width * 2.0, size[1] + width * 2.0],
            style.metrics.corner_radius + width,
            appearance.focus_ring,
            0.0,
            appearance.focus_ring,
        ));
    }
    draws.push(rect_draw(
        position,
        size,
        style.metrics.corner_radius,
        appearance.background.composite_over(surface),
        appearance.border_width,
        appearance.border,
    ));
    if appearance.state_layer.alpha > 0.0 {
        draws.push(rect_draw(
            position,
            size,
            style.metrics.corner_radius,
            appearance.state_layer,
            0.0,
            appearance.state_layer,
        ));
    }
    let adornment_x = match style.adornment {
        crate::PhysicalAdornmentPlacement::Left => position[0] + style.metrics.padding_inline,
        crate::PhysicalAdornmentPlacement::Right => {
            position[0] + size[0] - style.metrics.padding_inline - 10.0
        }
        crate::PhysicalAdornmentPlacement::None => return,
    };
    draws.push(rect_draw(
        [adornment_x, position[1] + (size[1] - 10.0) * 0.5],
        [10.0, 10.0],
        5.0,
        appearance.foreground,
        0.0,
        appearance.foreground,
    ));
}

fn rect_draw(
    position: [f32; 2],
    size: [f32; 2],
    radius: f32,
    color: LinearColor,
    border_width: f32,
    border_color: LinearColor,
) -> RectDraw {
    RectDraw {
        position,
        size,
        radii: [radius; 4],
        color: color.to_array(),
        border_width,
        border_color: border_color.to_array(),
    }
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

#[cfg(test)]
mod tests {
    use super::gallery_draws;

    #[test]
    fn gallery_contains_four_complete_direction_and_scheme_panels() {
        let (rects, texts) = gallery_draws([800.0, 600.0]);

        assert_eq!(rects.len(), 136);
        assert_eq!(texts.len(), 4);
        assert_eq!(texts[0].text, "Ltr / Light");
        assert_eq!(texts[1].text, "Rtl / Light");
        assert_eq!(texts[2].text, "Ltr / Dark");
        assert_eq!(texts[3].text, "Rtl / Dark");
    }

    #[test]
    fn rtl_gallery_geometry_is_the_exact_horizontal_mirror_of_ltr() {
        let (rects, _) = gallery_draws([800.0, 600.0]);
        let panel_width = 382.0;
        let rtl_origin = 406.0;

        for (index, (ltr, rtl)) in rects[0..34].iter().zip(&rects[34..68]).enumerate() {
            let ltr_local_x = ltr.position[0] - 12.0;
            let rtl_local_x = rtl.position[0] - rtl_origin;
            assert!(
                (rtl_local_x - (panel_width - ltr_local_x - ltr.size[0])).abs() < 0.001,
                "primitive={index} ltr={ltr:?} rtl={rtl:?}"
            );
            assert_eq!(ltr.position[1], rtl.position[1]);
            assert_eq!(ltr.size, rtl.size);
            assert_eq!(ltr.radii, rtl.radii);
            assert_eq!(ltr.color, rtl.color);
            assert_eq!(ltr.border_width, rtl.border_width);
            assert_eq!(ltr.border_color, rtl.border_color);
        }
    }
}
