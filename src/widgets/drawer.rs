// drawer.rs

use crate::{
    Direction, DirectionSetting, FocusPolicy, Key, KeyState, KeyboardEvent, LinearColor,
    LogicalPoint, LogicalRect, LogicalSize, RectDraw, ResolvedTheme, SemanticRole, Semantics,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DrawerEdge {
    #[default]
    InlineStart,
    InlineEnd,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DrawerAction {
    #[default]
    None,
    Opened,
    Dismissed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Drawer {
    label: String,
    pub open: bool,
    pub dismissible: bool,
    pub width: f32,
    pub edge: DrawerEdge,
    pub direction: DirectionSetting,
}

impl Drawer {
    pub fn new(label: impl Into<String>, width: f32) -> Self {
        Self {
            label: label.into(),
            open: false,
            dismissible: true,
            width: finite_non_negative(width),
            edge: DrawerEdge::InlineStart,
            direction: DirectionSetting::Inherit,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn open(&mut self) -> DrawerAction {
        if self.open {
            DrawerAction::None
        } else {
            self.open = true;
            DrawerAction::Opened
        }
    }

    pub fn dismiss(&mut self) -> DrawerAction {
        if self.open && self.dismissible {
            self.open = false;
            DrawerAction::Dismissed
        } else {
            DrawerAction::None
        }
    }

    pub fn handle_key(&mut self, event: &KeyboardEvent) -> DrawerAction {
        if event.state == KeyState::Pressed && !event.repeat && event.key == Key::Escape {
            self.dismiss()
        } else {
            DrawerAction::None
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

    pub fn layout(
        &self,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        viewport: LogicalSize,
    ) -> DrawerLayout {
        let direction = self.direction.resolve(inherited_direction);
        let panel_width = self.width.min(viewport.width);
        let physical_left = matches!(
            (self.edge, direction),
            (DrawerEdge::InlineStart, Direction::Ltr) | (DrawerEdge::InlineEnd, Direction::Rtl)
        );
        let panel_origin = LogicalPoint::new(
            if physical_left {
                0.0
            } else {
                viewport.width - panel_width
            },
            0.0,
        );
        let panel_size = LogicalSize::new(panel_width, viewport.height);
        DrawerLayout {
            size: if self.open {
                viewport
            } else {
                LogicalSize::default()
            },
            direction,
            panel_origin,
            panel_size,
            content_bounds: LogicalRect::new(
                LogicalPoint::new(panel_origin.x + theme.spacing.large, theme.spacing.large),
                LogicalSize::new(
                    (panel_width - theme.spacing.large * 2.0).max(0.0),
                    (viewport.height - theme.spacing.large * 2.0).max(0.0),
                ),
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawerLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub panel_origin: LogicalPoint,
    pub panel_size: LogicalSize,
    pub content_bounds: LogicalRect,
}

impl DrawerLayout {
    pub fn draws(&self, drawer: &Drawer, theme: &ResolvedTheme) -> Vec<RectDraw> {
        if !drawer.open {
            return Vec::new();
        }
        vec![
            RectDraw {
                position: [0.0, 0.0],
                size: [self.size.width, self.size.height],
                radii: [0.0; 4],
                color: LinearColor::new(0.0, 0.0, 0.0, 0.45).to_array(),
                border_width: 0.0,
                border_color: [0.0; 4],
            },
            RectDraw {
                position: [self.panel_origin.x, self.panel_origin.y],
                size: [self.panel_size.width, self.panel_size.height],
                radii: [0.0; 4],
                color: theme.colors.surface_elevated.to_array(),
                border_width: theme.borders.thin,
                border_color: theme.colors.border.to_array(),
            },
        ]
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Drawer, DrawerAction, DrawerEdge};
    use crate::{
        Direction, Key, KeyboardEvent, LogicalSize, SemanticRole, ThemeController, ThemeDefinition,
        UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn visibility_controls_focus_semantics_and_escape() {
        let mut drawer = Drawer::new("Navigation", 280.0);
        assert!(drawer.semantics().state.hidden);
        assert_eq!(drawer.open(), DrawerAction::Opened);
        assert_eq!(drawer.semantics().role, SemanticRole::Dialog);
        assert!(drawer.focus_policy().focusable);
        assert_eq!(
            drawer.handle_key(&KeyboardEvent::pressed(Key::Escape)),
            DrawerAction::Dismissed
        );
    }

    #[test]
    fn logical_edges_mirror_between_ltr_and_rtl() {
        let mut drawer = Drawer::new("Navigation", 120.0);
        drawer.open = true;
        drawer.edge = DrawerEdge::InlineStart;
        let viewport = LogicalSize::new(320.0, 200.0);
        let theme = theme();
        let ltr = drawer.layout(&theme, Direction::Ltr, viewport);
        let rtl = drawer.layout(&theme, Direction::Rtl, viewport);
        assert_eq!(ltr.panel_origin.x, 0.0);
        assert_eq!(rtl.panel_origin.x, 200.0);
        drawer.edge = DrawerEdge::InlineEnd;
        assert_eq!(
            drawer
                .layout(&theme, Direction::Rtl, viewport)
                .panel_origin
                .x,
            0.0
        );
    }

    #[test]
    fn width_clamps_and_scrim_precedes_panel() {
        let mut drawer = Drawer::new("Navigation", 500.0);
        drawer.open = true;
        let viewport = LogicalSize::new(320.0, 200.0);
        let theme = theme();
        let layout = drawer.layout(&theme, Direction::Ltr, viewport);
        assert_eq!(layout.panel_size.width, viewport.width);
        assert_eq!(layout.content_bounds.origin.x, theme.spacing.large);
        let draws = layout.draws(&drawer, &theme);
        assert_eq!(draws.len(), 2);
        assert_eq!(draws[0].size, [viewport.width, viewport.height]);
        assert_eq!(draws[1].position, [0.0, 0.0]);
    }
}
