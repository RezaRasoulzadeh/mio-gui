// preferences.rs

use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MotionPreference {
    #[default]
    NoPreference,
    Reduce,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ContrastPreference {
    #[default]
    NoPreference,
    More,
    Less,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserPreferences {
    pub motion: MotionPreference,
    pub contrast: ContrastPreference,
    text_scale: f32,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            motion: MotionPreference::NoPreference,
            contrast: ContrastPreference::NoPreference,
            text_scale: 1.0,
        }
    }
}

impl UserPreferences {
    pub fn new(motion: MotionPreference, contrast: ContrastPreference, text_scale: f32) -> Self {
        Self {
            motion,
            contrast,
            text_scale: normalize_text_scale(text_scale),
        }
    }

    pub fn text_scale(self) -> f32 {
        self.text_scale
    }

    pub fn set_text_scale(&mut self, text_scale: f32) {
        self.text_scale = normalize_text_scale(text_scale);
    }

    pub fn motion_duration(self, duration: Duration) -> Duration {
        match self.motion {
            MotionPreference::NoPreference => duration,
            MotionPreference::Reduce => Duration::ZERO,
        }
    }

    pub fn scaled_text_size(self, logical_size: f32) -> f32 {
        finite_non_negative(logical_size) * self.text_scale
    }

    pub fn resolve_contrast<T: Copy>(self, normal: T, more: T, less: T) -> T {
        match self.contrast {
            ContrastPreference::NoPreference => normal,
            ContrastPreference::More => more,
            ContrastPreference::Less => less,
        }
    }
}

fn normalize_text_scale(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.5, 3.0)
    } else {
        1.0
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{ContrastPreference, MotionPreference, UserPreferences};

    #[test]
    fn reduced_motion_removes_nonessential_duration() {
        let normal = UserPreferences::default();
        let reduced = UserPreferences::new(
            MotionPreference::Reduce,
            ContrastPreference::NoPreference,
            1.0,
        );
        let duration = Duration::from_millis(180);

        assert_eq!(normal.motion_duration(duration), duration);
        assert_eq!(reduced.motion_duration(duration), Duration::ZERO);
    }

    #[test]
    fn contrast_variants_are_selected_centrally() {
        let normal = UserPreferences::default();
        let more = UserPreferences::new(
            MotionPreference::NoPreference,
            ContrastPreference::More,
            1.0,
        );
        let less = UserPreferences::new(
            MotionPreference::NoPreference,
            ContrastPreference::Less,
            1.0,
        );

        assert_eq!(normal.resolve_contrast(1, 2, 3), 1);
        assert_eq!(more.resolve_contrast(1, 2, 3), 2);
        assert_eq!(less.resolve_contrast(1, 2, 3), 3);
    }

    #[test]
    fn platform_text_scale_is_bounded_and_applied_to_logical_size() {
        let minimum = UserPreferences::new(
            MotionPreference::NoPreference,
            ContrastPreference::NoPreference,
            0.1,
        );
        let maximum = UserPreferences::new(
            MotionPreference::NoPreference,
            ContrastPreference::NoPreference,
            9.0,
        );
        let invalid = UserPreferences::new(
            MotionPreference::NoPreference,
            ContrastPreference::NoPreference,
            f32::NAN,
        );

        assert_eq!(minimum.text_scale(), 0.5);
        assert_eq!(maximum.text_scale(), 3.0);
        assert_eq!(invalid.text_scale(), 1.0);
        assert_eq!(maximum.scaled_text_size(16.0), 48.0);
        assert_eq!(maximum.scaled_text_size(f32::INFINITY), 0.0);
    }
}
