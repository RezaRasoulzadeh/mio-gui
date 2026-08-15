// modal.rs

use crate::{
    FocusPolicy, Key, KeyState, KeyboardEvent, LinearColor, LogicalPoint, LogicalRect, LogicalSize,
    RectDraw, ResolvedTheme, SemanticRole, Semantics,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ModalAction {
    #[default]
    None,
    Opened,
    Dismissed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Modal {
    label: String,
    pub open: bool,
    pub dismissible: bool,
    pub content_size: LogicalSize,
}

impl Modal {
    pub fn new(label: impl Into<String>, content_size: LogicalSize) -> Self {
        Self {
            label: label.into(),
            open: false,
            dismissible: true,
            content_size,
        }
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn open(&mut self) -> ModalAction {
        if self.open {
            ModalAction::None
        } else {
            self.open = true;
            ModalAction::Opened
        }
    }
    pub fn dismiss(&mut self) -> ModalAction {
        if self.open && self.dismissible {
            self.open = false;
            ModalAction::Dismissed
        } else {
            ModalAction::None
        }
    }
    pub fn handle_key(&mut self, event: &KeyboardEvent) -> ModalAction {
        if event.state == KeyState::Pressed && !event.repeat && event.key == Key::Escape {
            self.dismiss()
        } else {
            ModalAction::None
        }
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Dialog).with_name(self.label.clone());
        semantics.state.hidden = !self.open;
        semantics
    }
    pub fn focus_policy(&self) -> FocusPolicy {
        if self.open {
            FocusPolicy::focusable()
        } else {
            FocusPolicy {
                hidden: true,
                ..FocusPolicy::default()
            }
        }
    }
    pub fn layout(&self, theme: &ResolvedTheme, viewport: LogicalSize) -> ModalLayout {
        let maximum = LogicalSize::new(
            (viewport.width - theme.spacing.large * 2.0).max(0.0),
            (viewport.height - theme.spacing.large * 2.0).max(0.0),
        );
        let content_size = LogicalSize::new(
            self.content_size.width.min(maximum.width),
            self.content_size.height.min(maximum.height),
        );
        let panel_size = LogicalSize::new(
            content_size.width + theme.spacing.large * 2.0,
            content_size.height + theme.spacing.large * 2.0,
        );
        let panel_origin = LogicalPoint::new(
            (viewport.width - panel_size.width) * 0.5,
            (viewport.height - panel_size.height) * 0.5,
        );
        ModalLayout {
            size: if self.open {
                viewport
            } else {
                LogicalSize::default()
            },
            panel_origin,
            panel_size,
            content_bounds: LogicalRect::new(
                LogicalPoint::new(
                    panel_origin.x + theme.spacing.large,
                    panel_origin.y + theme.spacing.large,
                ),
                content_size,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModalLayout {
    pub size: LogicalSize,
    pub panel_origin: LogicalPoint,
    pub panel_size: LogicalSize,
    pub content_bounds: LogicalRect,
}

impl ModalLayout {
    pub fn draws(&self, modal: &Modal, theme: &ResolvedTheme) -> Vec<RectDraw> {
        if !modal.open {
            return Vec::new();
        }
        vec![
            RectDraw {
                position: [0.0, 0.0],
                size: [self.size.width, self.size.height],
                radii: [0.0; 4],
                color: LinearColor::new(0.0, 0.0, 0.0, 0.55).to_array(),
                border_width: 0.0,
                border_color: [0.0; 4],
            },
            RectDraw {
                position: [self.panel_origin.x, self.panel_origin.y],
                size: [self.panel_size.width, self.panel_size.height],
                radii: [theme.radii.large; 4],
                color: theme.colors.surface_elevated.to_array(),
                border_width: theme.borders.thin,
                border_color: theme.colors.border.to_array(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{Modal, ModalAction};
    use crate::{
        Key, KeyboardEvent, LogicalSize, SemanticRole, ThemeController, ThemeDefinition,
        UserPreferences,
    };
    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }
    #[test]
    fn visibility_controls_focus_semantics_and_escape() {
        let mut modal = Modal::new("Confirm deletion", LogicalSize::new(240.0, 120.0));
        assert!(modal.semantics().state.hidden);
        assert_eq!(modal.open(), ModalAction::Opened);
        assert_eq!(modal.semantics().role, SemanticRole::Dialog);
        assert!(modal.focus_policy().focusable);
        assert_eq!(
            modal.handle_key(&KeyboardEvent::pressed(Key::Escape)),
            ModalAction::Dismissed
        );
    }
    #[test]
    fn non_dismissible_modal_ignores_escape() {
        let mut modal = Modal::new("Required action", LogicalSize::new(240.0, 120.0));
        modal.open = true;
        modal.dismissible = false;
        assert_eq!(
            modal.handle_key(&KeyboardEvent::pressed(Key::Escape)),
            ModalAction::None
        );
        assert!(modal.open);
    }
    #[test]
    fn panel_centers_clamps_and_emits_scrim_first() {
        let mut modal = Modal::new("Confirm deletion", LogicalSize::new(500.0, 300.0));
        modal.open = true;
        let viewport = LogicalSize::new(320.0, 200.0);
        let theme = theme();
        let layout = modal.layout(&theme, viewport);
        assert!(layout.panel_size.width <= viewport.width);
        assert!(layout.panel_size.height <= viewport.height);
        assert_eq!(
            layout.panel_origin.x,
            (viewport.width - layout.panel_size.width) * 0.5
        );
        let draws = layout.draws(&modal, &theme);
        assert_eq!(draws.len(), 2);
        assert_eq!(draws[0].size, [viewport.width, viewport.height]);
    }
}
