// context_menu.rs

use crate::{
    Direction, DirectionSetting, FocusPolicy, KeyboardEvent, LogicalConstraints, LogicalPoint,
    LogicalSize, Menu, MenuAction, MenuDraws, MenuLayout, ResolvedTheme, Semantics, TextSystem,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ContextMenuAction {
    #[default]
    None,
    Opened,
    ActiveChanged(usize),
    Activated(usize),
    Dismissed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenu {
    pub menu: Menu,
    pub open: bool,
    pub anchor: LogicalPoint,
    pub direction: DirectionSetting,
}

impl ContextMenu {
    pub fn new(menu: Menu) -> Self {
        Self {
            menu,
            open: false,
            anchor: LogicalPoint::default(),
            direction: DirectionSetting::Inherit,
        }
    }

    pub fn open_at(&mut self, anchor: LogicalPoint) -> ContextMenuAction {
        self.anchor = anchor;
        self.open = true;
        ContextMenuAction::Opened
    }

    pub fn dismiss(&mut self) -> bool {
        let was_open = self.open;
        self.open = false;
        was_open
    }

    pub fn handle_key(&mut self, event: &KeyboardEvent) -> ContextMenuAction {
        if !self.open {
            return ContextMenuAction::None;
        }
        match self.menu.handle_key(event) {
            MenuAction::None => ContextMenuAction::None,
            MenuAction::ActiveChanged(index) => ContextMenuAction::ActiveChanged(index),
            MenuAction::Activated(index) => {
                self.open = false;
                ContextMenuAction::Activated(index)
            }
            MenuAction::Dismissed => {
                self.open = false;
                ContextMenuAction::Dismissed
            }
        }
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = self.menu.semantics();
        semantics.state.hidden = !self.open;
        semantics
    }

    pub fn focus_policy(&self) -> FocusPolicy {
        if self.open {
            self.menu.focus_policy()
        } else {
            FocusPolicy {
                hidden: true,
                ..FocusPolicy::default()
            }
        }
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> ContextMenuLayout {
        let direction = self.direction.resolve(inherited_direction);
        let menu = self.menu.layout(
            text_system,
            theme,
            direction,
            LogicalConstraints::loose(constraints.max),
        );
        let viewport = LogicalSize::new(
            finite_or(constraints.max.width, menu.size.width),
            finite_or(constraints.max.height, menu.size.height),
        );
        let preferred_x = match direction {
            Direction::Ltr => self.anchor.x,
            Direction::Rtl => self.anchor.x - menu.size.width,
        };
        ContextMenuLayout {
            size: if self.open {
                constraints.constrain(viewport)
            } else {
                LogicalSize::default()
            },
            direction,
            menu_origin: LogicalPoint::new(
                preferred_x.clamp(0.0, (viewport.width - menu.size.width).max(0.0)),
                self.anchor
                    .y
                    .clamp(0.0, (viewport.height - menu.size.height).max(0.0)),
            ),
            menu,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenuLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub menu_origin: LogicalPoint,
    pub menu: MenuLayout,
}

impl ContextMenuLayout {
    pub fn draws(
        &self,
        context_menu: &ContextMenu,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
    ) -> MenuDraws {
        if !context_menu.open {
            return MenuDraws {
                rectangles: Vec::new(),
                text: Vec::new(),
            };
        }
        self.menu.draws(
            &context_menu.menu,
            LogicalPoint::new(origin.x + self.menu_origin.x, origin.y + self.menu_origin.y),
            theme,
        )
    }
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::{ContextMenu, ContextMenuAction};
    use crate::{
        ArrowKey, Direction, Key, KeyboardEvent, LogicalConstraints, LogicalPoint, LogicalSize,
        Menu, MenuItem, TextSystem, ThemeController, ThemeDefinition, UserPreferences,
    };

    fn context_menu() -> ContextMenu {
        ContextMenu::new(
            Menu::new(
                "Context actions",
                vec![MenuItem::new("Copy"), MenuItem::new("Delete")],
            )
            .unwrap(),
        )
    }

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn opening_keyboard_activation_and_dismissal_update_visibility() {
        let mut menu = context_menu();
        assert!(menu.semantics().state.hidden);
        assert_eq!(
            menu.open_at(LogicalPoint::new(20.0, 30.0)),
            ContextMenuAction::Opened
        );
        let semantics = menu.semantics();
        assert!(!semantics.state.hidden);
        assert_eq!(semantics.virtual_children().len(), 2);
        assert_eq!(
            menu.handle_key(&KeyboardEvent::pressed(Key::Arrow(ArrowKey::Down))),
            ContextMenuAction::ActiveChanged(1)
        );
        assert_eq!(
            menu.handle_key(&KeyboardEvent::pressed(Key::Enter)),
            ContextMenuAction::Activated(1)
        );
        assert!(!menu.open);
    }

    #[test]
    fn popup_clamps_to_viewport_edges_in_both_directions() {
        let mut menu = context_menu();
        menu.open_at(LogicalPoint::new(195.0, 95.0));
        let theme = theme();
        let mut text_system = TextSystem::new();
        let constraints = LogicalConstraints::tight(LogicalSize::new(200.0, 100.0));
        let ltr = menu.layout(&mut text_system, &theme, Direction::Ltr, constraints);
        let rtl = menu.layout(&mut text_system, &theme, Direction::Rtl, constraints);
        assert!(ltr.menu_origin.x + ltr.menu.size.width <= 200.0);
        assert!(ltr.menu_origin.y + ltr.menu.size.height <= 100.0);
        assert!(rtl.menu_origin.x >= 0.0);
        assert_eq!(ltr.size, LogicalSize::new(200.0, 100.0));
    }

    #[test]
    fn closed_context_menu_has_no_geometry_or_paint() {
        let menu = context_menu();
        let theme = theme();
        let mut text_system = TextSystem::new();
        let layout = menu.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        assert_eq!(layout.size, LogicalSize::default());
        assert!(
            layout
                .draws(&menu, LogicalPoint::default(), &theme)
                .rectangles
                .is_empty()
        );
    }
}
