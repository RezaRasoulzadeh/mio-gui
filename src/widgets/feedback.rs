// feedback.rs

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::{
    Button, ButtonDraws, ButtonLayout, ComponentStyle, Direction, DirectionSetting,
    LogicalConstraints, LogicalPoint, LogicalSize, RectDraw, ResolvedTheme, SemanticNumericValue,
    SemanticRole, Semantics, TextSystem, VisualVariant,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Alert {
    message: String,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Alert {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            style: ComponentStyle {
                variant: VisualVariant::Soft,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    pub fn semantics(&self) -> Semantics {
        Semantics::new(SemanticRole::Alert).with_name(self.message.clone())
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> AlertLayout {
        let mut button = Button::new(self.message.clone());
        button.style = self.style;
        button.direction = self.direction;
        AlertLayout {
            button: button.layout(text_system, theme, inherited_direction, constraints),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AlertLayout {
    pub button: ButtonLayout,
}

impl AlertLayout {
    pub fn draws(&self, alert: &Alert, origin: LogicalPoint) -> ButtonDraws {
        let mut button = Button::new(alert.message.clone());
        button.style = alert.style;
        button.direction = alert.direction;
        self.button.draws(&button, origin)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgressError;

impl Display for ProgressError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("progress value must be finite and between zero and one")
    }
}

impl Error for ProgressError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Progress {
    label: String,
    value: f32,
    pub direction: DirectionSetting,
}

impl Progress {
    pub fn new(label: impl Into<String>, value: f32) -> Result<Self, ProgressError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(ProgressError);
        }
        Ok(Self {
            label: label.into(),
            value,
            direction: DirectionSetting::Inherit,
        })
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) -> Result<bool, ProgressError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(ProgressError);
        }
        let changed = self.value != value;
        self.value = value;
        Ok(changed)
    }

    pub fn semantics(&self) -> Semantics {
        Semantics::new(SemanticRole::Progress)
            .with_name(self.label.clone())
            .with_value(format!("{}%", (self.value * 100.0).round()))
            .with_numeric_value(
                SemanticNumericValue::new(f64::from(self.value), 0.0, 1.0, None).unwrap(),
            )
    }

    pub fn layout(
        &self,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> ProgressLayout {
        let size = constraints.constrain(LogicalSize::new(160.0, theme.spacing.small.max(8.0)));
        ProgressLayout {
            size,
            direction: self.direction.resolve(inherited_direction),
            fraction: self.value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub fraction: f32,
}

impl ProgressLayout {
    pub fn draws(&self, origin: LogicalPoint, theme: &ResolvedTheme) -> [RectDraw; 2] {
        let fill_width = self.size.width * self.fraction;
        let fill_x = match self.direction {
            Direction::Ltr => origin.x,
            Direction::Rtl => origin.x + self.size.width - fill_width,
        };
        let radius = self.size.height * 0.5;
        [
            RectDraw {
                position: [origin.x, origin.y],
                size: [self.size.width, self.size.height],
                radii: [radius; 4],
                color: theme.colors.border.to_array(),
                border_width: 0.0,
                border_color: [0.0; 4],
            },
            RectDraw {
                position: [fill_x, origin.y],
                size: [fill_width, self.size.height],
                radii: [radius.min(fill_width * 0.5); 4],
                color: theme.colors.primary.to_array(),
                border_width: 0.0,
                border_color: [0.0; 4],
            },
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Loading {
    label: String,
    phase: f32,
    pub direction: DirectionSetting,
}

impl Loading {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            phase: 0.0,
            direction: DirectionSetting::Inherit,
        }
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn phase(&self) -> f32 {
        self.phase
    }
    pub fn set_phase(&mut self, phase: f32) -> bool {
        if !phase.is_finite() {
            return false;
        }
        let phase = phase.rem_euclid(1.0);
        let changed = self.phase != phase;
        self.phase = phase;
        changed
    }
    pub fn semantics(&self) -> Semantics {
        Semantics::new(SemanticRole::Progress).with_name(self.label.clone())
    }
    pub fn layout(
        &self,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> LoadingLayout {
        let dot = theme.spacing.small.max(8.0);
        let gap = theme.spacing.extra_small;
        LoadingLayout {
            size: constraints.constrain(LogicalSize::new(dot * 3.0 + gap * 2.0, dot)),
            direction: self.direction.resolve(inherited_direction),
            phase: self.phase,
            dot,
            gap,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoadingLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub phase: f32,
    pub dot: f32,
    pub gap: f32,
}

impl LoadingLayout {
    pub fn draws(&self, origin: LogicalPoint, theme: &ResolvedTheme) -> [RectDraw; 3] {
        std::array::from_fn(|index| {
            let logical_index = match self.direction {
                Direction::Ltr => index,
                Direction::Rtl => 2 - index,
            };
            let active = ((self.phase * 3.0).floor() as usize).min(2) == logical_index;
            let mut color = theme.colors.primary.to_array();
            color[3] = if active { 1.0 } else { 0.35 };
            let extent = self.dot.min(self.size.height);
            RectDraw {
                position: [origin.x + index as f32 * (self.dot + self.gap), origin.y],
                size: [extent, extent],
                radii: [extent * 0.5; 4],
                color,
                border_width: 0.0,
                border_color: [0.0; 4],
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Skeleton {
    pub size: LogicalSize,
    pub radius: f32,
}

impl Skeleton {
    pub const fn new(size: LogicalSize) -> Self {
        Self { size, radius: 0.0 }
    }
    pub fn semantics(self) -> Semantics {
        let mut semantics = Semantics::default();
        semantics.state.hidden = true;
        semantics
    }
    pub fn layout(self, constraints: LogicalConstraints) -> LogicalSize {
        constraints.constrain(self.size)
    }
    pub fn draw(self, origin: LogicalPoint, size: LogicalSize, theme: &ResolvedTheme) -> RectDraw {
        RectDraw {
            position: [origin.x, origin.y],
            size: [size.width, size.height],
            radii: [self.radius.max(0.0); 4],
            color: theme.colors.border.to_array(),
            border_width: 0.0,
            border_color: [0.0; 4],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadialProgress {
    label: String,
    value: f32,
    pub size: f32,
    pub direction: DirectionSetting,
}

impl RadialProgress {
    pub fn new(label: impl Into<String>, value: f32) -> Result<Self, ProgressError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(ProgressError);
        }
        Ok(Self {
            label: label.into(),
            value,
            size: 48.0,
            direction: DirectionSetting::Inherit,
        })
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn set_value(&mut self, value: f32) -> Result<bool, ProgressError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(ProgressError);
        }
        let changed = self.value != value;
        self.value = value;
        Ok(changed)
    }
    pub fn semantics(&self) -> Semantics {
        Semantics::new(SemanticRole::Progress)
            .with_name(self.label.clone())
            .with_value(format!("{}%", (self.value * 100.0).round()))
            .with_numeric_value(
                SemanticNumericValue::new(f64::from(self.value), 0.0, 1.0, None).unwrap(),
            )
    }
    pub fn layout(
        &self,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> RadialProgressLayout {
        let extent = if self.size.is_finite() {
            self.size.max(0.0)
        } else {
            0.0
        };
        RadialProgressLayout {
            size: constraints.constrain(LogicalSize::new(extent, extent)),
            direction: self.direction.resolve(inherited_direction),
            fraction: self.value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadialProgressLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub fraction: f32,
}

impl RadialProgressLayout {
    pub fn draws(&self, origin: LogicalPoint, theme: &ResolvedTheme) -> Vec<RectDraw> {
        const SEGMENTS: usize = 12;
        let extent = self.size.width.min(self.size.height);
        let dot = (extent * 0.12).max(1.0);
        let radius = (extent - dot) * 0.5;
        let center = LogicalPoint::new(
            origin.x + self.size.width * 0.5,
            origin.y + self.size.height * 0.5,
        );
        let filled = (self.fraction * SEGMENTS as f32).round() as usize;
        (0..SEGMENTS)
            .map(|index| {
                let logical = match self.direction {
                    Direction::Ltr => index,
                    Direction::Rtl => (SEGMENTS - index) % SEGMENTS,
                };
                let angle = -std::f32::consts::FRAC_PI_2
                    + logical as f32 * std::f32::consts::TAU / SEGMENTS as f32;
                RectDraw {
                    position: [
                        center.x + angle.cos() * radius - dot * 0.5,
                        center.y + angle.sin() * radius - dot * 0.5,
                    ],
                    size: [dot, dot],
                    radii: [dot * 0.5; 4],
                    color: if index < filled {
                        theme.colors.primary.to_array()
                    } else {
                        theme.colors.border.to_array()
                    },
                    border_width: 0.0,
                    border_color: [0.0; 4],
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToastAction {
    #[default]
    None,
    Dismissed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub alert: Alert,
    pub open: bool,
    pub dismissible: bool,
}

impl Toast {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            alert: Alert::new(message),
            open: true,
            dismissible: true,
        }
    }
    pub fn dismiss(&mut self) -> ToastAction {
        if self.open && self.dismissible {
            self.open = false;
            ToastAction::Dismissed
        } else {
            ToastAction::None
        }
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = self.alert.semantics();
        semantics.state.hidden = !self.open;
        semantics
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> ToastLayout {
        if self.open {
            let alert = self
                .alert
                .layout(text_system, theme, inherited_direction, constraints);
            ToastLayout {
                size: alert.button.size,
                alert: Some(alert),
            }
        } else {
            ToastLayout {
                size: LogicalSize::default(),
                alert: None,
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastLayout {
    pub size: LogicalSize,
    pub alert: Option<AlertLayout>,
}

impl ToastLayout {
    pub fn draws(&self, toast: &Toast, origin: LogicalPoint) -> Option<ButtonDraws> {
        self.alert
            .as_ref()
            .map(|layout| layout.draws(&toast.alert, origin))
    }
}

#[cfg(test)]
mod tests {
    use super::{Alert, Loading, Progress, RadialProgress, Skeleton, Toast, ToastAction};
    use crate::{
        Direction, LogicalConstraints, SemanticRole, ThemeController, ThemeDefinition,
        UserPreferences,
    };

    #[test]
    fn alert_and_progress_expose_feedback_semantics() {
        let alert = Alert::new("Saved");
        let progress = Progress::new("Upload", 0.4).unwrap();
        assert_eq!(alert.semantics().role, SemanticRole::Alert);
        assert_eq!(progress.semantics().role, SemanticRole::Progress);
        assert!((progress.semantics().numeric_value.unwrap().value() - 0.4).abs() < 0.000_001);
    }

    #[test]
    fn progress_validates_values_and_fills_from_logical_start() {
        assert!(Progress::new("Upload", f32::NAN).is_err());
        assert!(Progress::new("Upload", 1.1).is_err());
        let progress = Progress::new("Upload", 0.25).unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let ltr = progress.layout(&theme, Direction::Ltr, LogicalConstraints::unconstrained());
        let rtl = progress.layout(&theme, Direction::Rtl, LogicalConstraints::unconstrained());
        let ltr_draws = ltr.draws(Default::default(), &theme);
        let rtl_draws = rtl.draws(Default::default(), &theme);
        assert_eq!(ltr_draws[1].position[0], 0.0);
        assert_eq!(rtl_draws[1].position[0], 120.0);
    }

    #[test]
    fn loading_mirrors_activity_and_skeleton_is_hidden() {
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut loading = Loading::new("Working");
        assert!(loading.set_phase(0.1));
        let ltr = loading.layout(&theme, Direction::Ltr, LogicalConstraints::unconstrained());
        let rtl = loading.layout(&theme, Direction::Rtl, LogicalConstraints::unconstrained());
        assert_eq!(ltr.draws(Default::default(), &theme)[0].color[3], 1.0);
        assert_eq!(rtl.draws(Default::default(), &theme)[2].color[3], 1.0);
        assert!(
            Skeleton::new(crate::LogicalSize::new(80.0, 20.0))
                .semantics()
                .state
                .hidden
        );
    }

    #[test]
    fn radial_progress_validates_values_and_mirrors_segments() {
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let radial = RadialProgress::new("Download", 0.5).unwrap();
        assert_eq!(radial.semantics().role, SemanticRole::Progress);
        let ltr = radial.layout(Direction::Ltr, LogicalConstraints::unconstrained());
        let rtl = radial.layout(Direction::Rtl, LogicalConstraints::unconstrained());
        let ltr_draws = ltr.draws(Default::default(), &theme);
        let rtl_draws = rtl.draws(Default::default(), &theme);
        assert_eq!(ltr_draws.len(), 12);
        assert!((ltr_draws[1].position[0] - rtl_draws[11].position[0]).abs() < 0.001);
        assert!(RadialProgress::new("Download", -0.1).is_err());
    }

    #[test]
    fn toast_dismissal_hides_semantics_and_layout() {
        let mut toast = Toast::new("Saved");
        assert_eq!(toast.semantics().role, SemanticRole::Alert);
        assert_eq!(toast.dismiss(), ToastAction::Dismissed);
        assert!(toast.semantics().state.hidden);
        assert_eq!(toast.dismiss(), ToastAction::None);
    }
}
