// style.rs

use crate::{Direction, LinearColor, MINIMUM_TEXT_CONTRAST, ResolvedTheme};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ComponentSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ComponentMetrics {
    pub minimum_block_size: f32,
    pub padding_inline: f32,
    pub padding_block: f32,
    pub content_gap: f32,
    pub icon_size: f32,
    pub corner_radius: f32,
}

impl ComponentSize {
    pub fn resolve(self, theme: &ResolvedTheme) -> ComponentMetrics {
        match self {
            Self::Small => ComponentMetrics {
                minimum_block_size: 32.0,
                padding_inline: theme.spacing.small,
                padding_block: theme.spacing.extra_small,
                content_gap: theme.spacing.extra_small,
                icon_size: 16.0,
                corner_radius: theme.radii.small,
            },
            Self::Medium => ComponentMetrics {
                minimum_block_size: 40.0,
                padding_inline: theme.spacing.medium,
                padding_block: theme.spacing.small,
                content_gap: theme.spacing.small,
                icon_size: 20.0,
                corner_radius: theme.radii.medium,
            },
            Self::Large => ComponentMetrics {
                minimum_block_size: 48.0,
                padding_inline: theme.spacing.large,
                padding_block: theme.spacing.medium,
                content_gap: theme.spacing.small,
                icon_size: 24.0,
                corner_radius: theme.radii.large,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum VisualVariant {
    #[default]
    Solid,
    Outline,
    Soft,
    Ghost,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComponentAppearance {
    pub background: LinearColor,
    pub foreground: LinearColor,
    pub border: LinearColor,
    pub border_width: f32,
    pub state_layer: LinearColor,
    pub focus_ring: LinearColor,
    pub focus_ring_width: f32,
    pub opacity: f32,
}

impl Default for ComponentAppearance {
    fn default() -> Self {
        Self {
            background: transparent(),
            foreground: transparent(),
            border: transparent(),
            border_width: 0.0,
            state_layer: transparent(),
            focus_ring: transparent(),
            focus_ring_width: 0.0,
            opacity: 1.0,
        }
    }
}

impl VisualVariant {
    pub fn resolve(self, theme: &ResolvedTheme) -> ComponentAppearance {
        self.resolve_state(theme, ComponentState::default())
    }

    pub fn resolve_state(
        self,
        theme: &ResolvedTheme,
        state: ComponentState,
    ) -> ComponentAppearance {
        let state = state.normalized();
        let colors = theme.colors;
        let accent = if state.error {
            colors.error
        } else {
            colors.primary
        };
        let on_accent = if state.error {
            colors.on_error
        } else {
            colors.on_primary
        };
        let transparent = transparent();
        let mut appearance = match self {
            Self::Solid => ComponentAppearance {
                background: accent,
                foreground: on_accent,
                border: transparent,
                border_width: 0.0,
                ..ComponentAppearance::default()
            },
            Self::Outline => ComponentAppearance {
                background: transparent,
                foreground: accent,
                border: accent,
                border_width: theme.borders.regular,
                ..ComponentAppearance::default()
            },
            Self::Soft => ComponentAppearance {
                background: accessible_tint(accent, colors.surface, 0.16),
                foreground: accent,
                border: transparent,
                border_width: 0.0,
                ..ComponentAppearance::default()
            },
            Self::Ghost => ComponentAppearance {
                background: transparent,
                foreground: if state.error { accent } else { colors.text },
                border: transparent,
                border_width: 0.0,
                ..ComponentAppearance::default()
            },
        };
        appearance.opacity = if state.disabled { 0.45 } else { 1.0 };
        if state.selected && self != Self::Solid {
            appearance.background = accessible_tint(accent, colors.surface, 0.12);
        }
        let effective_background = appearance.background.composite_over(colors.surface);
        appearance.state_layer = if state.active {
            contrastive_overlay(appearance.foreground, effective_background, 0.16)
        } else if state.hovered {
            contrastive_overlay(appearance.foreground, effective_background, 0.08)
        } else {
            transparent
        };
        if state.focused {
            appearance.focus_ring = colors.focus;
            appearance.focus_ring_width = theme.borders.thick;
        }
        appearance
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ComponentState {
    pub hovered: bool,
    pub active: bool,
    pub focused: bool,
    pub disabled: bool,
    pub selected: bool,
    pub error: bool,
}

impl ComponentState {
    pub fn normalized(self) -> Self {
        if self.disabled {
            Self {
                hovered: false,
                active: false,
                focused: false,
                ..self
            }
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AdornmentPlacement {
    #[default]
    None,
    InlineStart,
    InlineEnd,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum PhysicalAdornmentPlacement {
    #[default]
    None,
    Left,
    Right,
}

impl AdornmentPlacement {
    pub const fn resolve(self, direction: Direction) -> PhysicalAdornmentPlacement {
        match (self, direction) {
            (Self::None, _) => PhysicalAdornmentPlacement::None,
            (Self::InlineStart, Direction::Ltr) | (Self::InlineEnd, Direction::Rtl) => {
                PhysicalAdornmentPlacement::Left
            }
            (Self::InlineStart, Direction::Rtl) | (Self::InlineEnd, Direction::Ltr) => {
                PhysicalAdornmentPlacement::Right
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ComponentStyle {
    pub size: ComponentSize,
    pub variant: VisualVariant,
    pub state: ComponentState,
    pub adornment: AdornmentPlacement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResolvedComponentStyle {
    pub metrics: ComponentMetrics,
    pub appearance: ComponentAppearance,
    pub adornment: PhysicalAdornmentPlacement,
}

impl ComponentStyle {
    pub fn resolve(self, theme: &ResolvedTheme, direction: Direction) -> ResolvedComponentStyle {
        ResolvedComponentStyle {
            metrics: self.size.resolve(theme),
            appearance: self.variant.resolve_state(theme, self.state),
            adornment: self.adornment.resolve(direction),
        }
    }
}

fn transparent() -> LinearColor {
    LinearColor::new(0.0, 0.0, 0.0, 0.0)
}

fn with_alpha(color: LinearColor, alpha: f32) -> LinearColor {
    LinearColor::new(color.red, color.green, color.blue, alpha)
}

fn contrastive_overlay(
    foreground: LinearColor,
    background: LinearColor,
    alpha: f32,
) -> LinearColor {
    if foreground.relative_luminance() > background.relative_luminance() {
        LinearColor::new(0.0, 0.0, 0.0, alpha)
    } else {
        LinearColor::new(1.0, 1.0, 1.0, alpha)
    }
}

fn accessible_tint(
    foreground: LinearColor,
    surface: LinearColor,
    maximum_alpha: f32,
) -> LinearColor {
    let mut lower = 0.0;
    let mut upper = maximum_alpha;
    for _ in 0..12 {
        let candidate = (lower + upper) * 0.5;
        let background = with_alpha(foreground, candidate).composite_over(surface);
        if foreground.contrast_ratio(background) >= MINIMUM_TEXT_CONTRAST {
            lower = candidate;
        } else {
            upper = candidate;
        }
    }
    with_alpha(foreground, lower)
}

#[cfg(test)]
mod tests {
    use crate::{
        ColorScheme, ContrastPreference, MINIMUM_TEXT_CONTRAST, MotionPreference, ThemeController,
        ThemeDefinition, ThemeMode, UserPreferences,
    };

    use super::{
        AdornmentPlacement, ComponentSize, ComponentState, ComponentStyle,
        PhysicalAdornmentPlacement, VisualVariant,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn component_sizes_resolve_monotonic_logical_metrics() {
        let theme = theme();
        let small = ComponentSize::Small.resolve(&theme);
        let medium = ComponentSize::Medium.resolve(&theme);
        let large = ComponentSize::Large.resolve(&theme);

        assert!(small.minimum_block_size < medium.minimum_block_size);
        assert!(medium.minimum_block_size < large.minimum_block_size);
        assert!(small.padding_inline < medium.padding_inline);
        assert!(medium.padding_inline < large.padding_inline);
        assert!(small.corner_radius < medium.corner_radius);
        assert!(medium.corner_radius < large.corner_radius);
    }

    #[test]
    fn visual_variants_use_only_resolved_semantic_colors() {
        let theme = theme();
        let solid = VisualVariant::Solid.resolve(&theme);
        let outline = VisualVariant::Outline.resolve(&theme);
        let soft = VisualVariant::Soft.resolve(&theme);
        let ghost = VisualVariant::Ghost.resolve(&theme);

        assert_eq!(solid.background, theme.colors.primary);
        assert_eq!(solid.foreground, theme.colors.on_primary);
        assert_eq!(outline.border, theme.colors.primary);
        assert_eq!(outline.border_width, theme.borders.regular);
        assert!(soft.background.alpha <= 0.16);
        assert!(soft.background.alpha > 0.0);
        assert_eq!(ghost.foreground, theme.colors.text);
    }

    #[test]
    fn direction_changes_only_semantically_directional_style_output() {
        let theme = theme();
        let style = ComponentStyle {
            size: ComponentSize::Medium,
            variant: VisualVariant::Outline,
            state: ComponentState {
                focused: true,
                selected: true,
                ..ComponentState::default()
            },
            adornment: AdornmentPlacement::InlineStart,
        };
        let ltr = style.resolve(&theme, crate::Direction::Ltr);
        let rtl = style.resolve(&theme, crate::Direction::Rtl);

        assert_eq!(ltr.metrics, rtl.metrics);
        assert_eq!(ltr.appearance, rtl.appearance);
        assert_eq!(ltr.adornment, PhysicalAdornmentPlacement::Left);
        assert_eq!(rtl.adornment, PhysicalAdornmentPlacement::Right);
    }

    #[test]
    fn logical_end_adornment_mirrors_and_none_remains_physical_none() {
        assert_eq!(
            AdornmentPlacement::InlineEnd.resolve(crate::Direction::Ltr),
            PhysicalAdornmentPlacement::Right
        );
        assert_eq!(
            AdornmentPlacement::InlineEnd.resolve(crate::Direction::Rtl),
            PhysicalAdornmentPlacement::Left
        );
        assert_eq!(
            AdornmentPlacement::None.resolve(crate::Direction::Rtl),
            PhysicalAdornmentPlacement::None
        );
    }

    #[test]
    fn interaction_precedence_is_active_then_hover_then_rest() {
        let theme = theme();
        let hovered = VisualVariant::Solid.resolve_state(
            &theme,
            ComponentState {
                hovered: true,
                ..ComponentState::default()
            },
        );
        let active = VisualVariant::Solid.resolve_state(
            &theme,
            ComponentState {
                hovered: true,
                active: true,
                ..ComponentState::default()
            },
        );

        assert_eq!(hovered.state_layer.alpha, 0.08);
        assert_eq!(active.state_layer.alpha, 0.16);
    }

    #[test]
    fn disabled_suppresses_transient_and_focus_states() {
        let theme = theme();
        let appearance = VisualVariant::Outline.resolve_state(
            &theme,
            ComponentState {
                hovered: true,
                active: true,
                focused: true,
                disabled: true,
                ..ComponentState::default()
            },
        );

        assert_eq!(appearance.opacity, 0.45);
        assert_eq!(appearance.state_layer.alpha, 0.0);
        assert_eq!(appearance.focus_ring_width, 0.0);
    }

    #[test]
    fn error_selected_and_focus_resolve_as_orthogonal_semantic_states() {
        let theme = theme();
        let appearance = VisualVariant::Outline.resolve_state(
            &theme,
            ComponentState {
                focused: true,
                selected: true,
                error: true,
                ..ComponentState::default()
            },
        );

        assert_eq!(appearance.foreground, theme.colors.error);
        assert_eq!(appearance.border, theme.colors.error);
        assert_eq!(appearance.background.red, theme.colors.error.red);
        assert!(appearance.background.alpha <= 0.12);
        assert_eq!(appearance.focus_ring, theme.colors.focus);
        assert_eq!(appearance.focus_ring_width, theme.borders.thick);
    }

    #[test]
    fn resolved_variant_states_preserve_text_contrast_after_compositing() {
        let definition = ThemeDefinition::default();
        let states = [
            ComponentState::default(),
            ComponentState {
                hovered: true,
                ..ComponentState::default()
            },
            ComponentState {
                active: true,
                ..ComponentState::default()
            },
            ComponentState {
                selected: true,
                ..ComponentState::default()
            },
            ComponentState {
                error: true,
                ..ComponentState::default()
            },
        ];

        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            for contrast in [
                ContrastPreference::NoPreference,
                ContrastPreference::More,
                ContrastPreference::Less,
            ] {
                let mut controller = ThemeController::default();
                controller.set_mode(match scheme {
                    ColorScheme::Light => ThemeMode::Light,
                    ColorScheme::Dark => ThemeMode::Dark,
                });
                let preferences =
                    UserPreferences::new(MotionPreference::NoPreference, contrast, 1.0);
                let theme = definition.resolve(controller, preferences);

                for variant in [
                    VisualVariant::Solid,
                    VisualVariant::Outline,
                    VisualVariant::Soft,
                    VisualVariant::Ghost,
                ] {
                    for state in states {
                        let appearance = variant.resolve_state(&theme, state);
                        let background = appearance.background.composite_over(theme.colors.surface);
                        let background = appearance.state_layer.composite_over(background);
                        assert!(
                            appearance.foreground.contrast_ratio(background)
                                >= MINIMUM_TEXT_CONTRAST,
                            "scheme={scheme:?} contrast={contrast:?} variant={variant:?} state={state:?} ratio={}",
                            appearance.foreground.contrast_ratio(background)
                        );
                    }
                }
            }
        }
    }
}
