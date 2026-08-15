// data_display.rs

use crate::{
    ArrowKey, Button, ButtonDraws, ButtonLayout, ComponentStyle, Direction, DirectionSetting,
    FocusPolicy, Image, ImageDraw, ImageFit, ImageLayout, Key, KeyState, KeyboardEvent,
    LogicalConstraints, LogicalPoint, LogicalRect, LogicalSize, PixelImage, RectDraw,
    ResolvedTheme, SemanticAction, SemanticColorToken, SemanticRole, Semantics, TextSystem,
    VisualVariant,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Avatar {
    pub image: Image,
    pub size: f32,
    pub radius: f32,
}

impl Avatar {
    pub fn new(source: PixelImage) -> Self {
        let mut image = Image::new(source);
        image.fit = ImageFit::Cover;
        Self {
            image,
            size: 48.0,
            radius: 24.0,
        }
    }
    pub fn with_alternative_text(mut self, text: impl Into<String>) -> Self {
        self.image = self.image.with_alternative_text(text);
        self
    }
    pub fn semantics(&self) -> Semantics {
        self.image.semantics()
    }
    pub fn layout(
        &self,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> AvatarLayout {
        let extent = if self.size.is_finite() {
            self.size.max(0.0)
        } else {
            0.0
        };
        let size = constraints.constrain(LogicalSize::new(extent, extent));
        AvatarLayout {
            size,
            image: self
                .image
                .layout(inherited_direction, LogicalConstraints::tight(size)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarLayout {
    pub size: LogicalSize,
    pub image: ImageLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AvatarDraws {
    pub background: RectDraw,
    pub image: ImageDraw,
}

impl AvatarLayout {
    pub fn draws(
        &self,
        avatar: &Avatar,
        origin: LogicalPoint,
        theme: &ResolvedTheme,
    ) -> AvatarDraws {
        AvatarDraws {
            background: RectDraw {
                position: [origin.x, origin.y],
                size: [self.size.width, self.size.height],
                radii: [avatar.radius.max(0.0); 4],
                color: theme.colors.surface_elevated.to_array(),
                border_width: theme.borders.regular,
                border_color: theme.colors.border.to_array(),
            },
            image: self.image.draw(avatar.image.source.clone(), origin),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    label: Option<String>,
    pub size: LogicalSize,
    pub padding: f32,
    pub radius: f32,
    pub color: SemanticColorToken,
}

impl Card {
    pub fn new(size: LogicalSize) -> Self {
        Self {
            label: None,
            size,
            padding: 16.0,
            radius: 10.0,
            color: SemanticColorToken::SurfaceElevated,
        }
    }
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.set_label(Some(label.into()));
        self
    }
    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label.filter(|label| !label.trim().is_empty());
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Group);
        if let Some(label) = &self.label {
            semantics.set_name(label.clone());
        }
        semantics
    }
    pub fn layout(&self, constraints: LogicalConstraints) -> CardLayout {
        CardLayout {
            size: constraints.constrain(self.size),
            padding: self.padding.max(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardLayout {
    pub size: LogicalSize,
    pub padding: f32,
}

impl CardLayout {
    pub fn draw(&self, card: &Card, origin: LogicalPoint, theme: &ResolvedTheme) -> RectDraw {
        RectDraw {
            position: [origin.x, origin.y],
            size: [self.size.width, self.size.height],
            radii: [card.radius.max(0.0); 4],
            color: theme.colors.resolve(card.color).to_array(),
            border_width: theme.borders.regular,
            border_color: theme.colors.border.to_array(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stat {
    title: String,
    value: String,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Stat {
    pub fn new(title: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            value: value.into(),
            style: ComponentStyle {
                variant: VisualVariant::Soft,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        }
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
    pub fn semantics(&self) -> Semantics {
        Semantics::new(SemanticRole::Group)
            .with_name(self.title.clone())
            .with_value(self.value.clone())
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> MetricDisplayLayout {
        metric_layout(
            self.display_text(),
            self.style,
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &MetricDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(self.display_text(), self.style, self.direction, origin)
    }
    fn display_text(&self) -> String {
        format!("{}\n{}", self.title, self.value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Countdown {
    label: String,
    remaining_seconds: u64,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Countdown {
    pub fn new(label: impl Into<String>, remaining_seconds: u64) -> Self {
        Self {
            label: label.into(),
            remaining_seconds,
            style: ComponentStyle {
                variant: VisualVariant::Outline,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        }
    }
    pub fn remaining_seconds(&self) -> u64 {
        self.remaining_seconds
    }
    pub fn set_remaining_seconds(&mut self, remaining_seconds: u64) -> bool {
        let changed = self.remaining_seconds != remaining_seconds;
        self.remaining_seconds = remaining_seconds;
        changed
    }
    pub fn tick(&mut self, elapsed_seconds: u64) -> bool {
        self.set_remaining_seconds(self.remaining_seconds.saturating_sub(elapsed_seconds))
    }
    pub fn formatted_remaining(&self) -> String {
        let hours = self.remaining_seconds / 3600;
        let minutes = self.remaining_seconds % 3600 / 60;
        let seconds = self.remaining_seconds % 60;
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }
    pub fn semantics(&self) -> Semantics {
        Semantics::new(SemanticRole::Timer)
            .with_name(self.label.clone())
            .with_value(self.formatted_remaining())
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> MetricDisplayLayout {
        metric_layout(
            self.formatted_remaining(),
            self.style,
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &MetricDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.formatted_remaining(),
            self.style,
            self.direction,
            origin,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetricDisplayLayout {
    pub button: ButtonLayout,
}

impl MetricDisplayLayout {
    pub fn draws(
        &self,
        text: String,
        style: ComponentStyle,
        direction: DirectionSetting,
        origin: LogicalPoint,
    ) -> ButtonDraws {
        let mut button = Button::new(text);
        button.style = style;
        button.direction = direction;
        self.button.draws(&button, origin)
    }
}

fn metric_layout(
    text: String,
    style: ComponentStyle,
    direction: DirectionSetting,
    text_system: &mut TextSystem,
    theme: &ResolvedTheme,
    inherited_direction: Direction,
    constraints: LogicalConstraints,
) -> MetricDisplayLayout {
    let mut button = Button::new(text);
    button.style = style;
    button.direction = direction;
    MetricDisplayLayout {
        button: button.layout(text_system, theme, inherited_direction, constraints),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatBubble {
    sender: String,
    message: String,
    pub outgoing: bool,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl ChatBubble {
    pub fn new(sender: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            sender: sender.into(),
            message: message.into(),
            outgoing: false,
            style: ComponentStyle {
                variant: VisualVariant::Soft,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        }
    }
    pub fn sender(&self) -> &str {
        &self.sender
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn semantics(&self) -> Semantics {
        Semantics::new(SemanticRole::Group)
            .with_name(self.sender.clone())
            .with_value(self.message.clone())
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> MetricDisplayLayout {
        metric_layout(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &MetricDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            origin,
        )
    }
    fn display_text(&self) -> String {
        format!("{}\n{}", self.sender, self.message)
    }
    fn resolved_style(&self) -> ComponentStyle {
        let mut style = self.style;
        style.variant = if self.outgoing {
            VisualVariant::Solid
        } else {
            VisualVariant::Soft
        };
        style
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiffError;

impl std::fmt::Display for DiffError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("diff split must be finite and between zero and one")
    }
}

impl std::error::Error for DiffError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Diff {
    pub before: PixelImage,
    pub after: PixelImage,
    split: f32,
    pub direction: DirectionSetting,
    label: String,
}

impl Diff {
    pub fn new(
        label: impl Into<String>,
        before: PixelImage,
        after: PixelImage,
        split: f32,
    ) -> Result<Self, DiffError> {
        if !split.is_finite() || !(0.0..=1.0).contains(&split) {
            return Err(DiffError);
        }
        Ok(Self {
            before,
            after,
            split,
            direction: DirectionSetting::Inherit,
            label: label.into(),
        })
    }
    pub fn split(&self) -> f32 {
        self.split
    }
    pub fn set_split(&mut self, split: f32) -> Result<bool, DiffError> {
        if !split.is_finite() || !(0.0..=1.0).contains(&split) {
            return Err(DiffError);
        }
        let changed = self.split != split;
        self.split = split;
        Ok(changed)
    }
    pub fn semantics(&self) -> Semantics {
        Semantics::new(SemanticRole::Group)
            .with_name(self.label.clone())
            .with_virtual_child(Semantics::new(SemanticRole::Image).with_name("Before"))
            .with_virtual_child(Semantics::new(SemanticRole::Image).with_name("After"))
    }
    pub fn layout(
        &self,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> DiffLayout {
        let intrinsic = LogicalSize::new(
            self.before.width().max(self.after.width()) as f32,
            self.before.height().max(self.after.height()) as f32,
        );
        DiffLayout {
            size: constraints.constrain(intrinsic),
            direction: self.direction.resolve(inherited_direction),
            split: self.split,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiffLayout {
    pub size: LogicalSize,
    pub direction: Direction,
    pub split: f32,
}

impl DiffLayout {
    pub fn draws(&self, diff: &Diff, origin: LogicalPoint) -> [ImageDraw; 2] {
        let split_width = self.size.width * self.split;
        let (before_x, after_x) = match self.direction {
            Direction::Ltr => (origin.x, origin.x + split_width),
            Direction::Rtl => (origin.x + self.size.width - split_width, origin.x),
        };
        let bounds = LogicalRect::new(origin, self.size);
        [
            ImageDraw {
                image: diff.before.clone(),
                bounds,
                clip: LogicalRect::new(
                    LogicalPoint::new(before_x, origin.y),
                    LogicalSize::new(split_width, self.size.height),
                ),
                mirror_horizontal: false,
                tint: None,
            },
            ImageDraw {
                image: diff.after.clone(),
                bounds,
                clip: LogicalRect::new(
                    LogicalPoint::new(after_x, origin.y),
                    LogicalSize::new(self.size.width - split_width, self.size.height),
                ),
                mirror_horizontal: false,
                tint: None,
            },
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableError;

impl std::fmt::Display for TableError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("table rows must match the non-empty column count")
    }
}

impl std::error::Error for TableError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Table {
    pub fn new(
        columns: impl IntoIterator<Item = impl Into<String>>,
        rows: impl IntoIterator<Item = impl IntoIterator<Item = impl Into<String>>>,
    ) -> Result<Self, TableError> {
        let columns = columns.into_iter().map(Into::into).collect::<Vec<_>>();
        let rows = rows
            .into_iter()
            .map(|row| row.into_iter().map(Into::into).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        if columns.is_empty() || rows.iter().any(|row| row.len() != columns.len()) {
            return Err(TableError);
        }
        Ok(Self {
            columns,
            rows,
            style: ComponentStyle {
                variant: VisualVariant::Outline,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        })
    }
    pub fn columns(&self) -> &[String] {
        &self.columns
    }
    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }
    pub fn semantics(&self) -> Semantics {
        self.rows.iter().enumerate().fold(
            Semantics::new(SemanticRole::Table).with_name("Table"),
            |semantics, (row_index, row)| {
                row.iter()
                    .enumerate()
                    .fold(semantics, |semantics, (column_index, value)| {
                        semantics.with_virtual_child(
                            Semantics::new(SemanticRole::Cell)
                                .with_name(format!(
                                    "{} {}",
                                    self.columns[column_index],
                                    row_index + 1
                                ))
                                .with_value(value.clone()),
                        )
                    })
            },
        )
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> MetricDisplayLayout {
        metric_layout(
            self.display_text(),
            self.style,
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &MetricDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(self.display_text(), self.style, self.direction, origin)
    }
    fn display_text(&self) -> String {
        std::iter::once(self.columns.join("  |  "))
            .chain(self.rows.iter().map(|row| row.join("  |  ")))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineItem {
    pub label: String,
    pub detail: String,
}

impl TimelineItem {
    pub fn new(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Timeline {
    items: Vec<TimelineItem>,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Timeline {
    pub fn new(items: impl IntoIterator<Item = TimelineItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            style: ComponentStyle {
                variant: VisualVariant::Ghost,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        }
    }
    pub fn items(&self) -> &[TimelineItem] {
        &self.items
    }
    pub fn semantics(&self) -> Semantics {
        self.items.iter().fold(
            Semantics::new(SemanticRole::List).with_name("Timeline"),
            |semantics, item| {
                semantics.with_virtual_child(
                    Semantics::new(SemanticRole::ListItem)
                        .with_name(item.label.clone())
                        .with_value(item.detail.clone()),
                )
            },
        )
    }
    pub fn layout(
        &self,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        inherited_direction: Direction,
        constraints: LogicalConstraints,
    ) -> MetricDisplayLayout {
        metric_layout(
            self.display_text(),
            self.style,
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &MetricDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(self.display_text(), self.style, self.direction, origin)
    }
    fn display_text(&self) -> String {
        self.items
            .iter()
            .map(|item| format!("{}  —  {}", item.label, item.detail))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Accordion {
    label: String,
    content: String,
    pub open: bool,
    pub disabled: bool,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Accordion {
    pub fn new(label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            content: content.into(),
            open: false,
            disabled: false,
            style: ComponentStyle {
                variant: VisualVariant::Outline,
                ..ComponentStyle::default()
            },
            direction: DirectionSetting::Inherit,
        }
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn activate(&mut self) -> bool {
        if self.disabled {
            false
        } else {
            self.open = !self.open;
            true
        }
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Button)
            .with_name(self.label.clone())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Activate);
        semantics.state.expanded = Some(self.open);
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
    ) -> MetricDisplayLayout {
        metric_layout(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &MetricDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            origin,
        )
    }
    fn display_text(&self) -> String {
        if self.open {
            format!("{}\n{}", self.label, self.content)
        } else {
            self.label.clone()
        }
    }
    fn resolved_style(&self) -> ComponentStyle {
        let mut style = self.style;
        style.state.disabled = self.disabled;
        style.state.selected = self.open;
        style
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CarouselError;

impl std::fmt::Display for CarouselError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("carousel requires at least one item")
    }
}

impl std::error::Error for CarouselError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Carousel {
    label: String,
    items: Vec<String>,
    active: usize,
    pub disabled: bool,
    pub style: ComponentStyle,
    pub direction: DirectionSetting,
}

impl Carousel {
    pub fn new(
        label: impl Into<String>,
        items: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, CarouselError> {
        let items = items.into_iter().map(Into::into).collect::<Vec<_>>();
        if items.is_empty() {
            return Err(CarouselError);
        }
        Ok(Self {
            label: label.into(),
            items,
            active: 0,
            disabled: false,
            style: ComponentStyle::default(),
            direction: DirectionSetting::Inherit,
        })
    }
    pub fn active_index(&self) -> usize {
        self.active
    }
    pub fn active_item(&self) -> &str {
        &self.items[self.active]
    }
    pub fn next_item(&mut self) -> bool {
        if self.disabled {
            false
        } else {
            self.active = (self.active + 1) % self.items.len();
            true
        }
    }
    pub fn previous_item(&mut self) -> bool {
        if self.disabled {
            false
        } else {
            self.active = (self.active + self.items.len() - 1) % self.items.len();
            true
        }
    }
    pub fn handle_key(&mut self, event: &KeyboardEvent, direction: Direction) -> bool {
        if event.state != KeyState::Pressed {
            return false;
        }
        match event.key {
            Key::Arrow(ArrowKey::Right) if direction == Direction::Ltr => self.next_item(),
            Key::Arrow(ArrowKey::Right) => self.previous_item(),
            Key::Arrow(ArrowKey::Left) if direction == Direction::Ltr => self.previous_item(),
            Key::Arrow(ArrowKey::Left) => self.next_item(),
            Key::Home if !self.disabled => {
                let changed = self.active != 0;
                self.active = 0;
                changed
            }
            Key::End if !self.disabled => {
                let last = self.items.len() - 1;
                let changed = self.active != last;
                self.active = last;
                changed
            }
            _ => false,
        }
    }
    pub fn semantics(&self) -> Semantics {
        let mut semantics = Semantics::new(SemanticRole::Group)
            .with_name(self.label.clone())
            .with_value(self.active_item().to_owned())
            .with_action(SemanticAction::Focus)
            .with_action(SemanticAction::Increment)
            .with_action(SemanticAction::Decrement);
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
    ) -> MetricDisplayLayout {
        metric_layout(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            text_system,
            theme,
            inherited_direction,
            constraints,
        )
    }
    pub fn draws(&self, layout: &MetricDisplayLayout, origin: LogicalPoint) -> ButtonDraws {
        layout.draws(
            self.display_text(),
            self.resolved_style(),
            self.direction,
            origin,
        )
    }
    fn display_text(&self) -> String {
        format!("{}\n{}", self.label, self.active_item())
    }
    fn resolved_style(&self) -> ComponentStyle {
        let mut style = self.style;
        style.state.disabled = self.disabled;
        style
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Accordion, Avatar, Card, Carousel, ChatBubble, Countdown, Diff, Stat, Table, Timeline,
        TimelineItem,
    };
    use crate::{
        ArrowKey, Direction, Key, KeyboardEvent, LogicalConstraints, LogicalSize, PixelFormat,
        PixelImage, SemanticRole,
    };

    #[test]
    fn avatar_preserves_image_semantics_and_fixed_square_layout() {
        let pixels = PixelImage::new(2, 1, PixelFormat::Rgba8, vec![255; 8]).unwrap();
        let avatar = Avatar::new(pixels).with_alternative_text("Profile photo");
        let layout = avatar.layout(Direction::Rtl, LogicalConstraints::unconstrained());
        assert_eq!(avatar.semantics().role, SemanticRole::Image);
        assert_eq!(layout.size, LogicalSize::new(48.0, 48.0));
        assert_eq!(layout.image.direction, Direction::Rtl);
    }

    #[test]
    fn card_is_a_named_group_with_normalized_padding() {
        let mut card = Card::new(LogicalSize::new(240.0, 120.0)).with_label("Account");
        card.padding = -4.0;
        assert_eq!(card.semantics().role, SemanticRole::Group);
        assert_eq!(
            card.layout(LogicalConstraints::unconstrained()).padding,
            0.0
        );
    }

    #[test]
    fn stat_and_countdown_preserve_named_values_and_saturating_time() {
        let mut stat = Stat::new("Revenue", "$42");
        stat.set_value("$43");
        assert_eq!(stat.semantics().value.as_deref(), Some("$43"));
        let mut countdown = Countdown::new("Offer ends", 3661);
        assert_eq!(countdown.semantics().role, SemanticRole::Timer);
        assert_eq!(countdown.formatted_remaining(), "01:01:01");
        assert!(countdown.tick(4000));
        assert_eq!(countdown.remaining_seconds(), 0);
        assert!(!countdown.tick(1));
    }

    #[test]
    fn chat_bubble_and_diff_expose_grouped_content() {
        let bubble = ChatBubble::new("Mina", "Hello");
        assert_eq!(bubble.semantics().name.as_deref(), Some("Mina"));
        let before = PixelImage::new(4, 2, PixelFormat::Rgba8, vec![0; 32]).unwrap();
        let after = PixelImage::new(4, 2, PixelFormat::Rgba8, vec![255; 32]).unwrap();
        let diff = Diff::new("Comparison", before, after, 0.25).unwrap();
        let ltr = diff.layout(Direction::Ltr, LogicalConstraints::unconstrained());
        let rtl = diff.layout(Direction::Rtl, LogicalConstraints::unconstrained());
        let ltr_draws = ltr.draws(&diff, Default::default());
        let rtl_draws = rtl.draws(&diff, Default::default());
        assert_eq!(ltr_draws[0].clip.origin.x, 0.0);
        assert_eq!(rtl_draws[0].clip.origin.x, 3.0);
        assert_eq!(diff.semantics().virtual_children().len(), 2);
    }

    #[test]
    fn table_and_timeline_validate_and_expose_collection_semantics() {
        let table = Table::new(["Name", "Role"], [["Mina", "Admin"]]).unwrap();
        assert_eq!(table.semantics().role, SemanticRole::Table);
        assert_eq!(table.semantics().virtual_children().len(), 2);
        assert!(Table::new(["Name", "Role"], [["Mina"]]).is_err());
        let timeline = Timeline::new([
            TimelineItem::new("Created", "09:00"),
            TimelineItem::new("Published", "10:30"),
        ]);
        assert_eq!(timeline.semantics().role, SemanticRole::List);
        assert_eq!(timeline.semantics().virtual_children().len(), 2);
    }

    #[test]
    fn accordion_and_carousel_support_directional_keyboard_state() {
        let mut accordion = Accordion::new("Details", "More information");
        assert!(accordion.activate());
        assert_eq!(accordion.semantics().state.expanded, Some(true));
        let mut carousel = Carousel::new("Gallery", ["One", "Two", "Three"]).unwrap();
        assert!(carousel.handle_key(
            &KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right)),
            Direction::Ltr
        ));
        assert_eq!(carousel.active_index(), 1);
        assert!(carousel.handle_key(
            &KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right)),
            Direction::Rtl
        ));
        assert_eq!(carousel.active_index(), 0);
        assert!(Carousel::new("Empty", std::iter::empty::<String>()).is_err());
    }
}
