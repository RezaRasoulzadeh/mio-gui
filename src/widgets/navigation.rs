// navigation.rs

use crate::{
    ArrowKey, Button, ButtonDraws, ButtonLayout, ComponentStyle, Direction, DirectionSetting,
    FocusPolicy, Key, KeyState, KeyboardEvent, LogicalConstraints, LogicalPoint, ResolvedTheme,
    SemanticAction, SemanticRole, Semantics, TextSystem, VisualVariant,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    label: String,
    destination: String,
    pub disabled: bool,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Link {
    pub fn new(label: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            destination: destination.into(),
            disabled: false,
            style: ComponentStyle {
                variant: VisualVariant::Ghost,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        }
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn destination(&self) -> &str {
        &self.destination
    }
    pub fn activate(&self) -> bool {
        !self.disabled
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Link)
            .with_name(self.label.clone())
            .with_value(self.destination.clone())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Activate);
        semantics.state.disabled = self.disabled;
        semantics
    }
    pub fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy {
            focusable: true,
            disabled: self.disabled,
            ..FocusPolicy::default()
        }
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> NavigationLayout {
        navigation_layout(
            self.label.clone(),
            self.resolved_style(),
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &NavigationLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.label.clone(),
            self.resolved_style(),
            self.direction,
            origin,
        )
    }
    fn resolved_style(&self) -> ComponentStyle {
        let mut style = self.style;
        style.state.disabled = self.disabled;
        style
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BreadcrumbError;

impl std::fmt::Display for BreadcrumbError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("breadcrumbs require at least one non-empty item")
    }
}

impl std::error::Error for BreadcrumbError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Breadcrumbs {
    items: Vec<String>,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Breadcrumbs {
    pub fn new(
        items: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, BreadcrumbError> {
        let items = items.into_iter().map(Into::into).collect::<Vec<_>>();
        if items.is_empty() || items.iter().any(|item| item.trim().is_empty()) {
            return Err(BreadcrumbError);
        }
        Ok(Self {
            items,
            style: ComponentStyle {
                variant: VisualVariant::Ghost,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        })
    }
    pub fn items(&self) -> &[String] {
        &self.items
    }
    pub fn semantics(&self) -> Semantics {
        self.items.iter().fold(
            Semantics::new(SemanticRole::List).with_name("Breadcrumbs"),
            |semantics, item| {
                semantics
                    .with_virtual_child(Semantics::new(SemanticRole::Link).with_name(item.clone()))
            },
        )
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> NavigationLayout {
        let direction = self.direction.resolve(inherited_direction);
        navigation_layout(
            self.display_text(direction),
            self.style,
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &NavigationLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.display_text(layout.direction),
            self.style,
            self.direction,
            origin,
        )
    }
    fn display_text(&self, direction: Direction) -> String {
        self.items.join(match direction {
            Direction::Ltr => "  ›  ",
            Direction::Rtl => "  ‹  ",
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationLayout {
    pub button: ButtonLayout,
    pub direction: Direction,
}

impl NavigationLayout {
    pub fn draws(
        &self,
        text: String,
        style: ComponentStyle,
        direction: DirectionSetting,
        origin: LogicalPoint,
    ) -> ButtonDraws {
        let mut button = Button::new(text);
        button.style = style;
        button.direction = direction;
        self.button.draws(&button, origin)
    }
}

fn navigation_layout(
    text: String,
    style: ComponentStyle,
    direction: DirectionSetting,
    text_system: &mut TextSystem,
    theme: &ResolvedTheme,
    inherited_direction: Direction,
    constraints: LogicalConstraints,
) -> NavigationLayout {
    let mut button = Button::new(text);
    button.style = style;
    button.direction = direction;
    NavigationLayout {
        button: button.layout(text_system, theme, inherited_direction, constraints),
        direction: direction.resolve(inherited_direction),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NavigationSelectionError;

impl std::fmt::Display for NavigationSelectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("navigation selection requires at least one item")
    }
}

impl std::error::Error for NavigationSelectionError {}

#[derive(Clone, Debug, PartialEq)]
struct SelectionCore {
    label: String,
    items: Vec<String>,
    active: usize,
}

impl SelectionCore {
    fn new(
        label: impl Into<String>,
        items: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, NavigationSelectionError> {
        let items = items.into_iter().map(Into::into).collect::<Vec<_>>();
        if items.is_empty() {
            return Err(NavigationSelectionError);
        }
        Ok(Self {
            label: label.into(),
            items,
            active: 0,
        })
    }
    fn next(&mut self, disabled: bool) -> bool {
        if disabled {
            false
        } else {
            self.active = (self.active + 1) % self.items.len();
            true
        }
    }
    fn previous(&mut self, disabled: bool) -> bool {
        if disabled {
            false
        } else {
            self.active = (self.active + self.items.len() - 1) % self.items.len();
            true
        }
    }
    fn handle_key(&mut self, event: &KeyboardEvent, direction: Direction, disabled: bool) -> bool {
        if event.state != KeyState::Pressed {
            return false;
        }
        match event.key {
            Key::Arrow(ArrowKey::Right) if direction == Direction::Ltr => self.next(disabled),
            Key::Arrow(ArrowKey::Right) => self.previous(disabled),
            Key::Arrow(ArrowKey::Left) if direction == Direction::Ltr => self.previous(disabled),
            Key::Arrow(ArrowKey::Left) => self.next(disabled),
            Key::Home if !disabled => {
                let changed = self.active != 0;
                self.active = 0;
                changed
            }
            Key::End if !disabled => {
                let last = self.items.len() - 1;
                let changed = self.active != last;
                self.active = last;
                changed
            }
            _ => false,
        }
    }
    fn semantics(&self, role: SemanticRole, item_role: SemanticRole, disabled: bool) -> Semantics {
        let mut semantics = self.items.iter().enumerate().fold(
            Semantics::new(role)
                .with_name(self.label.clone())
                .with_value(self.items[self.active].clone())
                .with_action(SemanticAction::Focus)
                .with_action(SemanticAction::Increment)
                .with_action(SemanticAction::Decrement),
            |semantics, (index, item)| {
                let mut child = Semantics::new(item_role).with_name(item.clone());
                child.state.selected = index == self.active;
                semantics.with_virtual_child(child)
            },
        );
        semantics.state.disabled = disabled;
        semantics
    }
    fn display_text(&self, direction: Direction) -> String {
        let separator = match direction {
            Direction::Ltr => "  →  ",
            Direction::Rtl => "  ←  ",
        };
        self.items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if index == self.active {
                    format!("[{item}]")
                } else {
                    item.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(separator)
    }
}

macro_rules! selection_navigation {
    ($name:ident, $role:ident, $item_role:ident) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            core: SelectionCore,
            pub disabled: bool,
            pub style: ComponentStyle,
            pub direction: DirectionSetting,
        }

        impl $name {
            pub fn new(
                label: impl Into<String>,
                items: impl IntoIterator<Item = impl Into<String>>,
            ) -> Result<Self, NavigationSelectionError> {
                Ok(Self {
                    core: SelectionCore::new(label, items)?,
                    disabled: false,
                    style: ComponentStyle::default(),
                    direction: DirectionSetting::Inherit,
                })
            }
            pub fn active_index(&self) -> usize {
                self.core.active
            }
            pub fn active_item(&self) -> &str {
                &self.core.items[self.core.active]
            }
            pub fn next_item(&mut self) -> bool {
                self.core.next(self.disabled)
            }
            pub fn previous_item(&mut self) -> bool {
                self.core.previous(self.disabled)
            }
            pub fn handle_key(&mut self, event: &KeyboardEvent, direction: Direction) -> bool {
                self.core.handle_key(event, direction, self.disabled)
            }
            pub fn semantics(&self) -> Semantics {
                self.core
                    .semantics(SemanticRole::$role, SemanticRole::$item_role, self.disabled)
            }
            pub fn focus_policy(&self) -> FocusPolicy {
                FocusPolicy {
                    focusable: true,
                    disabled: self.disabled,
                    ..FocusPolicy::default()
                }
            }
            pub fn layout(
                &self,
                text_system: &mut TextSystem,
                theme: &ResolvedTheme,
                inherited_direction: Direction,
                constraints: LogicalConstraints,
            ) -> NavigationLayout {
                let direction = self.direction.resolve(inherited_direction);
                navigation_layout(
                    self.core.display_text(direction),
                    self.resolved_style(),
                    self.direction,
                    text_system,
                    theme,
                    inherited_direction,
                    constraints,
                )
            }
            pub fn draws(&self, layout: &NavigationLayout, origin: LogicalPoint) -> ButtonDraws {
                layout.draws(
                    self.core.display_text(layout.direction),
                    self.resolved_style(),
                    self.direction,
                    origin,
                )
            }
            fn resolved_style(&self) -> ComponentStyle {
                let mut style = self.style;
                style.state.disabled = self.disabled;
                style
            }
        }
    };
}

selection_navigation!(Steps, List, ListItem);
selection_navigation!(Tabs, TabList, Tab);
selection_navigation!(Dock, List, ListItem);
selection_navigation!(Navbar, List, ListItem);

#[derive(Clone, Debug, PartialEq)]
pub struct Pagination {
    selection: Steps,
}

impl Pagination {
    pub fn new(
        label: impl Into<String>,
        page_count: usize,
    ) -> Result<Self, NavigationSelectionError> {
        let pages = (1..=page_count).map(|page| page.to_string());
        Ok(Self {
            selection: Steps::new(label, pages)?,
        })
    }
    pub fn selected_page(&self) -> usize {
        self.selection.active_index() + 1
    }
    pub fn next_page(&mut self) -> bool {
        self.selection.next_item()
    }
    pub fn previous_page(&mut self) -> bool {
        self.selection.previous_item()
    }
    pub fn handle_key(&mut self, event: &KeyboardEvent, direction: Direction) -> bool {
        self.selection.handle_key(event, direction)
    }
    pub fn semantics(&self) -> Semantics {
        self.selection.semantics()
    }
    pub fn focus_policy(&self) -> FocusPolicy {
        self.selection.focus_policy()
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> NavigationLayout {
        self.selection
            .layout(text_system, theme, inherited_direction, constraints)
    }
    pub fn draws(&self, layout: &NavigationLayout, origin: LogicalPoint) -> ButtonDraws {
        self.selection.draws(layout, origin)
    }
    pub fn style_mut(&mut self) -> &mut ComponentStyle {
        &mut self.selection.style
    }
}

#[cfg(test)]
mod tests {
    use super::{Breadcrumbs, Dock, Link, Navbar, Pagination, Steps, Tabs};
    use crate::{ArrowKey, Direction, Key, KeyboardEvent, SemanticAction, SemanticRole};

    #[test]
    fn link_and_breadcrumbs_expose_navigation_semantics() {
        let link = Link::new("Settings", "settings");
        assert_eq!(link.semantics().role, SemanticRole::Link);
        assert!(link.semantics().supports(SemanticAction::Activate));
        let breadcrumbs = Breadcrumbs::new(["Home", "Account", "Settings"]).unwrap();
        assert_eq!(breadcrumbs.semantics().role, SemanticRole::List);
        assert_eq!(breadcrumbs.semantics().virtual_children().len(), 3);
        assert!(Breadcrumbs::new(std::iter::empty::<String>()).is_err());
    }

    #[test]
    fn pagination_steps_and_tabs_mirror_selection_navigation() {
        let mut pagination = Pagination::new("Pages", 3).unwrap();
        assert!(pagination.next_page());
        assert_eq!(pagination.selected_page(), 2);
        let mut steps = Steps::new("Checkout", ["Cart", "Address", "Pay"]).unwrap();
        assert!(steps.handle_key(
            &KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right)),
            Direction::Rtl
        ));
        assert_eq!(steps.active_index(), 2);
        let tabs = Tabs::new("Sections", ["Overview", "Activity"]).unwrap();
        assert_eq!(tabs.semantics().role, SemanticRole::TabList);
        assert!(Pagination::new("Empty", 0).is_err());
    }

    #[test]
    fn dock_and_navbar_share_accessible_directional_selection() {
        let mut dock = Dock::new("Primary", ["Home", "Search", "Profile"]).unwrap();
        assert!(dock.next_item());
        assert_eq!(dock.active_item(), "Search");
        let mut navbar = Navbar::new("Site", ["Docs", "Examples"]).unwrap();
        assert!(navbar.handle_key(
            &KeyboardEvent::pressed(Key::Arrow(ArrowKey::Left)),
            Direction::Rtl
        ));
        assert_eq!(navbar.active_item(), "Examples");
        assert_eq!(dock.semantics().role, SemanticRole::List);
    }
}
