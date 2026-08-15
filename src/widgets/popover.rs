// popover.rs

use crate::{
    Direction, DirectionSetting, FocusPolicy, Key, KeyState, KeyboardEvent, LogicalPoint,
    LogicalRect, LogicalSize, RectDraw, ResolvedTheme, SemanticRole, Semantics, TooltipPlacement,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverAction {
    #[default]
    None,
    Opened,
    Dismissed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Popover {
    label: String,
    pub open: bool,
    pub content_size: LogicalSize,
    pub placement: TooltipPlacement,
    pub direction: DirectionSetting,
}

impl Popover {
    pub fn new(label: impl Into<String>, content_size: LogicalSize) -> Self {
        Self {
            label: label.into(),
            open: false,
            content_size,
            placement: TooltipPlacement::BlockEnd,
            direction: DirectionSetting::Inherit,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn open(&mut self) -> PopoverAction {
        if self.open {
            PopoverAction::None
        } else {
            self.open = true;
            PopoverAction::Opened
        }
    }

    pub fn dismiss(&mut self) -> PopoverAction {
        if self.open {
            self.open = false;
            PopoverAction::Dismissed
        } else {
            PopoverAction::None
        }
    }

    pub fn handle_key(&mut self, event: &KeyboardEvent) -> PopoverAction {
        if self.open
            && event.state == KeyState::Pressed
            && !event.repeat
            && event.key == Key::Escape
        {
            self.dismiss()
        } else {
            PopoverAction::None
        }
    }

    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Dialog).with_name(self.label.clone());
        semantics.state.hidden = !self.open;
        semantics
    }

    pub fn focus_policy(&self) -> FocusPolicy {
        if self.open {
            FocusPolicy::focusable()
        } else {
            FocusPolicy {
                hidden: true,
                ..FocusPolicy::default()
            }
        }
    }

    pub fn layout(
        &self,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        anchor: LogicalRect,
        viewport: LogicalSize,
    ) -> PopoverLayout {
        let direction = self.direction.resolve(inherited_direction);
        let panel_size = LogicalSize::new(
            self.content_size.width + theme.spacing.medium * 2.0,
            self.content_size.height + theme.spacing.medium * 2.0,
        );
        let gap = theme.spacing.extra_small;
        let preferred = preferred_origin(self.placement, direction, anchor, panel_size, gap);
        PopoverLayout {
            size: if self.open {
                viewport
            } else {
                LogicalSize::default()
            },
            direction,
            panel_origin: LogicalPoint::new(
                preferred
                    .x
                    .clamp(0.0, (viewport.width - panel_size.width).max(0.0)),
                preferred
                    .y
                    .clamp(0.0, (viewport.height - panel_size.height).max(0.0)),
            ),
            panel_size,
            content_origin: LogicalPoint::new(theme.spacing.medium, theme.spacing.medium),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopoverLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub panel_origin: LogicalPoint,
    pub panel_size: LogicalSize,
    pub content_origin: LogicalPoint,
}

impl PopoverLayout {
    pub fn draw(&self, popover: &Popover, theme: &ResolvedTheme) -> Option<RectDraw> {
        popover.open.then_some(RectDraw {
            position: [self.panel_origin.x, self.panel_origin.y],
            size: [self.panel_size.width, self.panel_size.height],
            radii: [theme.radii.large; 4],
            color: theme.colors.surface_elevated.to_array(),
            border_width: theme.borders.thin,
            border_color: theme.colors.border.to_array(),
        })
    }

    pub fn content_bounds(&self) -> LogicalRect {
        LogicalRect::new(
            LogicalPoint::new(
                self.panel_origin.x + self.content_origin.x,
                self.panel_origin.y + self.content_origin.y,
            ),
            LogicalSize::new(
                (self.panel_size.width - self.content_origin.x * 2.0).max(0.0),
                (self.panel_size.height - self.content_origin.y * 2.0).max(0.0),
            ),
        )
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
    use super::{Popover, PopoverAction};
    use crate::{
        Direction, Key, KeyboardEvent, LogicalPoint, LogicalRect, LogicalSize, SemanticRole,
        ThemeController, ThemeDefinition, TooltipPlacement, UserPreferences,
    };

    fn theme() -> crate::ResolvedTheme {
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default())
    }

    #[test]
    fn open_escape_and_dismissal_control_focus_and_semantics() {
        let mut popover = Popover::new("Formatting", LogicalSize::new(120.0, 80.0));
        assert!(popover.semantics().state.hidden);
        assert_eq!(popover.open(), PopoverAction::Opened);
        assert_eq!(popover.semantics().role, SemanticRole::Dialog);
        assert!(popover.focus_policy().focusable);
        assert_eq!(
            popover.handle_key(&KeyboardEvent::pressed(Key::Escape)),
            PopoverAction::Dismissed
        );
        assert!(!popover.focus_policy().focusable);
    }

    #[test]
    fn inline_placement_mirrors_and_clamps_to_viewport() {
        let mut popover = Popover::new("Formatting", LogicalSize::new(100.0, 50.0));
        popover.open = true;
        popover.placement = TooltipPlacement::InlineStart;
        let anchor = LogicalRect::new(LogicalPoint::new(100.0, 40.0), LogicalSize::new(20.0, 20.0));
        let viewport = LogicalSize::new(240.0, 120.0);
        let theme = theme();
        let ltr = popover.layout(&theme, Direction::Ltr, anchor, viewport);
        let rtl = popover.layout(&theme, Direction::Rtl, anchor, viewport);
        assert!(ltr.panel_origin.x < anchor.origin.x);
        assert!(rtl.panel_origin.x > anchor.origin.x);
        assert!(rtl.panel_origin.x + rtl.panel_size.width <= viewport.width);
        assert_eq!(ltr.content_bounds().size, popover.content_size);
    }

    #[test]
    fn closed_popover_has_no_overlay_geometry_or_paint() {
        let popover = Popover::new("Formatting", LogicalSize::new(100.0, 50.0));
        let layout = popover.layout(
            &theme(),
            Direction::Ltr,
            LogicalRect::new(LogicalPoint::default(), LogicalSize::default()),
            LogicalSize::new(240.0, 120.0),
        );
        assert_eq!(layout.size, LogicalSize::default());
        assert!(layout.draw(&popover, &theme()).is_none());
    }
}
