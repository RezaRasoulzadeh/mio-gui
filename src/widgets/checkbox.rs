// checkbox.rs

use crate::{
    Direction, DirectionSetting, FocusPolicy, LogicalConstraints, LogicalPoint, LogicalSize,
    RectDraw, ResolvedTheme, SemanticAction, SemanticRole, Semantics, Text, TextDraw, TextStyle,
    TextSystem, TextWrap,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Checkbox {
    label: String,
    pub checked: bool,
    pub disabled: bool,
    pub direction: DirectionSetting,
}

impl Checkbox {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked: false,
            disabled: false,
            direction: DirectionSetting::Inherit,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    pub fn activate(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        self.checked = !self.checked;
        true
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Checkbox)
            .with_name(self.label.clone())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Activate);
        semantics.state.checked = Some(self.checked);
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
    ) -> CheckboxLayout {
        let direction = self.direction.resolve(inherited_direction);
        let indicator_extent = theme.typography.size.max(16.0);
        let gap = theme.spacing.small;
        let mut label = Text::new(self.label.clone());
        label.direction = match direction {
            Direction::Ltr => DirectionSetting::Ltr,
            Direction::Rtl => DirectionSetting::Rtl,
        };
        label.wrap = TextWrap::NoWrap;
        label.style = TextStyle {
            family: Some(theme.typography.family.clone()),
            font_size: theme.typography.size,
            line_height: theme.typography.line_height,
            letter_spacing: theme.typography.letter_spacing,
            weight: theme.typography.weight,
            ..TextStyle::default()
        };
        let reserved = indicator_extent + gap;
        let label = label.layout(
            text_system,
            direction,
            LogicalConstraints::loose(LogicalSize::new(
                subtract(constraints.max.width, reserved),
                constraints.max.height,
            )),
        );
        let size = constraints.constrain(LogicalSize::new(
            reserved + label.size.width,
            indicator_extent.max(label.size.height),
        ));
        let indicator_y = ((size.height - indicator_extent) * 0.5).max(0.0);
        let label_y = ((size.height - label.size.height) * 0.5).max(0.0);
        let (indicator_x, label_x) = match direction {
            Direction::Ltr => (0.0, reserved),
            Direction::Rtl => ((size.width - indicator_extent).max(0.0), 0.0),
        };
        CheckboxLayout {
            size,
            direction,
            indicator_origin: LogicalPoint::new(indicator_x, indicator_y),
            indicator_extent,
            label,
            label_origin: LogicalPoint::new(label_x, label_y),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckboxLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub indicator_origin: LogicalPoint,
    pub indicator_extent: f32,
    pub label: crate::TextLayout,
    pub label_origin: LogicalPoint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckboxDraws {
    pub indicator: Vec<RectDraw>,
    pub label: Vec<TextDraw>,
}

impl CheckboxLayout {
    pub fn draws(
        &self,
        checkbox: &Checkbox,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
    ) -> CheckboxDraws {
        let opacity = if checkbox.disabled { 0.45 } else { 1.0 };
        let indicator_origin = add(origin, self.indicator_origin);
        let fill = if checkbox.checked {
            theme.colors.primary
        } else {
            theme.colors.surface
        };
        let mut indicator = vec![RectDraw {
            position: [indicator_origin.x, indicator_origin.y],
            size: [self.indicator_extent, self.indicator_extent],
            radii: [theme.radii.small; 4],
            color: faded(fill.to_array(), opacity),
            border_width: theme.borders.thin,
            border_color: faded(
                if checkbox.checked {
                    theme.colors.primary.to_array()
                } else {
                    theme.colors.border.to_array()
                },
                opacity,
            ),
        }];
        if checkbox.checked {
            let inset = self.indicator_extent * 0.3;
            indicator.push(RectDraw {
                position: [indicator_origin.x + inset, indicator_origin.y + inset],
                size: [self.indicator_extent - inset * 2.0; 2],
                radii: [theme.radii.small; 4],
                color: faded(theme.colors.on_primary.to_array(), opacity),
                border_width: 0.0,
                border_color: [0.0; 4],
            });
        }
        CheckboxDraws {
            indicator,
            label: self.label.draws(
                checkbox.label(),
                add(origin, self.label_origin),
                faded(theme.colors.text.to_array(), opacity),
            ),
        }
    }
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

fn faded(mut color: [f32; 4], opacity: f32) -> [f32; 4] {
    color[3] *= opacity;
    color
}

#[cfg(test)]
mod tests {
    use super::Checkbox;
    use crate::{
        Direction, LogicalConstraints, LogicalPoint, SemanticAction, SemanticRole, TextSystem,
        ThemeController, ThemeDefinition, UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn activation_toggles_only_enabled_checkboxes() {
        let mut checkbox = Checkbox::new("Remember me");
        assert!(checkbox.activate());
        assert!(checkbox.checked);
        checkbox.disabled = true;
        assert!(!checkbox.activate());
        assert!(checkbox.checked);
    }

    #[test]
    fn semantics_expose_checked_disabled_and_activation_state() {
        let mut checkbox = Checkbox::new("Remember me");
        checkbox.checked = true;
        checkbox.disabled = true;
        let semantics = checkbox.semantics();
        assert_eq!(semantics.role, SemanticRole::Checkbox);
        assert_eq!(semantics.name.as_deref(), Some("Remember me"));
        assert_eq!(semantics.state.checked, Some(true));
        assert!(semantics.state.disabled);
        assert!(semantics.supports(SemanticAction::Activate));
        assert!(checkbox.focus_policy().disabled);
    }

    #[test]
    fn rtl_layout_places_the_indicator_after_the_label() {
        let checkbox = Checkbox::new("Remember me");
        let mut text_system = TextSystem::new();
        let theme = theme();
        let ltr = checkbox.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let rtl = checkbox.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::unconstrained(),
        );
        assert!(ltr.indicator_origin.x < ltr.label_origin.x);
        assert!(rtl.indicator_origin.x > rtl.label_origin.x);
        assert_eq!(ltr.size, rtl.size);
    }

    #[test]
    fn checked_paint_uses_primary_fill_and_visible_mark() {
        let mut checkbox = Checkbox::new("Remember me");
        checkbox.checked = true;
        let mut text_system = TextSystem::new();
        let theme = theme();
        let layout = checkbox.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let draws = layout.draws(&checkbox, LogicalPoint::new(4.0, 8.0), &theme);
        assert_eq!(draws.indicator.len(), 2);
        assert_eq!(draws.indicator[0].color, theme.colors.primary.to_array());
        assert_eq!(draws.label.len(), 1);
    }
}
