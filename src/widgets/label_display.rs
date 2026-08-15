// label_display.rs

use crate::{
    Button, ButtonDraws, ButtonLayout, ComponentStyle, Direction, DirectionSetting,
    LogicalConstraints, LogicalPoint, ResolvedTheme, SemanticRole, Semantics, TextSystem,
    VisualVariant,
};

macro_rules! label_display {
    ($name:ident, $variant:expr) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            label: String,
            pub style: ComponentStyle,
            pub direction: DirectionSetting,
        }

        impl $name {
            pub fn new(label: impl Into<String>) -> Self {
                let mut style = ComponentStyle::default();
                style.variant = $variant;
                Self {
                    label: label.into(),
                    style,
                    direction: DirectionSetting::Inherit,
                }
            }
            pub fn label(&self) -> &str {
                &self.label
            }
            pub fn set_label(&mut self, label: impl Into<String>) {
                self.label = label.into();
            }
            pub fn semantics(&self) -> Semantics {
                Semantics::new(SemanticRole::Text).with_name(self.label.clone())
            }
            pub fn layout(
                &self,
                text_system: &mut TextSystem,
                theme: &ResolvedTheme,
                direction: Direction,
                constraints: LogicalConstraints,
            ) -> LabelDisplayLayout {
                let mut button = Button::new(self.label.clone());
                button.style = self.style;
                button.direction = self.direction;
                LabelDisplayLayout {
                    button: button.layout(text_system, theme, direction, constraints),
                }
            }
        }
    };
}

label_display!(Badge, VisualVariant::Soft);
label_display!(Kbd, VisualVariant::Outline);

#[derive(Clone, Debug, PartialEq)]
pub struct LabelDisplayLayout {
    pub button: ButtonLayout,
}

impl LabelDisplayLayout {
    pub fn draws(
        &self,
        label: &str,
        style: ComponentStyle,
        direction: DirectionSetting,
        origin: LogicalPoint,
    ) -> ButtonDraws {
        let mut button = Button::new(label);
        button.style = style;
        button.direction = direction;
        self.button.draws(&button, origin)
    }
}

#[cfg(test)]
mod tests {
    use super::{Badge, Kbd};
    use crate::{SemanticRole, VisualVariant};

    #[test]
    fn badge_and_kbd_are_named_static_text() {
        let badge = Badge::new("New");
        let kbd = Kbd::new("Ctrl K");
        assert_eq!(badge.semantics().role, SemanticRole::Text);
        assert_eq!(kbd.semantics().name.as_deref(), Some("Ctrl K"));
        assert_eq!(badge.style.variant, VisualVariant::Soft);
        assert_eq!(kbd.style.variant, VisualVariant::Outline);
    }
}
