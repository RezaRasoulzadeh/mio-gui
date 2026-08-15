// button.rs

use crate::{
    AdornmentPlacement, ComponentStyle, Direction, DirectionSetting, FocusPolicy, Icon, IconLayout,
    ImageDraw, LogicalConstraints, LogicalPoint, LogicalSize, PhysicalAdornmentPlacement, RectDraw,
    ResolvedComponentStyle, ResolvedTheme, SemanticAction, SemanticRole, Semantics, Text, TextDraw,
    TextStyle, TextSystem, TextWrap,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Button {
    label: String,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
    icon: Option<Icon>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: ComponentStyle::default(),
            direction: DirectionSetting::Inherit,
            icon: None,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    pub fn with_icon(mut self, icon: Icon, placement: AdornmentPlacement) -> Self {
        self.icon = Some(icon);
        self.style.adornment = placement;
        self
    }

    pub fn icon(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

    pub fn set_icon(&mut self, icon: Option<Icon>) {
        self.icon = icon;
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Button)
            .with_name(self.label.clone())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Activate);
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

    pub fn resolve(&self, theme: &ResolvedTheme, inherited_direction: Direction) -> ButtonStyle {
        let direction = self.direction.resolve(inherited_direction);
        ButtonStyle {
            direction,
            component: self.style.resolve(theme, direction),
        }
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> ButtonLayout {
        let resolved = self.resolve(theme, inherited_direction);
        let metrics = resolved.component.metrics;
        let icon_extent = self.icon.as_ref().map(|_| metrics.icon_size).unwrap_or(0.0);
        let gap = self
            .icon
            .as_ref()
            .map(|_| metrics.content_gap)
            .unwrap_or(0.0);
        let reserved_inline = metrics.padding_inline * 2.0 + icon_extent + gap;
        let mut text = Text::new(self.label.clone());
        text.style = theme_text_style(theme);
        text.wrap = TextWrap::NoWrap;
        text.align = crate::InlineAlignment::Start;
        let text_layout = text.layout(
            text_system,
            resolved.direction,
            LogicalConstraints::loose(LogicalSize::new(
                subtract(constraints.max.width, reserved_inline),
                subtract(constraints.max.height, metrics.padding_block * 2.0),
            )),
        );
        let content_width = text_layout.size.width + icon_extent + gap;
        let content_height = text_layout.size.height.max(icon_extent);
        let size = constraints.constrain(LogicalSize::new(
            content_width + metrics.padding_inline * 2.0,
            (content_height + metrics.padding_block * 2.0).max(metrics.minimum_block_size),
        ));
        let content_start = ((size.width - content_width) * 0.5).max(0.0);
        let label_y = ((size.height - text_layout.size.height) * 0.5).max(0.0);
        let icon_y = ((size.height - icon_extent) * 0.5).max(0.0);
        let (label_x, icon_x) = match (self.icon.as_ref(), resolved.component.adornment) {
            (Some(_), PhysicalAdornmentPlacement::Right) => (
                content_start,
                Some(content_start + text_layout.size.width + gap),
            ),
            (Some(_), _) => (content_start + icon_extent + gap, Some(content_start)),
            (None, _) => (content_start, None),
        };
        let icon_layout = self.icon.as_ref().map(|icon| {
            icon.layout(
                resolved.direction,
                LogicalConstraints::tight(LogicalSize::new(icon_extent, icon_extent)),
            )
        });
        ButtonLayout {
            size,
            direction: resolved.direction,
            component: resolved.component,
            label: text_layout,
            label_origin: LogicalPoint::new(label_x, label_y),
            icon: icon_layout,
            icon_origin: icon_x.map(|x| LogicalPoint::new(x, icon_y)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IconButton {
    label: String,
    pub icon: Icon,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl IconButton {
    pub fn new(icon: Icon, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon,
            style: ComponentStyle::default(),
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
        let mut semantics = Semantics::new(SemanticRole::Button)
            .with_name(self.label.clone())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Activate);
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

    pub fn resolve(&self, theme: &ResolvedTheme, inherited_direction: Direction) -> ButtonStyle {
        let direction = self.direction.resolve(inherited_direction);
        ButtonStyle {
            direction,
            component: self.style.resolve(theme, direction),
        }
    }

    pub fn layout(
        &self,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> IconButtonLayout {
        let resolved = self.resolve(theme, inherited_direction);
        let extent = resolved.component.metrics.minimum_block_size;
        let size = constraints.constrain(LogicalSize::new(extent, extent));
        let icon_extent = resolved
            .component
            .metrics
            .icon_size
            .min(size.width)
            .min(size.height);
        let icon_origin = LogicalPoint::new(
            (size.width - icon_extent) * 0.5,
            (size.height - icon_extent) * 0.5,
        );
        IconButtonLayout {
            size,
            direction: resolved.direction,
            component: resolved.component,
            icon: self.icon.layout(
                resolved.direction,
                LogicalConstraints::tight(LogicalSize::new(icon_extent, icon_extent)),
            ),
            icon_origin,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonStyle {
    pub direction: Direction,
    pub component: ResolvedComponentStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub component: ResolvedComponentStyle,
    pub label: crate::TextLayout,
    pub label_origin: LogicalPoint,
    pub icon: Option<IconLayout>,
    pub icon_origin: Option<LogicalPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconButtonLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub component: ResolvedComponentStyle,
    pub icon: IconLayout,
    pub icon_origin: LogicalPoint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonDraws {
    pub background: RectDraw,
    pub text: Vec<TextDraw>,
    pub icon: Option<ImageDraw>,
}

impl ButtonLayout {
    pub fn draws(&self, button: &Button, origin: LogicalPoint) -> ButtonDraws {
        let foreground = faded(
            self.component.appearance.foreground.to_array(),
            self.component.appearance.opacity,
        );
        ButtonDraws {
            background: background_draw(self.size, origin, self.component),
            text: self
                .label
                .draws(button.label(), add(origin, self.label_origin), foreground),
            icon: self
                .icon
                .zip(self.icon_origin)
                .and_then(|(layout, offset)| {
                    button.icon().map(|icon| {
                        layout.draw(icon.source.clone(), add(origin, offset), foreground)
                    })
                }),
        }
    }
}

impl IconButtonLayout {
    pub fn draws(&self, button: &IconButton, origin: LogicalPoint) -> ButtonDraws {
        let foreground = faded(
            self.component.appearance.foreground.to_array(),
            self.component.appearance.opacity,
        );
        ButtonDraws {
            background: background_draw(self.size, origin, self.component),
            text: Vec::new(),
            icon: Some(self.icon.draw(
                button.icon.source.clone(),
                add(origin, self.icon_origin),
                foreground,
            )),
        }
    }
}

fn background_draw(
    size: LogicalSize,
    origin: LogicalPoint,
    component: ResolvedComponentStyle,
) -> RectDraw {
    let appearance = component.appearance;
    let background = appearance.state_layer.composite_over(appearance.background);
    RectDraw {
        position: [origin.x, origin.y],
        size: [size.width, size.height],
        radii: [component.metrics.corner_radius; 4],
        color: faded(background.to_array(), appearance.opacity),
        border_width: appearance.border_width,
        border_color: faded(appearance.border.to_array(), appearance.opacity),
    }
}

fn theme_text_style(theme: &ResolvedTheme) -> TextStyle {
    TextStyle {
        family: Some(theme.typography.family.clone()),
        font_size: theme.typography.size,
        line_height: theme.typography.line_height,
        letter_spacing: theme.typography.letter_spacing,
        weight: theme.typography.weight,
        ..TextStyle::default()
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
    use super::{Button, IconButton};
    use crate::{
        AdornmentPlacement, Direction, Icon, PhysicalAdornmentPlacement, PixelFormat, PixelImage,
        SemanticAction, SemanticRole, ThemeController, ThemeDefinition, UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    fn icon() -> Icon {
        Icon::new(PixelImage::new(1, 1, PixelFormat::Alpha8, vec![255]).unwrap()).unwrap()
    }

    #[test]
    fn buttons_expose_named_activation_and_disabled_semantics() {
        let mut button = Button::new("Save");
        assert_eq!(button.semantics().role, SemanticRole::Button);
        assert_eq!(button.semantics().name.as_deref(), Some("Save"));
        assert!(button.semantics().supports(SemanticAction::Activate));
        button.style.state.disabled = true;
        assert!(button.semantics().state.disabled);

        let icon_button = IconButton::new(icon(), "Open menu");
        assert_eq!(icon_button.semantics().name.as_deref(), Some("Open menu"));
        assert_eq!(icon_button.icon.semantics().name, None);
    }

    #[test]
    fn button_adornment_resolution_mirrors_with_inherited_direction() {
        let mut button = Button::new("Next");
        button.style.adornment = AdornmentPlacement::InlineEnd;
        assert_eq!(
            button.resolve(&theme(), Direction::Ltr).component.adornment,
            PhysicalAdornmentPlacement::Right
        );
        assert_eq!(
            button.resolve(&theme(), Direction::Rtl).component.adornment,
            PhysicalAdornmentPlacement::Left
        );
    }

    #[test]
    fn button_layout_mirrors_icon_position_and_emits_theme_paint() {
        let mut button = Button::new("Next").with_icon(icon(), AdornmentPlacement::InlineEnd);
        button.style.variant = crate::VisualVariant::Solid;
        let theme = theme();
        let constraints = crate::LogicalConstraints::unconstrained();
        let mut text_system = crate::TextSystem::new();
        let ltr = button.layout(&mut text_system, &theme, Direction::Ltr, constraints);
        let rtl = button.layout(&mut text_system, &theme, Direction::Rtl, constraints);
        assert!(ltr.icon_origin.unwrap().x > ltr.label_origin.x);
        assert!(rtl.icon_origin.unwrap().x < rtl.label_origin.x);
        assert_eq!(ltr.size, rtl.size);
        let draws = rtl.draws(&button, crate::LogicalPoint::new(4.0, 6.0));
        assert_eq!(draws.background.position, [4.0, 6.0]);
        assert_eq!(draws.text.len(), 1);
        assert_eq!(
            draws.icon.unwrap().tint,
            Some(theme.colors.on_primary.to_array())
        );
    }

    #[test]
    fn icon_button_is_square_and_centers_the_tinted_mask() {
        let button = IconButton::new(icon(), "Menu");
        let theme = theme();
        let layout = button.layout(
            &theme,
            Direction::Rtl,
            crate::LogicalConstraints::unconstrained(),
        );
        assert_eq!(layout.size.width, layout.size.height);
        let draw = layout.draws(&button, crate::LogicalPoint::default());
        assert!(draw.text.is_empty());
        assert_eq!(
            draw.icon.unwrap().bounds.size.width,
            layout.component.metrics.icon_size
        );
    }
}
