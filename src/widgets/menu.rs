// menu.rs

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    ArrowKey, Direction, DirectionSetting, FocusPolicy, Key, KeyState, KeyboardEvent,
    LogicalConstraints, LogicalPoint, LogicalSize, RectDraw, ResolvedTheme, SemanticAction,
    SemanticRole, Semantics, Text, TextDraw, TextStyle, TextSystem, TextWrap,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuItem {
    pub label: String,
    pub disabled: bool,
    pub selected: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            selected: false,
        }
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::MenuItem)
            .with_name(self.label.clone())
            .with_action(SemanticAction::Focus);
        if !self.disabled {
            semantics.add_action(SemanticAction::Activate);
        }
        semantics.state.disabled = self.disabled;
        semantics.state.selected = self.selected;
        semantics
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuError {
    EmptyItems,
    NoEnabledItems,
}

impl Display for MenuError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyItems => formatter.write_str("menu requires at least one item"),
            Self::NoEnabledItems => formatter.write_str("menu requires an enabled item"),
        }
    }
}

impl Error for MenuError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    None,
    ActiveChanged(usize),
    Activated(usize),
    Dismissed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    label: String,
    items: Vec<MenuItem>,
    active: usize,
    pub direction: DirectionSetting,
}

impl Menu {
    pub fn new(label: impl Into<String>, items: Vec<MenuItem>) -> Result<Self, MenuError> {
        if items.is_empty() {
            return Err(MenuError::EmptyItems);
        }
        let active = items
            .iter()
            .position(|item| !item.disabled)
            .ok_or(MenuError::NoEnabledItems)?;
        Ok(Self {
            label: label.into(),
            items,
            active,
            direction: DirectionSetting::Inherit,
        })
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn activate(&mut self, index: usize) -> MenuAction {
        if self.items.get(index).is_none_or(|item| item.disabled) {
            return MenuAction::None;
        }
        self.active = index;
        MenuAction::Activated(index)
    }

    pub fn handle_key(&mut self, event: &KeyboardEvent) -> MenuAction {
        if event.state != KeyState::Pressed || event.repeat {
            return MenuAction::None;
        }
        match event.key {
            Key::Arrow(ArrowKey::Down) => self.move_active(true),
            Key::Arrow(ArrowKey::Up) => self.move_active(false),
            Key::Home => self.select_edge(true),
            Key::End => self.select_edge(false),
            Key::Enter | Key::Space => MenuAction::Activated(self.active),
            Key::Escape => MenuAction::Dismissed,
            _ => MenuAction::None,
        }
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = self.items.iter().fold(
            Semantics::new(SemanticRole::Menu).with_name(self.label.clone()),
            |semantics, item| semantics.with_virtual_child(item.semantics()),
        );
        semantics.set_active_virtual_child(self.active);
        semantics
    }

    pub fn item_semantics(&self, index: usize) -> Option<Semantics> {
        self.items.get(index).map(MenuItem::semantics)
    }

    pub fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy::focusable()
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> MenuLayout {
        let direction = self.direction.resolve(inherited_direction);
        let padding = theme.spacing.small;
        let item_height = theme.typography.line_height + padding * 2.0;
        let labels = self
            .items
            .iter()
            .map(|item| {
                let mut text = Text::new(item.label.clone());
                text.direction = match direction {
                    Direction::Ltr => DirectionSetting::Ltr,
                    Direction::Rtl => DirectionSetting::Rtl,
                };
                text.wrap = TextWrap::NoWrap;
                text.style = TextStyle {
                    family: Some(theme.typography.family.clone()),
                    font_size: theme.typography.size,
                    line_height: theme.typography.line_height,
                    letter_spacing: theme.typography.letter_spacing,
                    weight: theme.typography.weight,
                    ..TextStyle::default()
                };
                text.layout(text_system, direction, LogicalConstraints::unconstrained())
            })
            .collect::<Vec<_>>();
        let natural_width = labels
            .iter()
            .map(|label| label.size.width)
            .fold(0.0_f32, f32::max)
            + padding * 4.0;
        let size = constraints.constrain(LogicalSize::new(
            natural_width.max(120.0),
            item_height * self.items.len() as f32 + padding * 2.0,
        ));
        MenuLayout {
            size,
            direction,
            item_height,
            padding,
            labels,
        }
    }

    fn move_active(&mut self, forward: bool) -> MenuAction {
        let count = self.items.len();
        let mut index = self.active;
        for _ in 0..count {
            index = if forward {
                (index + 1) % count
            } else {
                (index + count - 1) % count
            };
            if !self.items[index].disabled {
                self.active = index;
                return MenuAction::ActiveChanged(index);
            }
        }
        MenuAction::None
    }

    fn select_edge(&mut self, first: bool) -> MenuAction {
        let active = if first {
            self.items.iter().position(|item| !item.disabled)
        } else {
            self.items.iter().rposition(|item| !item.disabled)
        }
        .unwrap();
        self.active = active;
        MenuAction::ActiveChanged(active)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub item_height: f32,
    pub padding: f32,
    pub labels: Vec<crate::TextLayout>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuDraws {
    pub rectangles: Vec<RectDraw>,
    pub text: Vec<TextDraw>,
}

impl MenuLayout {
    pub fn item_bounds(&self, origin: LogicalPoint, index: usize) -> Option<crate::LogicalRect> {
        (index < self.labels.len()).then(|| {
            crate::LogicalRect::from_xywh(
                origin.x,
                origin.y + self.padding + index as f32 * self.item_height,
                self.size.width,
                self.item_height,
            )
        })
    }

    pub fn hit_test(&self, origin: LogicalPoint, point: LogicalPoint) -> Option<usize> {
        let local_x = point.x - origin.x;
        let local_y = point.y - origin.y - self.padding;
        if local_x < 0.0
            || local_x >= self.size.width
            || local_y < 0.0
            || local_y >= self.item_height * self.labels.len() as f32
        {
            return None;
        }
        Some((local_y / self.item_height) as usize)
    }

    pub fn draws(&self, menu: &Menu, origin: LogicalPoint, theme: &ResolvedTheme) -> MenuDraws {
        let mut rectangles = vec![RectDraw {
            position: [origin.x, origin.y],
            size: [self.size.width, self.size.height],
            radii: [theme.radii.medium; 4],
            color: theme.colors.surface_elevated.to_array(),
            border_width: theme.borders.thin,
            border_color: theme.colors.border.to_array(),
        }];
        let mut text = Vec::with_capacity(menu.items.len());
        for (index, (item, label)) in menu.items.iter().zip(&self.labels).enumerate() {
            let y = origin.y + self.padding + index as f32 * self.item_height;
            if index == menu.active || item.selected {
                let mut color = theme.colors.primary.to_array();
                color[3] *= if index == menu.active { 0.16 } else { 0.1 };
                rectangles.push(RectDraw {
                    position: [origin.x + self.padding, y],
                    size: [self.size.width - self.padding * 2.0, self.item_height],
                    radii: [theme.radii.small; 4],
                    color,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                });
            }
            let x = match self.direction {
                Direction::Ltr => origin.x + self.padding * 2.0,
                Direction::Rtl => {
                    origin.x + self.size.width - self.padding * 2.0 - label.size.width
                }
            };
            let mut color = theme.colors.text.to_array();
            if item.disabled {
                color[3] *= 0.45;
            }
            text.extend(label.draws(&item.label, LogicalPoint::new(x, y + self.padding), color));
        }
        MenuDraws { rectangles, text }
    }
}

#[cfg(test)]
mod tests {
    use super::{Menu, MenuAction, MenuError, MenuItem};
    use crate::{
        ArrowKey, Direction, Key, KeyboardEvent, LogicalConstraints, LogicalPoint, SemanticAction,
        SemanticRole, TextSystem, ThemeController, ThemeDefinition, UserPreferences,
    };

    fn items() -> Vec<MenuItem> {
        vec![
            MenuItem::new("Open"),
            MenuItem {
                disabled: true,
                ..MenuItem::new("Rename")
            },
            MenuItem::new("Delete"),
        ]
    }

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn validates_items_and_exposes_item_semantics() {
        assert_eq!(Menu::new("Actions", Vec::new()), Err(MenuError::EmptyItems));
        let menu = Menu::new("Actions", items()).unwrap();
        let semantics = menu.semantics();
        assert_eq!(semantics.role, SemanticRole::Menu);
        assert_eq!(semantics.virtual_children().len(), 3);
        assert_eq!(semantics.virtual_children()[0].role, SemanticRole::MenuItem);
        assert_eq!(semantics.active_virtual_child(), Some(0));
        let disabled = menu.item_semantics(1).unwrap();
        assert_eq!(disabled.role, SemanticRole::MenuItem);
        assert!(disabled.state.disabled);
        assert!(!disabled.supports(SemanticAction::Activate));
    }

    #[test]
    fn keyboard_roving_focus_wraps_and_skips_disabled_items() {
        let mut menu = Menu::new("Actions", items()).unwrap();
        assert_eq!(
            menu.handle_key(&KeyboardEvent::pressed(Key::Arrow(ArrowKey::Down))),
            MenuAction::ActiveChanged(2)
        );
        assert_eq!(
            menu.handle_key(&KeyboardEvent::pressed(Key::Arrow(ArrowKey::Down))),
            MenuAction::ActiveChanged(0)
        );
        assert_eq!(
            menu.handle_key(&KeyboardEvent::pressed(Key::Enter)),
            MenuAction::Activated(0)
        );
        assert_eq!(
            menu.handle_key(&KeyboardEvent::pressed(Key::Escape)),
            MenuAction::Dismissed
        );
    }

    #[test]
    fn pointer_hit_testing_activates_enabled_items_only() {
        let mut menu = Menu::new("Actions", items()).unwrap();
        let theme = theme();
        let mut text_system = TextSystem::new();
        let layout = menu.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let origin = LogicalPoint::new(10.0, 20.0);
        let item = |index| {
            LogicalPoint::new(
                origin.x + layout.padding,
                origin.y + layout.padding + layout.item_height * (index as f32 + 0.5),
            )
        };

        assert_eq!(layout.hit_test(origin, item(2)), Some(2));
        assert_eq!(menu.activate(2), MenuAction::Activated(2));
        assert_eq!(menu.activate(1), MenuAction::None);
        assert_eq!(layout.hit_test(origin, LogicalPoint::new(0.0, 0.0)), None);
    }

    #[test]
    fn menu_layout_and_text_alignment_mirror_in_rtl() {
        let menu = Menu::new("Actions", items()).unwrap();
        let theme = theme();
        let mut text_system = TextSystem::new();
        let ltr = menu.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let rtl = menu.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::unconstrained(),
        );
        let ltr_draws = ltr.draws(&menu, LogicalPoint::default(), &theme);
        let rtl_draws = rtl.draws(&menu, LogicalPoint::default(), &theme);
        assert_eq!(ltr.size, rtl.size);
        assert!(ltr_draws.text[0].baseline[0] < rtl_draws.text[0].baseline[0]);
        assert_eq!(ltr_draws.rectangles.len(), 2);
    }
}
