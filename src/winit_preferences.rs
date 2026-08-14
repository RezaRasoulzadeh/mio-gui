// winit_preferences.rs

use crate::{ColorScheme, ThemeController};

pub fn color_scheme_from_winit(theme: winit::window::Theme) -> ColorScheme {
    match theme {
        winit::window::Theme::Light => ColorScheme::Light,
        winit::window::Theme::Dark => ColorScheme::Dark,
    }
}

pub fn apply_winit_theme(controller: &mut ThemeController, theme: winit::window::Theme) -> bool {
    controller.set_system_scheme(color_scheme_from_winit(theme))
}

#[cfg(test)]
mod tests {
    use crate::{ColorScheme, ThemeController, ThemeMode};

    use super::{apply_winit_theme, color_scheme_from_winit};

    #[test]
    fn native_themes_map_to_core_color_schemes() {
        assert_eq!(
            color_scheme_from_winit(winit::window::Theme::Light),
            ColorScheme::Light
        );
        assert_eq!(
            color_scheme_from_winit(winit::window::Theme::Dark),
            ColorScheme::Dark
        );
    }

    #[test]
    fn native_theme_changes_only_invalidate_system_mode() {
        let mut controller = ThemeController::default();

        assert!(apply_winit_theme(
            &mut controller,
            winit::window::Theme::Dark
        ));
        assert_eq!(controller.effective_scheme(), ColorScheme::Dark);
        assert_eq!(controller.generation(), 1);

        controller.set_mode(ThemeMode::Light);
        let generation = controller.generation();
        assert!(!apply_winit_theme(
            &mut controller,
            winit::window::Theme::Light
        ));
        assert_eq!(controller.generation(), generation);
    }
}
