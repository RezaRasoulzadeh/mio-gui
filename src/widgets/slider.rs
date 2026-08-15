// slider.rs

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::RangeInclusive;

use crate::{
    ArrowKey, Direction, DirectionSetting, FocusPolicy, Key, KeyState, KeyboardEvent,
    LogicalConstraints, LogicalPoint, LogicalSize, RectDraw, ResolvedTheme, SemanticAction,
    SemanticNumericValue, SemanticRole, Semantics,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SliderError {
    InvalidRange,
    InvalidValue,
    InvalidStep,
}

impl Display for SliderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRange => formatter.write_str("slider range must be finite and increasing"),
            Self::InvalidValue => formatter.write_str("slider value must be finite"),
            Self::InvalidStep => formatter.write_str("slider step must be finite and positive"),
        }
    }
}

impl Error for SliderError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Slider {
    label: String,
    minimum: f32,
    maximum: f32,
    value: f32,
    step: f32,
    pub disabled: bool,
    pub direction: DirectionSetting,
}

impl Slider {
    pub fn new(
        label: impl Into<String>,
        range: RangeInclusive<f32>,
        value: f32,
    ) -> Result<Self, SliderError> {
        let minimum = *range.start();
        let maximum = *range.end();
        if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
            return Err(SliderError::InvalidRange);
        }
        if !value.is_finite() {
            return Err(SliderError::InvalidValue);
        }
        Ok(Self {
            label: label.into(),
            minimum,
            maximum,
            value: value.clamp(minimum, maximum),
            step: (maximum - minimum) / 100.0,
            disabled: false,
            direction: DirectionSetting::Inherit,
        })
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    pub fn range(&self) -> RangeInclusive<f32> {
        self.minimum..=self.maximum
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) -> Result<bool, SliderError> {
        if !value.is_finite() {
            return Err(SliderError::InvalidValue);
        }
        if self.disabled {
            return Ok(false);
        }
        let value = value.clamp(self.minimum, self.maximum);
        let changed = value != self.value;
        self.value = value;
        Ok(changed)
    }

    pub fn step(&self) -> f32 {
        self.step
    }

    pub fn set_step(&mut self, step: f32) -> Result<(), SliderError> {
        if !step.is_finite() || step <= 0.0 {
            return Err(SliderError::InvalidStep);
        }
        self.step = step;
        Ok(())
    }

    pub fn increment(&mut self) -> bool {
        self.adjust(self.step)
    }

    pub fn decrement(&mut self) -> bool {
        self.adjust(-self.step)
    }

    pub fn handle_key(&mut self, event: &KeyboardEvent, direction: Direction) -> bool {
        if event.state != KeyState::Pressed {
            return false;
        }
        match event.key {
            Key::Arrow(ArrowKey::Up) => self.increment(),
            Key::Arrow(ArrowKey::Down) => self.decrement(),
            Key::Arrow(ArrowKey::Right) if direction == Direction::Ltr => self.increment(),
            Key::Arrow(ArrowKey::Right) => self.decrement(),
            Key::Arrow(ArrowKey::Left) if direction == Direction::Ltr => self.decrement(),
            Key::Arrow(ArrowKey::Left) => self.increment(),
            Key::Home => self.set_value(self.minimum).unwrap_or(false),
            Key::End => self.set_value(self.maximum).unwrap_or(false),
            Key::PageUp => self.adjust(self.step * 10.0),
            Key::PageDown => self.adjust(self.step * -10.0),
            _ => false,
        }
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Slider)
            .with_name(self.label.clone())
            .with_value(self.value.to_string())
            .with_numeric_value(
                SemanticNumericValue::new(
                    f64::from(self.value),
                    f64::from(self.minimum),
                    f64::from(self.maximum),
                    Some(f64::from(self.step)),
                )
                .unwrap(),
            )
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Increment)
            .with_action(SemanticAction::Decrement)
            .with_action(SemanticAction::SetValue);
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
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> SliderLayout {
        let direction = self.direction.resolve(inherited_direction);
        let thumb_extent = theme.typography.size.max(16.0);
        let preferred = LogicalSize::new(160.0, thumb_extent);
        let size = constraints.constrain(preferred);
        let track_height = theme.borders.thick.max(4.0).min(size.height);
        let track_origin = LogicalPoint::new(0.0, (size.height - track_height) * 0.5);
        SliderLayout {
            size,
            direction,
            track_origin,
            track_size: LogicalSize::new(size.width, track_height),
            thumb_extent: thumb_extent.min(size.width).min(size.height),
            fraction: (self.value - self.minimum) / (self.maximum - self.minimum),
        }
    }

    fn adjust(&mut self, delta: f32) -> bool {
        if self.disabled {
            return false;
        }
        let value = (self.value + delta).clamp(self.minimum, self.maximum);
        let changed = value != self.value;
        self.value = value;
        changed
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub track_origin: LogicalPoint,
    pub track_size: LogicalSize,
    pub thumb_extent: f32,
    pub fraction: f32,
}

impl SliderLayout {
    pub fn draws(
        &self,
        slider: &Slider,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
    ) -> Vec<RectDraw> {
        let opacity = if slider.disabled { 0.45 } else { 1.0 };
        let track = add(origin, self.track_origin);
        let fill_width = self.track_size.width * self.fraction;
        let fill_x = match self.direction {
            Direction::Ltr => track.x,
            Direction::Rtl => track.x + self.track_size.width - fill_width,
        };
        let center_x = match self.direction {
            Direction::Ltr => track.x + fill_width,
            Direction::Rtl => track.x + self.track_size.width - fill_width,
        };
        vec![
            RectDraw {
                position: [track.x, track.y],
                size: [self.track_size.width, self.track_size.height],
                radii: [self.track_size.height * 0.5; 4],
                color: faded(theme.colors.border.to_array(), opacity),
                border_width: 0.0,
                border_color: [0.0; 4],
            },
            RectDraw {
                position: [fill_x, track.y],
                size: [fill_width, self.track_size.height],
                radii: [self.track_size.height * 0.5; 4],
                color: faded(theme.colors.primary.to_array(), opacity),
                border_width: 0.0,
                border_color: [0.0; 4],
            },
            RectDraw {
                position: [
                    center_x - self.thumb_extent * 0.5,
                    origin.y + (self.size.height - self.thumb_extent) * 0.5,
                ],
                size: [self.thumb_extent; 2],
                radii: [self.thumb_extent * 0.5; 4],
                color: faded(theme.colors.primary.to_array(), opacity),
                border_width: theme.borders.thin,
                border_color: faded(theme.colors.surface.to_array(), opacity),
            },
        ]
    }
}

fn add(left: LogicalPoint, right: LogicalPoint) -> LogicalPoint {
    LogicalPoint::new(left.x + right.x, left.y + right.y)
}

fn faded(mut color: [f32; 4], opacity: f32) -> [f32; 4] {
    color[3] *= opacity;
    color
}

#[cfg(test)]
mod tests {
    use super::{Slider, SliderError};
    use crate::{
        Direction, LogicalConstraints, LogicalPoint, SemanticAction, SemanticRole, ThemeController,
        ThemeDefinition, UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn validates_range_value_and_step() {
        assert_eq!(
            Slider::new("Volume", 1.0..=1.0, 1.0),
            Err(SliderError::InvalidRange)
        );
        assert_eq!(
            Slider::new("Volume", 0.0..=1.0, f32::NAN),
            Err(SliderError::InvalidValue)
        );
        let mut slider = Slider::new("Volume", 0.0..=10.0, 20.0).unwrap();
        assert_eq!(slider.value(), 10.0);
        assert_eq!(slider.set_step(0.0), Err(SliderError::InvalidStep));
    }

    #[test]
    fn adjustment_clamps_and_respects_disabled_state() {
        let mut slider = Slider::new("Volume", 0.0..=10.0, 5.0).unwrap();
        slider.set_step(3.0).unwrap();
        assert!(slider.increment());
        assert_eq!(slider.value(), 8.0);
        assert!(slider.increment());
        assert_eq!(slider.value(), 10.0);
        assert!(!slider.increment());
        slider.disabled = true;
        assert!(!slider.decrement());
    }

    #[test]
    fn keyboard_supports_arrows_edges_pages_repeat_and_rtl() {
        let mut slider = Slider::new("Volume", 0.0..=100.0, 50.0).unwrap();
        slider.set_step(5.0).unwrap();
        assert!(slider.handle_key(
            &crate::KeyboardEvent::pressed(crate::Key::Arrow(crate::ArrowKey::Up)),
            Direction::Ltr,
        ));
        assert_eq!(slider.value(), 55.0);
        assert!(slider.handle_key(
            &crate::KeyboardEvent::pressed(crate::Key::Arrow(crate::ArrowKey::Right)),
            Direction::Rtl,
        ));
        assert_eq!(slider.value(), 50.0);
        assert!(slider.handle_key(
            &crate::KeyboardEvent::pressed(crate::Key::PageUp),
            Direction::Rtl,
        ));
        assert_eq!(slider.value(), 100.0);
        assert!(slider.handle_key(
            &crate::KeyboardEvent::pressed(crate::Key::Home),
            Direction::Ltr,
        ));
        assert_eq!(slider.value(), 0.0);
        let mut repeated = crate::KeyboardEvent::pressed(crate::Key::Arrow(crate::ArrowKey::Up));
        repeated.repeat = true;
        assert!(slider.handle_key(&repeated, Direction::Ltr));
        let released = crate::KeyboardEvent {
            state: crate::KeyState::Released,
            ..crate::KeyboardEvent::pressed(crate::Key::End)
        };
        assert!(!slider.handle_key(&released, Direction::Ltr));
        assert_eq!(slider.value(), 5.0);
    }

    #[test]
    fn semantics_expose_value_and_adjustment_actions() {
        let slider = Slider::new("Volume", 0.0..=10.0, 4.0).unwrap();
        let semantics = slider.semantics();
        assert_eq!(semantics.role, SemanticRole::Slider);
        assert_eq!(semantics.value.as_deref(), Some("4"));
        assert!(semantics.supports(SemanticAction::Increment));
        assert!(semantics.supports(SemanticAction::Decrement));
        assert!(semantics.supports(SemanticAction::SetValue));
        let numeric = semantics.numeric_value.unwrap();
        assert_eq!(numeric.value(), 4.0);
        assert_eq!(numeric.range(), 0.0..=10.0);
        assert_eq!(numeric.step(), Some(f64::from(0.1_f32)));
    }

    #[test]
    fn rtl_fill_and_thumb_mirror_the_ltr_geometry() {
        let slider = Slider::new("Volume", 0.0..=100.0, 25.0).unwrap();
        let theme = theme();
        let constraints = LogicalConstraints::unconstrained();
        let ltr = slider.layout(&theme, Direction::Ltr, constraints);
        let rtl = slider.layout(&theme, Direction::Rtl, constraints);
        let ltr_draws = ltr.draws(&slider, LogicalPoint::default(), &theme);
        let rtl_draws = rtl.draws(&slider, LogicalPoint::default(), &theme);
        assert_eq!(ltr_draws[1].size, rtl_draws[1].size);
        assert!(ltr_draws[2].position[0] < rtl_draws[2].position[0]);
    }
}
