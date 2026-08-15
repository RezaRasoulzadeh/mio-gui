// select.rs

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    AdornmentPlacement, ArrowKey, Button, ButtonLayout, ComponentStyle, Direction,
    DirectionSetting, FocusPolicy, Icon, Key, KeyState, KeyboardEvent, LogicalConstraints,
    LogicalPoint, LogicalSize, Menu, MenuDraws, MenuItem, MenuLayout, PixelFormat, PixelImage,
    ResolvedTheme, SemanticAction, SemanticRole, Semantics, TextSystem, VisualVariant,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            disabled: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectError {
    EmptyOptions,
    NoEnabledOptions,
    InvalidSelection,
}

impl Display for SelectError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyOptions => formatter.write_str("select requires at least one option"),
            Self::NoEnabledOptions => formatter.write_str("select requires an enabled option"),
            Self::InvalidSelection => {
                formatter.write_str("select index is out of range or disabled")
            }
        }
    }
}

impl Error for SelectError {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SelectAction {
    #[default]
    None,
    Opened,
    Closed,
    Changed,
    Submitted,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Select {
    label: String,
    options: Vec<SelectOption>,
    selected: usize,
    pub open: bool,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Select {
    pub fn new(label: impl Into<String>, options: Vec<SelectOption>) -> Result<Self, SelectError> {
        if options.is_empty() {
            return Err(SelectError::EmptyOptions);
        }
        let selected = options
            .iter()
            .position(|option| !option.disabled)
            .ok_or(SelectError::NoEnabledOptions)?;
        let style = ComponentStyle {
            variant: VisualVariant::Outline,
            ..ComponentStyle::default()
        };
        Ok(Self {
            label: label.into(),
            options,
            selected,
            open: false,
            style,
            direction: DirectionSetting::Inherit,
        })
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn options(&self) -> &[SelectOption] {
        &self.options
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected(&self) -> &SelectOption {
        &self.options[self.selected]
    }

    pub fn select(&mut self, index: usize) -> Result<bool, SelectError> {
        if self.options.get(index).is_none_or(|option| option.disabled) {
            return Err(SelectError::InvalidSelection);
        }
        let changed = self.selected != index;
        self.selected = index;
        Ok(changed)
    }

    pub fn handle_key(&mut self, event: &KeyboardEvent) -> SelectAction {
        if self.style.state.disabled || event.state != KeyState::Pressed || event.repeat {
            return SelectAction::None;
        }
        match event.key {
            Key::Enter | Key::Space if self.open => {
                self.open = false;
                SelectAction::Submitted
            }
            Key::Enter | Key::Space => {
                self.open = true;
                SelectAction::Opened
            }
            Key::Escape if self.open => {
                self.open = false;
                SelectAction::Closed
            }
            Key::Arrow(ArrowKey::Down) => self.move_selection(true),
            Key::Arrow(ArrowKey::Up) => self.move_selection(false),
            Key::Home => self.select_edge(true),
            Key::End => self.select_edge(false),
            _ => SelectAction::None,
        }
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::ComboBox)
            .with_name(self.label.clone())
            .with_value(self.selected().label.clone())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::ShowMenu);
        semantics.state.disabled = self.style.state.disabled;
        semantics.state.expanded = Some(self.open);
        if self.open {
            for (index, option) in self.options.iter().enumerate() {
                let mut child = Semantics::new(SemanticRole::ListBoxOption)
                    .with_name(option.label.clone())
                    .with_action(SemanticAction::Focus);
                if !option.disabled {
                    child.add_action(SemanticAction::Activate);
                }
                child.state.disabled = option.disabled;
                child.state.selected = index == self.selected;
                semantics = semantics.with_virtual_child(child);
            }
            semantics.set_active_virtual_child(self.selected);
        }
        semantics
    }

    pub fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy {
            focusable: true,
            disabled: self.style.state.disabled,
            ..FocusPolicy::default()
        }
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> SelectLayout {
        let direction = self.direction.resolve(inherited_direction);
        let trigger = self.button().layout(
            text_system,
            theme,
            direction,
            LogicalConstraints::loose(constraints.max),
        );
        let menu = self.menu().layout(
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
        let start = |width: f32| match direction {
            Direction::Ltr => 0.0,
            Direction::Rtl => (size.width - width).max(0.0),
        };
        SelectLayout {
            size,
            trigger_origin: LogicalPoint::new(start(trigger.size.width), 0.0),
            trigger,
            menu_origin: LogicalPoint::new(
                start(menu.size.width),
                natural.height - menu.size.height,
            ),
            menu,
        }
    }

    pub fn draws(
        &self,
        layout: &SelectLayout,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
    ) -> SelectDraws {
        let trigger = layout
            .trigger
            .draws(&self.button(), add(origin, layout.trigger_origin));
        let mut draws = SelectDraws {
            rectangles: vec![trigger.background],
            text: trigger.text,
            images: trigger.icon.into_iter().collect(),
        };
        if self.open {
            let menu = self.menu();
            let MenuDraws { rectangles, text } =
                layout
                    .menu
                    .draws(&menu, add(origin, layout.menu_origin), theme);
            draws.rectangles.extend(rectangles);
            draws.text.extend(text);
        }
        draws
    }

    fn button(&self) -> Button {
        let mut button = Button::new(self.selected().label.clone());
        button.style = self.style;
        button.direction = self.direction;
        button.with_icon(chevron_icon(), AdornmentPlacement::InlineEnd)
    }

    fn menu(&self) -> Menu {
        let items = self
            .options
            .iter()
            .enumerate()
            .map(|(index, option)| MenuItem {
                label: option.label.clone(),
                disabled: option.disabled,
                selected: index == self.selected,
            })
            .collect();
        let mut menu = Menu::new(self.label.clone(), items).unwrap();
        menu.activate(self.selected);
        menu
    }

    fn move_selection(&mut self, forward: bool) -> SelectAction {
        let mut index = self.selected;
        loop {
            let next = if forward {
                index
                    .checked_add(1)
                    .filter(|next| *next < self.options.len())
            } else {
                index.checked_sub(1)
            };
            let Some(next) = next else {
                return SelectAction::None;
            };
            index = next;
            if !self.options[index].disabled {
                self.selected = index;
                return SelectAction::Changed;
            }
        }
    }

    fn select_edge(&mut self, first: bool) -> SelectAction {
        let selected = if first {
            self.options.iter().position(|option| !option.disabled)
        } else {
            self.options.iter().rposition(|option| !option.disabled)
        };
        let Some(selected) = selected else {
            return SelectAction::None;
        };
        if selected == self.selected {
            SelectAction::None
        } else {
            self.selected = selected;
            SelectAction::Changed
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectLayout {
    pub size: LogicalSize,
    pub trigger_origin: LogicalPoint,
    pub trigger: ButtonLayout,
    pub menu_origin: LogicalPoint,
    pub menu: MenuLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectDraws {
    pub rectangles: Vec<crate::RectDraw>,
    pub text: Vec<crate::TextDraw>,
    pub images: Vec<crate::ImageDraw>,
}

fn add(left: LogicalPoint, right: LogicalPoint) -> LogicalPoint {
    LogicalPoint::new(left.x + right.x, left.y + right.y)
}

fn chevron_icon() -> Icon {
    Icon::new(
        PixelImage::new(
            5,
            3,
            PixelFormat::Alpha8,
            vec![255, 0, 0, 0, 255, 0, 255, 0, 255, 0, 0, 0, 255, 0, 0],
        )
        .expect("built-in select icon pixels are valid"),
    )
    .expect("built-in select icon uses an alpha mask")
}

#[cfg(test)]
mod tests {
    use super::{Select, SelectAction, SelectError, SelectOption};
    use crate::{
        ArrowKey, Direction, Key, KeyboardEvent, LogicalConstraints, LogicalPoint,
        PhysicalAdornmentPlacement, SemanticAction, SemanticRole, TextSystem, ThemeController,
        ThemeDefinition, UserPreferences,
    };

    fn options() -> Vec<SelectOption> {
        vec![
            SelectOption::new("Small", "sm"),
            SelectOption {
                disabled: true,
                ..SelectOption::new("Medium", "md")
            },
            SelectOption::new("Large", "lg"),
        ]
    }

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn validates_options_and_selection() {
        assert_eq!(
            Select::new("Size", Vec::new()),
            Err(SelectError::EmptyOptions)
        );
        let mut select = Select::new("Size", options()).unwrap();
        assert_eq!(select.select(1), Err(SelectError::InvalidSelection));
        assert_eq!(select.select(2), Ok(true));
        assert_eq!(select.selected().value, "lg");
    }

    #[test]
    fn keyboard_navigation_skips_disabled_options_and_controls_popup() {
        let mut select = Select::new("Size", options()).unwrap();
        assert_eq!(
            select.handle_key(&KeyboardEvent::pressed(Key::Enter)),
            SelectAction::Opened
        );
        assert_eq!(
            select.handle_key(&KeyboardEvent::pressed(Key::Arrow(ArrowKey::Down))),
            SelectAction::Changed
        );
        assert_eq!(select.selected_index(), 2);
        assert_eq!(
            select.handle_key(&KeyboardEvent::pressed(Key::Escape)),
            SelectAction::Closed
        );
    }

    #[test]
    fn semantics_expose_label_value_expansion_and_menu_action() {
        let mut select = Select::new("Size", options()).unwrap();
        select.open = true;
        let semantics = select.semantics();
        assert_eq!(semantics.role, SemanticRole::ComboBox);
        assert_eq!(semantics.name.as_deref(), Some("Size"));
        assert_eq!(semantics.value.as_deref(), Some("Small"));
        assert_eq!(semantics.state.expanded, Some(true));
        assert!(semantics.supports(SemanticAction::ShowMenu));
        assert_eq!(semantics.virtual_children().len(), 3);
        assert_eq!(
            semantics.virtual_children()[0].role,
            SemanticRole::ListBoxOption
        );
        assert!(semantics.virtual_children()[0].state.selected);
        assert!(semantics.virtual_children()[1].state.disabled);
        assert_eq!(semantics.active_virtual_child(), Some(0));
    }

    #[test]
    fn select_adornment_mirrors_with_direction() {
        let select = Select::new("Size", options()).unwrap();
        let theme = theme();
        let mut text_system = TextSystem::new();
        let ltr = select.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let rtl = select.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::unconstrained(),
        );
        assert_eq!(
            ltr.trigger.component.adornment,
            PhysicalAdornmentPlacement::Right
        );
        assert_eq!(
            rtl.trigger.component.adornment,
            PhysicalAdornmentPlacement::Left
        );
        assert!(ltr.trigger.icon_origin.unwrap().x > ltr.trigger.label_origin.x);
        assert!(rtl.trigger.icon_origin.unwrap().x < rtl.trigger.label_origin.x);
        assert_eq!(
            select
                .draws(&rtl, LogicalPoint::default(), &theme)
                .images
                .len(),
            1
        );
    }

    #[test]
    fn open_select_lays_out_paints_and_hit_tests_options() {
        let mut select = Select::new("Size", options()).unwrap();
        select.open = true;
        let theme = theme();
        let mut text_system = TextSystem::new();
        let layout = select.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let origin = LogicalPoint::new(10.0, 20.0);
        let draws = select.draws(&layout, origin, &theme);
        assert!(layout.size.height > layout.trigger.size.height);
        assert_eq!(draws.text.len(), 1 + select.options().len());
        assert!(draws.rectangles.len() >= 3);
        let option = LogicalPoint::new(
            origin.x + layout.menu_origin.x + layout.menu.padding,
            origin.y + layout.menu_origin.y + layout.menu.padding + layout.menu.item_height * 2.5,
        );
        assert_eq!(
            layout.menu.hit_test(
                LogicalPoint::new(
                    origin.x + layout.menu_origin.x,
                    origin.y + layout.menu_origin.y,
                ),
                option,
            ),
            Some(2)
        );
    }
}
