// theme_switcher.rs

use crate::{
    Button, ButtonDraws, ButtonLayout, ComponentStyle, Direction, DirectionSetting, FocusPolicy,
    LogicalConstraints, LogicalPoint, ResolvedTheme, SemanticAction, SemanticRole, Semantics,
    TextSystem, ThemeMode,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ThemeSwitcher {
    label: String,
    pub mode: ThemeMode,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl ThemeSwitcher {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            mode: ThemeMode::System,
            style: ComponentStyle::default(),
            direction: DirectionSetting::Inherit,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }

    pub fn activate(&mut self) -> Option<ThemeMode> {
        if self.style.state.disabled {
            return None;
        }
        self.mode = match self.mode {
            ThemeMode::System => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
        };
        Some(self.mode)
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Button)
            .with_name(self.label.clone())
            .with_value(self.mode_label())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Activate);
        semantics.state.disabled = self.style.state.disabled;
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
    ) -> ThemeSwitcherLayout {
        let button = self.button();
        ThemeSwitcherLayout {
            button: button.layout(text_system, theme, inherited_direction, constraints),
        }
    }

    fn button(&self) -> Button {
        let mut button = Button::new(self.mode_label());
        button.style = self.style;
        button.direction = self.direction;
        button
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ThemeSwitcherLayout {
    pub button: ButtonLayout,
}

impl ThemeSwitcherLayout {
    pub fn draws(&self, switcher: &ThemeSwitcher, origin: LogicalPoint) -> ButtonDraws {
        self.button.draws(&switcher.button(), origin)
    }
}

#[cfg(test)]
mod tests {
    use super::ThemeSwitcher;
    use crate::{SemanticAction, SemanticRole, ThemeMode};

    #[test]
    fn activation_cycles_system_light_dark_and_semantics() {
        let mut switcher = ThemeSwitcher::new("Theme");
        assert_eq!(switcher.activate(), Some(ThemeMode::Light));
        assert_eq!(switcher.activate(), Some(ThemeMode::Dark));
        assert_eq!(switcher.activate(), Some(ThemeMode::System));
        let semantics = switcher.semantics();
        assert_eq!(semantics.role, SemanticRole::Button);
        assert_eq!(semantics.value.as_deref(), Some("System"));
        assert!(semantics.supports(SemanticAction::Activate));
    }

    #[test]
    fn disabled_switcher_rejects_activation() {
        let mut switcher = ThemeSwitcher::new("Theme");
        switcher.style.state.disabled = true;
        assert_eq!(switcher.activate(), None);
        assert!(switcher.focus_policy().disabled);
    }
}
