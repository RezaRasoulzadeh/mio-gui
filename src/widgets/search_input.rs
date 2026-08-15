// search_input.rs

use crate::{
    Direction, FocusPolicy, Icon, IconLayout, ImageDraw, Key, KeyState, KeyboardEvent,
    LogicalConstraints, LogicalPoint, LogicalSize, PixelFormat, PixelImage, ResolvedTheme,
    SemanticRole, Semantics, TextInput, TextInputDraws, TextInputLayout, TextSystem,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SearchInputAction {
    #[default]
    None,
    Edited,
    Cleared,
    Submitted,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchInput {
    pub input: TextInput,
    pub icon: Icon,
}

impl SearchInput {
    pub fn new(label: impl Into<String>) -> Self {
        let mut input = TextInput::new(label);
        input.set_placeholder("Search");
        Self {
            input,
            icon: search_icon(),
        }
    }

    pub fn with_text(label: impl Into<String>, text: impl Into<String>) -> Self {
        let mut search = Self::new(label);
        search.input.edit = crate::TextEditState::new(text);
        search
    }

    pub fn clear(&mut self) -> bool {
        if self.input.disabled || self.input.read_only || self.input.text().is_empty() {
            return false;
        }
        self.input.edit.select_all();
        self.input.edit.replace_selection("");
        true
    }

    pub fn handle_key(&mut self, event: &KeyboardEvent) -> SearchInputAction {
        if event.state == KeyState::Pressed && event.key == Key::Enter && !self.input.disabled {
            return SearchInputAction::Submitted;
        }
        if event.state == KeyState::Pressed && event.key == Key::Escape {
            return if self.clear() {
                SearchInputAction::Cleared
            } else {
                SearchInputAction::None
            };
        }
        if self.input.handle_key(event) {
            SearchInputAction::Edited
        } else {
            SearchInputAction::None
        }
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = self.input.semantics();
        semantics.role = SemanticRole::SearchField;
        semantics
    }

    pub fn focus_policy(&self) -> FocusPolicy {
        self.input.focus_policy()
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> SearchInputLayout {
        let direction = self.input.direction.resolve(inherited_direction);
        let icon_extent = theme.typography.size.max(16.0);
        let gap = theme.spacing.small;
        let reserved = icon_extent + gap;
        let input_constraints = LogicalConstraints::new(
            LogicalSize::new(
                subtract(constraints.min.width, reserved),
                constraints.min.height,
            ),
            LogicalSize::new(
                subtract(constraints.max.width, reserved),
                constraints.max.height,
            ),
        );
        let input = self
            .input
            .layout(text_system, theme, direction, input_constraints);
        let size = constraints.constrain(LogicalSize::new(
            input.size.width + reserved,
            input.size.height,
        ));
        let icon_origin = match direction {
            Direction::Ltr => {
                LogicalPoint::new(theme.spacing.medium, (size.height - icon_extent) * 0.5)
            }
            Direction::Rtl => LogicalPoint::new(
                size.width - theme.spacing.medium - icon_extent,
                (size.height - icon_extent) * 0.5,
            ),
        };
        SearchInputLayout {
            size,
            direction,
            input,
            input_offset: match direction {
                Direction::Ltr => LogicalPoint::new(reserved, 0.0),
                Direction::Rtl => LogicalPoint::default(),
            },
            icon: self.icon.layout(
                direction,
                LogicalConstraints::tight(LogicalSize::new(icon_extent, icon_extent)),
            ),
            icon_origin,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchInputLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub input: TextInputLayout,
    pub input_offset: LogicalPoint,
    pub icon: IconLayout,
    pub icon_origin: LogicalPoint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchInputDraws {
    pub input: TextInputDraws,
    pub icon: ImageDraw,
}

impl SearchInputLayout {
    pub fn draws(
        &self,
        search: &SearchInput,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
    ) -> SearchInputDraws {
        let mut input = self
            .input
            .draws(&search.input, add(origin, self.input_offset), theme);
        input.background.position = [origin.x, origin.y];
        input.background.size = [self.size.width, self.size.height];
        SearchInputDraws {
            input,
            icon: self.icon.draw(
                search.icon.source.clone(),
                add(origin, self.icon_origin),
                theme.colors.text_muted.to_array(),
            ),
        }
    }
}

fn search_icon() -> Icon {
    Icon::new(
        PixelImage::new(
            5,
            5,
            PixelFormat::Alpha8,
            vec![
                0, 255, 255, 255, 0, 255, 0, 0, 0, 255, 255, 0, 255, 0, 255, 255, 0, 0, 255, 0, 0,
                255, 255, 0, 255,
            ],
        )
        .expect("built-in search icon pixels are valid"),
    )
    .expect("built-in search icon uses an alpha mask")
}

fn add(left: LogicalPoint, right: LogicalPoint) -> LogicalPoint {
    LogicalPoint::new(left.x + right.x, left.y + right.y)
}

fn subtract(value: f32, amount: f32) -> f32 {
    if value.is_finite() {
        (value - amount).max(0.0)
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{SearchInput, SearchInputAction};
    use crate::{
        Direction, Key, KeyboardEvent, LogicalConstraints, LogicalPoint, TextSystem,
        ThemeController, ThemeDefinition, UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn reports_edit_submit_and_clear_actions() {
        let mut search = SearchInput::new("Site search");
        assert_eq!(
            search.handle_key(&KeyboardEvent::pressed(Key::Character("mio".into()))),
            SearchInputAction::Edited
        );
        assert_eq!(
            search.handle_key(&KeyboardEvent::pressed(Key::Enter)),
            SearchInputAction::Submitted
        );
        assert_eq!(
            search.handle_key(&KeyboardEvent::pressed(Key::Escape)),
            SearchInputAction::Cleared
        );
        assert_eq!(search.input.text(), "");
    }

    #[test]
    fn read_only_search_cannot_clear_or_edit() {
        let mut search = SearchInput::with_text("Site search", "Mio");
        search.input.read_only = true;
        assert!(!search.clear());
        assert_eq!(
            search.handle_key(&KeyboardEvent::pressed(Key::Character("!".into()))),
            SearchInputAction::None
        );
    }

    #[test]
    fn search_adornment_moves_between_logical_starts() {
        let search = SearchInput::new("Site search");
        let theme = theme();
        let mut text_system = TextSystem::new();
        let ltr = search.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let rtl = search.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::unconstrained(),
        );
        assert!(ltr.icon_origin.x < ltr.input.text_origin.x + ltr.input_offset.x);
        assert!(rtl.icon_origin.x > rtl.input.text_origin.x);
        let draws = rtl.draws(&search, LogicalPoint::default(), &theme);
        assert_eq!(draws.icon.tint, Some(theme.colors.text_muted.to_array()));
        assert_eq!(draws.input.background.size[0], rtl.size.width);
    }
}
