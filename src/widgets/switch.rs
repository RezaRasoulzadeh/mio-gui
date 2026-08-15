// switch.rs

use crate::{
    Direction, DirectionSetting, FocusPolicy, LogicalConstraints, LogicalPoint, LogicalSize,
    RectDraw, ResolvedTheme, SemanticAction, SemanticRole, Semantics, Text, TextDraw, TextStyle,
    TextSystem, TextWrap,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Switch {
    label: String,
    pub checked: bool,
    pub disabled: bool,
    pub direction: DirectionSetting,
}

impl Switch {
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
        let mut semantics = Semantics::new(SemanticRole::Switch)
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
    ) -> SwitchLayout {
        let direction = self.direction.resolve(inherited_direction);
        let track_height = theme.typography.size.max(16.0);
        let track_size = LogicalSize::new(track_height * 1.75, track_height);
        let gap = theme.spacing.small;
        let reserved = track_size.width + gap;
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
            track_size.height.max(label.size.height),
        ));
        let track_y = ((size.height - track_size.height) * 0.5).max(0.0);
        let label_y = ((size.height - label.size.height) * 0.5).max(0.0);
        let (track_x, label_x) = match direction {
            Direction::Ltr => (0.0, reserved),
            Direction::Rtl => ((size.width - track_size.width).max(0.0), 0.0),
        };
        SwitchLayout {
            size,
            direction,
            track_origin: LogicalPoint::new(track_x, track_y),
            track_size,
            label,
            label_origin: LogicalPoint::new(label_x, label_y),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SwitchLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub track_origin: LogicalPoint,
    pub track_size: LogicalSize,
    pub label: crate::TextLayout,
    pub label_origin: LogicalPoint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SwitchDraws {
    pub control: Vec<RectDraw>,
    pub label: Vec<TextDraw>,
}

impl SwitchLayout {
    pub fn draws(
        &self,
        switch: &Switch,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
    ) -> SwitchDraws {
        let opacity = if switch.disabled { 0.45 } else { 1.0 };
        let track_origin = add(origin, self.track_origin);
        let radius = self.track_size.height * 0.5;
        let track_color = if switch.checked {
            theme.colors.primary
        } else {
            theme.colors.border
        };
        let thumb_extent = self.track_size.height * 0.75;
        let inset = (self.track_size.height - thumb_extent) * 0.5;
        let thumb_x = match (self.direction, switch.checked) {
            (Direction::Ltr, false) | (Direction::Rtl, true) => track_origin.x + inset,
            (Direction::Ltr, true) | (Direction::Rtl, false) => {
                track_origin.x + self.track_size.width - thumb_extent - inset
            }
        };
        SwitchDraws {
            control: vec![
                RectDraw {
                    position: [track_origin.x, track_origin.y],
                    size: [self.track_size.width, self.track_size.height],
                    radii: [radius; 4],
                    color: faded(track_color.to_array(), opacity),
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
                RectDraw {
                    position: [thumb_x, track_origin.y + inset],
                    size: [thumb_extent; 2],
                    radii: [thumb_extent * 0.5; 4],
                    color: faded(theme.colors.surface.to_array(), opacity),
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            ],
            label: self.label.draws(
                switch.label(),
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
    use super::Switch;
    use crate::{
        Direction, LogicalConstraints, LogicalPoint, SemanticRole, TextSystem, ThemeController,
        ThemeDefinition, UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn activation_toggles_only_enabled_switches() {
        let mut switch = Switch::new("Notifications");
        assert!(switch.activate());
        assert!(switch.checked);
        switch.disabled = true;
        assert!(!switch.activate());
        assert!(switch.checked);
    }

    #[test]
    fn semantics_expose_switch_state() {
        let mut switch = Switch::new("Notifications");
        switch.checked = true;
        let semantics = switch.semantics();
        assert_eq!(semantics.role, SemanticRole::Switch);
        assert_eq!(semantics.state.checked, Some(true));
        assert_eq!(semantics.name.as_deref(), Some("Notifications"));
    }

    #[test]
    fn rtl_layout_and_checked_thumb_follow_logical_direction() {
        let mut switch = Switch::new("Notifications");
        switch.checked = true;
        let theme = theme();
        let mut text_system = TextSystem::new();
        let ltr = switch.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalConstraints::unconstrained(),
        );
        let rtl = switch.layout(
            &mut text_system,
            &theme,
            Direction::Rtl,
            LogicalConstraints::unconstrained(),
        );
        assert!(ltr.track_origin.x < ltr.label_origin.x);
        assert!(rtl.track_origin.x > rtl.label_origin.x);
        let ltr_draws = ltr.draws(&switch, LogicalPoint::default(), &theme);
        let rtl_draws = rtl.draws(&switch, LogicalPoint::default(), &theme);
        assert!(ltr_draws.control[1].position[0] > ltr_draws.control[0].position[0]);
        assert_eq!(
            rtl_draws.control[1].position[0],
            rtl_draws.control[0].position[0] + 2.0
        );
    }
}
