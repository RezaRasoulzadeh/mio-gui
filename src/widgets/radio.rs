// radio.rs

use crate::{
    Direction, DirectionSetting, FocusPolicy, LogicalConstraints, LogicalPoint, LogicalSize,
    RectDraw, ResolvedTheme, SemanticAction, SemanticRole, Semantics, Text, TextDraw, TextStyle,
    TextSystem, TextWrap,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Radio {
    label: String,
    pub selected: bool,
    pub disabled: bool,
    pub direction: DirectionSetting,
}

impl Radio {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            selected: false,
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
        if self.disabled || self.selected {
            return false;
        }
        self.selected = true;
        true
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Radio)
            .with_name(self.label.clone())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Activate);
        semantics.state.checked = Some(self.selected);
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
    ) -> RadioLayout {
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
        RadioLayout {
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
pub struct RadioLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub indicator_origin: LogicalPoint,
    pub indicator_extent: f32,
    pub label: crate::TextLayout,
    pub label_origin: LogicalPoint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadioDraws {
    pub indicator: Vec<RectDraw>,
    pub label: Vec<TextDraw>,
}

impl RadioLayout {
    pub fn draws(&self, radio: &Radio, origin: LogicalPoint, theme: &ResolvedTheme) -> RadioDraws {
        let opacity = if radio.disabled { 0.45 } else { 1.0 };
        let indicator_origin = add(origin, self.indicator_origin);
        let radius = self.indicator_extent * 0.5;
        let mut indicator = vec![RectDraw {
            position: [indicator_origin.x, indicator_origin.y],
            size: [self.indicator_extent; 2],
            radii: [radius; 4],
            color: faded(theme.colors.surface.to_array(), opacity),
            border_width: theme.borders.thin,
            border_color: faded(
                if radio.selected {
                    theme.colors.primary.to_array()
                } else {
                    theme.colors.border.to_array()
                },
                opacity,
            ),
        }];
        if radio.selected {
            let inset = self.indicator_extent * 0.3;
            let inner = self.indicator_extent - inset * 2.0;
            indicator.push(RectDraw {
                position: [indicator_origin.x + inset, indicator_origin.y + inset],
                size: [inner; 2],
                radii: [inner * 0.5; 4],
                color: faded(theme.colors.primary.to_array(), opacity),
                border_width: 0.0,
                border_color: [0.0; 4],
            });
        }
        RadioDraws {
            indicator,
            label: self.label.draws(
                radio.label(),
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
    use super::Radio;
    use crate::{
        Direction, LogicalConstraints, LogicalPoint, SemanticRole, TextSystem, ThemeController,
        ThemeDefinition, UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn activation_selects_once_and_respects_disabled_state() {
        let mut radio = Radio::new("Standard");
        assert!(radio.activate());
        assert!(radio.selected);
        assert!(!radio.activate());
        radio.selected = false;
        radio.disabled = true;
        assert!(!radio.activate());
    }

    #[test]
    fn semantics_expose_radio_selection() {
        let mut radio = Radio::new("Standard");
        radio.selected = true;
        let semantics = radio.semantics();
        assert_eq!(semantics.role, SemanticRole::Radio);
        assert_eq!(semantics.state.checked, Some(true));
        assert_eq!(semantics.name.as_deref(), Some("Standard"));
    }

    #[test]
    fn layout_and_paint_mirror_and_show_selected_dot() {
        let mut radio = Radio::new("Standard");
        radio.selected = true;
        let theme = theme();
        let mut text_system = TextSystem::new();
        let ltr = radio.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let rtl = radio.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::unconstrained(),
        );
        assert!(ltr.indicator_origin.x < ltr.label_origin.x);
        assert!(rtl.indicator_origin.x > rtl.label_origin.x);
        assert_eq!(ltr.size, rtl.size);
        let draws = rtl.draws(&radio, LogicalPoint::default(), &theme);
        assert_eq!(draws.indicator.len(), 2);
        assert_eq!(draws.indicator[1].color, theme.colors.primary.to_array());
    }
}
