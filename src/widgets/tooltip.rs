// tooltip.rs

use crate::{
    Direction, DirectionSetting, LogicalConstraints, LogicalPoint, LogicalRect, LogicalSize,
    RectDraw, ResolvedTheme, SemanticRole, Semantics, Text, TextDraw, TextStyle, TextSystem,
    TextWrap,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TooltipPlacement {
    #[default]
    BlockStart,
    BlockEnd,
    InlineStart,
    InlineEnd,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tooltip {
    content: String,
    pub visible: bool,
    pub placement: TooltipPlacement,
    pub direction: DirectionSetting,
}

impl Tooltip {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            visible: false,
            placement: TooltipPlacement::BlockStart,
            direction: DirectionSetting::Inherit,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }
    pub fn show(&mut self) -> bool {
        let changed = !self.visible;
        self.visible = true;
        changed
    }
    pub fn hide(&mut self) -> bool {
        let changed = self.visible;
        self.visible = false;
        changed
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Text).with_name(self.content.clone());
        semantics.state.hidden = !self.visible;
        semantics
    }

    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        anchor: LogicalRect,
        viewport: LogicalSize,
    ) -> TooltipLayout {
        let direction = self.direction.resolve(inherited_direction);
        let inline = theme.spacing.small;
        let block = theme.spacing.extra_small;
        let mut text = Text::new(self.content.clone());
        text.direction = match direction {
            Direction::Ltr => DirectionSetting::Ltr,
            Direction::Rtl => DirectionSetting::Rtl,
        };
        text.wrap = TextWrap::Word;
        text.style = TextStyle {
            family: Some(theme.typography.family.clone()),
            font_size: (theme.typography.size - 2.0).max(1.0),
            line_height: (theme.typography.line_height - 2.0).max(1.0),
            letter_spacing: theme.typography.letter_spacing,
            weight: theme.typography.weight,
            ..TextStyle::default()
        };
        let text = text.layout(
            text_system,
            direction,
            LogicalConstraints::loose(LogicalSize::new(
                (viewport.width - inline * 2.0).clamp(0.0, 280.0),
                viewport.height,
            )),
        );
        let size = LogicalSize::new(
            text.size.width + inline * 2.0,
            text.size.height + block * 2.0,
        );
        let preferred = preferred_origin(self.placement, direction, anchor, size, block);
        TooltipLayout {
            size,
            direction,
            origin: LogicalPoint::new(
                preferred
                    .x
                    .clamp(0.0, (viewport.width - size.width).max(0.0)),
                preferred
                    .y
                    .clamp(0.0, (viewport.height - size.height).max(0.0)),
            ),
            text,
            text_origin: LogicalPoint::new(inline, block),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TooltipLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub origin: LogicalPoint,
    pub text: crate::TextLayout,
    pub text_origin: LogicalPoint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TooltipDraws {
    pub background: Option<RectDraw>,
    pub text: Vec<TextDraw>,
}

impl TooltipLayout {
    pub fn draws(&self, tooltip: &Tooltip, theme: &ResolvedTheme) -> TooltipDraws {
        if !tooltip.visible {
            return TooltipDraws {
                background: None,
                text: Vec::new(),
            };
        }
        TooltipDraws {
            background: Some(RectDraw {
                position: [self.origin.x, self.origin.y],
                size: [self.size.width, self.size.height],
                radii: [theme.radii.small; 4],
                color: theme.colors.text.to_array(),
                border_width: 0.0,
                border_color: [0.0; 4],
            }),
            text: self.text.draws(
                tooltip.content(),
                LogicalPoint::new(
                    self.origin.x + self.text_origin.x,
                    self.origin.y + self.text_origin.y,
                ),
                theme.colors.background.to_array(),
            ),
        }
    }
}

fn preferred_origin(
    placement: TooltipPlacement,
    direction: Direction,
    anchor: LogicalRect,
    size: LogicalSize,
    gap: f32,
) -> LogicalPoint {
    match placement {
        TooltipPlacement::BlockStart => LogicalPoint::new(
            anchor.origin.x + (anchor.size.width - size.width) * 0.5,
            anchor.origin.y - size.height - gap,
        ),
        TooltipPlacement::BlockEnd => LogicalPoint::new(
            anchor.origin.x + (anchor.size.width - size.width) * 0.5,
            anchor.origin.y + anchor.size.height + gap,
        ),
        TooltipPlacement::InlineStart if direction == Direction::Ltr => LogicalPoint::new(
            anchor.origin.x - size.width - gap,
            anchor.origin.y + (anchor.size.height - size.height) * 0.5,
        ),
        TooltipPlacement::InlineStart => LogicalPoint::new(
            anchor.origin.x + anchor.size.width + gap,
            anchor.origin.y + (anchor.size.height - size.height) * 0.5,
        ),
        TooltipPlacement::InlineEnd if direction == Direction::Ltr => LogicalPoint::new(
            anchor.origin.x + anchor.size.width + gap,
            anchor.origin.y + (anchor.size.height - size.height) * 0.5,
        ),
        TooltipPlacement::InlineEnd => LogicalPoint::new(
            anchor.origin.x - size.width - gap,
            anchor.origin.y + (anchor.size.height - size.height) * 0.5,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{Tooltip, TooltipPlacement};
    use crate::{
        Direction, LogicalPoint, LogicalRect, LogicalSize, SemanticRole, TextSystem,
        ThemeController, ThemeDefinition, UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn visibility_controls_semantics_and_paint() {
        let mut tooltip = Tooltip::new("More information");
        assert!(tooltip.semantics().state.hidden);
        assert!(tooltip.show());
        assert_eq!(tooltip.semantics().role, SemanticRole::Text);
        assert!(!tooltip.semantics().state.hidden);
        assert!(!tooltip.show());
        assert!(tooltip.hide());
    }

    #[test]
    fn inline_placement_mirrors_with_direction() {
        let mut tooltip = Tooltip::new("Info");
        tooltip.visible = true;
        tooltip.placement = TooltipPlacement::InlineStart;
        let anchor = LogicalRect::new(LogicalPoint::new(100.0, 40.0), LogicalSize::new(20.0, 20.0));
        let viewport = LogicalSize::new(240.0, 120.0);
        let theme = theme();
        let mut text_system = TextSystem::new();
        let ltr = tooltip.layout(&mut text_system, &theme, Direction::Ltr, anchor, viewport);
        let rtl = tooltip.layout(&mut text_system, &theme, Direction::Rtl, anchor, viewport);
        assert!(ltr.origin.x < anchor.origin.x);
        assert!(rtl.origin.x > anchor.origin.x);
        assert_eq!(ltr.size, rtl.size);
    }

    #[test]
    fn placement_clamps_and_hidden_tooltip_does_not_draw() {
        let tooltip = Tooltip::new("Long tooltip content");
        let theme = theme();
        let mut text_system = TextSystem::new();
        let viewport = LogicalSize::new(100.0, 60.0);
        let layout = tooltip.layout(
            &mut text_system,
            &theme,
            Direction::Ltr,
            LogicalRect::new(LogicalPoint::new(95.0, 0.0), LogicalSize::new(5.0, 5.0)),
            viewport,
        );
        assert!(layout.origin.x + layout.size.width <= viewport.width);
        assert!(layout.origin.y >= 0.0);
        assert!(layout.draws(&tooltip, &theme).background.is_none());
    }
}
