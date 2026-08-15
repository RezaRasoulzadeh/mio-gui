// dropdown.rs

use crate::{
    Button, ButtonDraws, ButtonLayout, Direction, DirectionSetting, FocusPolicy, ImageDraw, Key,
    KeyState, KeyboardEvent, LogicalConstraints, LogicalPoint, LogicalSize, Menu, MenuAction,
    MenuDraws, MenuLayout, RectDraw, ResolvedTheme, SemanticAction, SemanticRole, Semantics,
    TextDraw, TextSystem,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DropdownAction {
    #[default]
    None,
    Opened,
    Closed,
    ActiveChanged(usize),
    Activated(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dropdown {
    pub trigger: Button,
    pub menu: Menu,
    pub open: bool,
    pub direction: DirectionSetting,
}

impl Dropdown {
    pub fn new(trigger: Button, menu: Menu) -> Self {
        Self {
            trigger,
            menu,
            open: false,
            direction: DirectionSetting::Inherit,
        }
    }

    pub fn handle_key(&mut self, event: &KeyboardEvent) -> DropdownAction {
        if self.trigger.style.state.disabled || event.state != KeyState::Pressed || event.repeat {
            return DropdownAction::None;
        }
        if !self.open {
            return match event.key {
                Key::Enter | Key::Space | Key::Arrow(crate::ArrowKey::Down) => {
                    self.open = true;
                    DropdownAction::Opened
                }
                _ => DropdownAction::None,
            };
        }
        match self.menu.handle_key(event) {
            MenuAction::None => DropdownAction::None,
            MenuAction::ActiveChanged(index) => DropdownAction::ActiveChanged(index),
            MenuAction::Activated(index) => {
                self.open = false;
                DropdownAction::Activated(index)
            }
            MenuAction::Dismissed => {
                self.open = false;
                DropdownAction::Closed
            }
        }
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Button)
            .with_name(self.trigger.label().to_owned())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::ShowMenu);
        semantics.state.disabled = self.trigger.style.state.disabled;
        semantics.state.expanded = Some(self.open);
        if self.open {
            for item in self.menu.semantics().virtual_children() {
                semantics = semantics.with_virtual_child(item.clone());
            }
            semantics.set_active_virtual_child(self.menu.active_index());
        }
        semantics
    }

    pub fn focus_policy(&self) -> FocusPolicy {
        self.trigger.focus_policy()
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> DropdownLayout {
        let direction = self.direction.resolve(inherited_direction);
        let trigger = self.trigger.layout(
            text_system,
            theme,
            direction,
            LogicalConstraints::loose(constraints.max),
        );
        let menu = self.menu.layout(
            text_system,
            theme,
            direction,
            LogicalConstraints::loose(constraints.max),
        );
        let gap = theme.spacing.extra_small;
        let natural = LogicalSize::new(
            trigger
                .size
                .width
                .max(if self.open { menu.size.width } else { 0.0 }),
            trigger.size.height
                + if self.open {
                    gap + menu.size.height
                } else {
                    0.0
                },
        );
        let size = constraints.constrain(natural);
        let logical_start = |child_width: f32| match direction {
            Direction::Ltr => 0.0,
            Direction::Rtl => (size.width - child_width).max(0.0),
        };
        DropdownLayout {
            size,
            direction,
            trigger_origin: LogicalPoint::new(logical_start(trigger.size.width), 0.0),
            trigger,
            menu_origin: LogicalPoint::new(
                logical_start(menu.size.width),
                natural.height - menu.size.height,
            ),
            menu,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DropdownLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub trigger_origin: LogicalPoint,
    pub trigger: ButtonLayout,
    pub menu_origin: LogicalPoint,
    pub menu: MenuLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DropdownDraws {
    pub rectangles: Vec<RectDraw>,
    pub text: Vec<TextDraw>,
    pub images: Vec<ImageDraw>,
}

impl DropdownLayout {
    pub fn draws(
        &self,
        dropdown: &Dropdown,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
    ) -> DropdownDraws {
        let ButtonDraws {
            background,
            text,
            icon,
        } = self
            .trigger
            .draws(&dropdown.trigger, add(origin, self.trigger_origin));
        let mut draws = DropdownDraws {
            rectangles: vec![background],
            text,
            images: icon.into_iter().collect(),
        };
        if dropdown.open {
            let MenuDraws { rectangles, text } =
                self.menu
                    .draws(&dropdown.menu, add(origin, self.menu_origin), theme);
            draws.rectangles.extend(rectangles);
            draws.text.extend(text);
        }
        draws
    }
}

fn add(left: LogicalPoint, right: LogicalPoint) -> LogicalPoint {
    LogicalPoint::new(left.x + right.x, left.y + right.y)
}

#[cfg(test)]
mod tests {
    use super::{Dropdown, DropdownAction};
    use crate::{
        ArrowKey, Button, Direction, Key, KeyboardEvent, LogicalConstraints, LogicalPoint, Menu,
        MenuItem, SemanticAction, TextSystem, ThemeController, ThemeDefinition, UserPreferences,
    };

    fn dropdown() -> Dropdown {
        Dropdown::new(
            Button::new("Actions"),
            Menu::new(
                "Actions",
                vec![MenuItem::new("Open"), MenuItem::new("Delete")],
            )
            .unwrap(),
        )
    }

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn keyboard_opens_delegates_and_closes_after_activation() {
        let mut dropdown = dropdown();
        assert_eq!(
            dropdown.handle_key(&KeyboardEvent::pressed(Key::Arrow(ArrowKey::Down))),
            DropdownAction::Opened
        );
        assert_eq!(
            dropdown.handle_key(&KeyboardEvent::pressed(Key::Arrow(ArrowKey::Down))),
            DropdownAction::ActiveChanged(1)
        );
        assert_eq!(
            dropdown.handle_key(&KeyboardEvent::pressed(Key::Enter)),
            DropdownAction::Activated(1)
        );
        assert!(!dropdown.open);
    }

    #[test]
    fn semantics_expose_trigger_and_expansion() {
        let mut dropdown = dropdown();
        dropdown.open = true;
        let semantics = dropdown.semantics();
        assert_eq!(semantics.name.as_deref(), Some("Actions"));
        assert_eq!(semantics.state.expanded, Some(true));
        assert!(semantics.supports(SemanticAction::ShowMenu));
        assert_eq!(semantics.virtual_children().len(), 2);
        assert_eq!(
            semantics.virtual_children()[0].name.as_deref(),
            Some("Open")
        );
        assert_eq!(semantics.active_virtual_child(), Some(0));
    }

    #[test]
    fn popup_anchors_to_logical_start_and_only_paints_when_open() {
        let mut dropdown = dropdown();
        let theme = theme();
        let mut text_system = TextSystem::new();
        let closed = dropdown.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::unconstrained(),
        );
        assert_eq!(
            closed
                .draws(&dropdown, LogicalPoint::default(), &theme)
                .rectangles
                .len(),
            1
        );
        dropdown.open = true;
        let ltr = dropdown.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let rtl = dropdown.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::unconstrained(),
        );
        assert_eq!(ltr.menu_origin.x, 0.0);
        assert_eq!(rtl.menu_origin.x + rtl.menu.size.width, rtl.size.width);
        assert!(
            ltr.draws(&dropdown, LogicalPoint::default(), &theme)
                .rectangles
                .len()
                > 1
        );
    }
}
