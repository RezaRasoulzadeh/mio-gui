// swap.rs

use crate::{
    Button, ButtonDraws, ButtonLayout, ComponentStyle, Direction, DirectionSetting, FocusPolicy,
    LogicalConstraints, LogicalPoint, ResolvedTheme, SemanticAction, SemanticRole, Semantics,
    TextSystem,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Swap {
    label: String,
    off_content: String,
    on_content: String,
    pub on: bool,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Swap {
    pub fn new(
        label: impl Into<String>,
        off_content: impl Into<String>,
        on_content: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            off_content: off_content.into(),
            on_content: on_content.into(),
            on: false,
            style: ComponentStyle::default(),
            direction: DirectionSetting::Inherit,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn content(&self) -> &str {
        if self.on {
            &self.on_content
        } else {
            &self.off_content
        }
    }

    pub fn activate(&mut self) -> bool {
        if self.style.state.disabled {
            return false;
        }
        self.on = !self.on;
        true
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Switch)
            .with_name(self.label.clone())
            .with_value(self.content().to_owned())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Activate);
        semantics.state.checked = Some(self.on);
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
    ) -> SwapLayout {
        let mut button = Button::new(self.content());
        button.style = self.style;
        button.style.state.selected = self.on;
        button.direction = self.direction;
        SwapLayout {
            button: button.layout(text_system, theme, inherited_direction, constraints),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SwapLayout {
    pub button: ButtonLayout,
}

impl SwapLayout {
    pub fn draws(&self, swap: &Swap, origin: LogicalPoint) -> ButtonDraws {
        let mut button = Button::new(swap.content());
        button.style = swap.style;
        button.style.state.selected = swap.on;
        button.direction = swap.direction;
        self.button.draws(&button, origin)
    }
}

#[cfg(test)]
mod tests {
    use super::Swap;
    use crate::{SemanticAction, SemanticRole};

    #[test]
    fn activation_swaps_content_and_semantic_state() {
        let mut swap = Swap::new("Playback", "Play", "Pause");
        assert_eq!(swap.content(), "Play");
        assert!(swap.activate());
        assert_eq!(swap.content(), "Pause");
        let semantics = swap.semantics();
        assert_eq!(semantics.role, SemanticRole::Switch);
        assert_eq!(semantics.state.checked, Some(true));
        assert!(semantics.supports(SemanticAction::Activate));
    }

    #[test]
    fn disabled_swap_rejects_activation_and_focus() {
        let mut swap = Swap::new("Playback", "Play", "Pause");
        swap.style.state.disabled = true;
        assert!(!swap.activate());
        assert!(swap.focus_policy().disabled);
        assert!(!swap.on);
    }
}
