// widget.rs

use std::collections::HashMap;

use crate::{
    Direction, FrameNode, FrameSnapshot, ImageDraw, LayoutChild, LogicalConstraints, LogicalPoint,
    LogicalRect, LogicalSize, Overflow, RectDraw, ResolvedTheme, SemanticSnapshot, Semantics,
    StackChild, TextDraw, TextSystem, WidgetGeometry, WidgetId, WidgetTree,
};

use super::{
    Button, ButtonLayout, Checkbox, CheckboxLayout, Column, Container, Divider, Icon, IconButton,
    IconButtonLayout, IconLayout, Image, ImageLayout, Radio, RadioLayout, Row, ScrollLayout,
    ScrollView, Spacer, Stack, StackLayout, Surface, Text, TextLayout,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Widget {
    Text(Text),
    Image(Image),
    Icon(Icon),
    Spacer(Spacer),
    Divider(Divider),
    Surface(Surface),
    Container(Container),
    Row(Row),
    Column(Column),
    Stack(Stack),
    ScrollView(ScrollView),
    Button(Button),
    IconButton(IconButton),
    Checkbox(Checkbox),
    Radio(Radio),
}

impl From<Text> for Widget {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}

impl From<Image> for Widget {
    fn from(image: Image) -> Self {
        Self::Image(image)
    }
}

impl From<Icon> for Widget {
    fn from(icon: Icon) -> Self {
        Self::Icon(icon)
    }
}
impl From<Spacer> for Widget {
    fn from(spacer: Spacer) -> Self {
        Self::Spacer(spacer)
    }
}
impl From<Divider> for Widget {
    fn from(divider: Divider) -> Self {
        Self::Divider(divider)
    }
}
impl From<Surface> for Widget {
    fn from(surface: Surface) -> Self {
        Self::Surface(surface)
    }
}
impl From<Container> for Widget {
    fn from(value: Container) -> Self {
        Self::Container(value)
    }
}
impl From<Row> for Widget {
    fn from(value: Row) -> Self {
        Self::Row(value)
    }
}
impl From<Column> for Widget {
    fn from(value: Column) -> Self {
        Self::Column(value)
    }
}
impl From<Stack> for Widget {
    fn from(value: Stack) -> Self {
        Self::Stack(value)
    }
}
impl From<ScrollView> for Widget {
    fn from(value: ScrollView) -> Self {
        Self::ScrollView(value)
    }
}
impl From<Button> for Widget {
    fn from(value: Button) -> Self {
        Self::Button(value)
    }
}
impl From<IconButton> for Widget {
    fn from(value: IconButton) -> Self {
        Self::IconButton(value)
    }
}
impl From<Checkbox> for Widget {
    fn from(value: Checkbox) -> Self {
        Self::Checkbox(value)
    }
}
impl From<Radio> for Widget {
    fn from(value: Radio) -> Self {
        Self::Radio(value)
    }
}

impl Widget {
    pub fn semantics(&self) -> Semantics {
        match self {
            Self::Text(text) => text.semantics(),
            Self::Image(image) => image.semantics(),
            Self::Icon(icon) => icon.semantics(),
            Self::Button(button) => button.semantics(),
            Self::IconButton(button) => button.semantics(),
            Self::Checkbox(checkbox) => checkbox.semantics(),
            Self::Radio(radio) => radio.semantics(),
            Self::Spacer(_)
            | Self::Divider(_)
            | Self::Surface(_)
            | Self::Container(_)
            | Self::Row(_)
            | Self::Column(_)
            | Self::Stack(_)
            | Self::ScrollView(_) => Semantics::default(),
        }
    }

    pub fn focus_policy(&self) -> crate::FocusPolicy {
        match self {
            Self::Button(button) => button.focus_policy(),
            Self::IconButton(button) => button.focus_policy(),
            Self::Checkbox(checkbox) => checkbox.focus_policy(),
            Self::Radio(radio) => radio.focus_policy(),
            _ => crate::FocusPolicy::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidgetPlacement {
    pub origin: LogicalPoint,
    pub constraints: LogicalConstraints,
    pub inherited_direction: Direction,
}

impl WidgetPlacement {
    pub fn new(
        origin: LogicalPoint,
        constraints: LogicalConstraints,
        inherited_direction: Direction,
    ) -> Self {
        Self {
            origin,
            constraints,
            inherited_direction,
        }
    }
}

#[derive(Clone, Debug)]
enum WidgetLayout {
    Text(TextLayout),
    Image(ImageLayout),
    Icon(IconLayout),
    Spacer(crate::LogicalSize),
    Divider(crate::LogicalSize),
    Surface(crate::LogicalSize),
    Container(crate::LogicalSize),
    Row(LogicalSize),
    Column(LogicalSize),
    Stack(StackLayout),
    ScrollView(ScrollLayout),
    Button(ButtonLayout),
    IconButton(IconButtonLayout),
    Checkbox(CheckboxLayout),
    Radio(RadioLayout),
}

#[derive(Clone, Debug)]
pub struct WidgetFrame {
    pub geometry: FrameSnapshot,
    pub semantics: SemanticSnapshot,
    pub rectangles: Vec<RectDraw>,
    pub text: Vec<TextDraw>,
    pub images: Vec<ImageDraw>,
}

impl WidgetFrame {
    pub fn build(
        tree: &WidgetTree<Widget>,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        mut place: impl FnMut(WidgetId, Option<FrameNode>) -> WidgetPlacement,
    ) -> Self {
        let mut layouts = HashMap::with_capacity(tree.len());
        let geometry = FrameSnapshot::build(tree, |id, parent| {
            let placement = place(id, parent);
            let layout = match &tree.get(id).unwrap().state {
                Widget::Text(text) => WidgetLayout::Text(text.layout(
                    text_system,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Image(image) => WidgetLayout::Image(
                    image.layout(placement.inherited_direction, placement.constraints),
                ),
                Widget::Icon(icon) => WidgetLayout::Icon(
                    icon.layout(placement.inherited_direction, placement.constraints),
                ),
                Widget::Spacer(spacer) => {
                    WidgetLayout::Spacer(spacer.layout(placement.constraints))
                }
                Widget::Divider(divider) => {
                    WidgetLayout::Divider(divider.layout(placement.constraints))
                }
                Widget::Surface(surface) => {
                    WidgetLayout::Surface(surface.layout(placement.constraints))
                }
                Widget::Container(container) => {
                    WidgetLayout::Container(container.layout(placement.constraints))
                }
                Widget::Row(row) => WidgetLayout::Row(
                    row.layout(placement.inherited_direction, &[], placement.constraints)
                        .size,
                ),
                Widget::Column(column) => WidgetLayout::Column(
                    column
                        .layout(placement.inherited_direction, &[], placement.constraints)
                        .size,
                ),
                Widget::Stack(stack) => WidgetLayout::Stack(stack.layout(
                    placement.inherited_direction,
                    &[],
                    placement.constraints,
                )),
                Widget::ScrollView(scroll) => WidgetLayout::ScrollView(scroll.layout(
                    placement.inherited_direction,
                    LogicalSize::default(),
                    placement.constraints,
                )),
                Widget::Button(button) => WidgetLayout::Button(button.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::IconButton(button) => WidgetLayout::IconButton(button.layout(
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Checkbox(checkbox) => WidgetLayout::Checkbox(checkbox.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
                Widget::Radio(radio) => WidgetLayout::Radio(radio.layout(
                    text_system,
                    theme,
                    placement.inherited_direction,
                    placement.constraints,
                )),
            };
            let size = match &layout {
                WidgetLayout::Text(layout) => layout.size,
                WidgetLayout::Image(layout) => layout.size,
                WidgetLayout::Icon(layout) => layout.size(),
                WidgetLayout::Spacer(size)
                | WidgetLayout::Divider(size)
                | WidgetLayout::Surface(size)
                | WidgetLayout::Container(size)
                | WidgetLayout::Row(size)
                | WidgetLayout::Column(size) => *size,
                WidgetLayout::Stack(layout) => layout.size,
                WidgetLayout::ScrollView(layout) => layout.viewport,
                WidgetLayout::Button(layout) => layout.size,
                WidgetLayout::IconButton(layout) => layout.size,
                WidgetLayout::Checkbox(layout) => layout.size,
                WidgetLayout::Radio(layout) => layout.size,
            };
            layouts.insert(id, (placement.origin, layout));
            let mut geometry = WidgetGeometry::new(LogicalRect::new(placement.origin, size));
            if matches!(&layouts[&id].1, WidgetLayout::ScrollView(_)) {
                geometry.overflow = Overflow::Clip;
            }
            geometry
        });
        let semantics = SemanticSnapshot::build(tree, |_, widget| widget.semantics());
        let mut rectangles = Vec::new();
        let mut text = Vec::new();
        let mut images = Vec::new();
        geometry.paint(|node| {
            let (origin, layout) = layouts.get(&node.id).unwrap();
            match layout {
                WidgetLayout::Text(layout) => {
                    let widget = &tree.get(node.id).unwrap().state;
                    let Widget::Text(widget) = widget else {
                        unreachable!();
                    };
                    text.extend(layout.draws(
                        widget.content(),
                        *origin,
                        theme.colors.resolve(layout.color).to_array(),
                    ));
                }
                WidgetLayout::Image(layout) => {
                    let widget = &tree.get(node.id).unwrap().state;
                    let Widget::Image(widget) = widget else {
                        unreachable!();
                    };
                    images.push(layout.draw(widget.source.clone(), *origin));
                }
                WidgetLayout::Icon(layout) => {
                    let widget = &tree.get(node.id).unwrap().state;
                    let Widget::Icon(widget) = widget else {
                        unreachable!();
                    };
                    images.push(layout.draw(
                        widget.source.clone(),
                        *origin,
                        theme.colors.resolve(widget.color).to_array(),
                    ));
                }
                WidgetLayout::Spacer(_) => {}
                WidgetLayout::Divider(size) => {
                    let widget = &tree.get(node.id).unwrap().state;
                    let Widget::Divider(widget) = widget else {
                        unreachable!()
                    };
                    rectangles.push(widget.draw(
                        *origin,
                        *size,
                        theme.colors.resolve(widget.color).to_array(),
                    ));
                }
                WidgetLayout::Surface(size) => {
                    let Widget::Surface(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    rectangles.push(widget.draw(*origin, *size, theme));
                }
                WidgetLayout::Container(_) => {}
                WidgetLayout::Row(_) | WidgetLayout::Column(_) => {}
                WidgetLayout::Stack(_) | WidgetLayout::ScrollView(_) => {}
                WidgetLayout::Button(layout) => {
                    let Widget::Button(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin);
                    rectangles.push(draws.background);
                    text.extend(draws.text);
                    images.extend(draws.icon);
                }
                WidgetLayout::IconButton(layout) => {
                    let Widget::IconButton(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin);
                    rectangles.push(draws.background);
                    images.extend(draws.icon);
                }
                WidgetLayout::Checkbox(layout) => {
                    let Widget::Checkbox(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin, theme);
                    rectangles.extend(draws.indicator);
                    text.extend(draws.label);
                }
                WidgetLayout::Radio(layout) => {
                    let Widget::Radio(widget) = &tree.get(node.id).unwrap().state else {
                        unreachable!()
                    };
                    let draws = layout.draws(widget, *origin, theme);
                    rectangles.extend(draws.indicator);
                    text.extend(draws.label);
                }
            }
        });
        rectangles.shrink_to_fit();
        images.shrink_to_fit();

        Self {
            geometry,
            semantics,
            rectangles,
            text,
            images,
        }
    }

    pub fn build_composed(
        tree: &WidgetTree<Widget>,
        text_system: &mut TextSystem,
        theme: &ResolvedTheme,
        root: WidgetPlacement,
    ) -> Self {
        let direction = root.inherited_direction;
        let mut measured = HashMap::with_capacity(tree.len());
        let ids = tree.depth_first(tree.root()).collect::<Vec<_>>();
        for id in ids.iter().rev().copied() {
            let node = tree.get(id).unwrap();
            let children = node
                .children()
                .iter()
                .map(|child| LayoutChild::new(measured[child]))
                .collect::<Vec<_>>();
            let constraints = if id == tree.root() {
                root.constraints
            } else {
                LogicalConstraints::unconstrained()
            };
            let size = match &node.state {
                Widget::Text(widget) => widget.layout(text_system, direction, constraints).size,
                Widget::Image(widget) => widget.layout(direction, constraints).size,
                Widget::Icon(widget) => widget.layout(direction, constraints).size(),
                Widget::Spacer(widget) => widget.layout(constraints),
                Widget::Divider(widget) => widget.layout(constraints),
                Widget::Surface(widget) => widget.layout(constraints),
                Widget::Container(widget) => widget.layout(constraints),
                Widget::Row(widget) => widget.layout(direction, &children, constraints).size,
                Widget::Column(widget) => widget.layout(direction, &children, constraints).size,
                Widget::Stack(widget) => {
                    let children = children
                        .iter()
                        .map(|child| StackChild::new(child.preferred))
                        .collect::<Vec<_>>();
                    widget.layout(direction, &children, constraints).size
                }
                Widget::ScrollView(widget) => {
                    let content = children.iter().fold(LogicalSize::default(), |size, child| {
                        LogicalSize::new(
                            size.width.max(child.preferred.width),
                            size.height.max(child.preferred.height),
                        )
                    });
                    widget.layout(direction, content, constraints).viewport
                }
                Widget::Button(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::IconButton(widget) => widget.layout(theme, direction, constraints).size,
                Widget::Checkbox(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
                Widget::Radio(widget) => {
                    widget
                        .layout(text_system, theme, direction, constraints)
                        .size
                }
            };
            measured.insert(id, size);
        }

        let mut placements = HashMap::with_capacity(tree.len());
        let root_size = measured[&tree.root()];
        placements.insert(
            tree.root(),
            WidgetPlacement::new(root.origin, LogicalConstraints::tight(root_size), direction),
        );
        for id in ids {
            let node = tree.get(id).unwrap();
            if node.children().is_empty() {
                continue;
            }
            let parent = placements[&id];
            let children = node
                .children()
                .iter()
                .map(|child| LayoutChild::new(measured[child]))
                .collect::<Vec<_>>();
            let child_bounds = match &node.state {
                Widget::Row(widget) => {
                    widget
                        .layout(direction, &children, parent.constraints)
                        .children
                }
                Widget::Column(widget) => {
                    widget
                        .layout(direction, &children, parent.constraints)
                        .children
                }
                Widget::Stack(widget) => {
                    let stack_children = children
                        .iter()
                        .map(|child| StackChild::new(child.preferred))
                        .collect::<Vec<_>>();
                    widget
                        .layout(direction, &stack_children, parent.constraints)
                        .children
                }
                Widget::ScrollView(widget) => {
                    let content = children.iter().fold(LogicalSize::default(), |size, child| {
                        LogicalSize::new(
                            size.width.max(child.preferred.width),
                            size.height.max(child.preferred.height),
                        )
                    });
                    let layout = widget.layout(direction, content, parent.constraints);
                    children
                        .iter()
                        .map(|child| {
                            LogicalRect::new(layout.content_bounds.origin, child.preferred)
                        })
                        .collect()
                }
                _ => children
                    .iter()
                    .map(|child| LogicalRect::new(LogicalPoint::default(), child.preferred))
                    .collect(),
            };
            for (child, bounds) in node.children().iter().copied().zip(child_bounds) {
                placements.insert(
                    child,
                    WidgetPlacement::new(
                        LogicalPoint::new(
                            parent.origin.x + bounds.origin.x,
                            parent.origin.y + bounds.origin.y,
                        ),
                        LogicalConstraints::tight(bounds.size),
                        direction,
                    ),
                );
            }
        }
        Self::build(tree, text_system, theme, |id, _| placements[&id])
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Direction, Icon, Image, LogicalConstraints, LogicalPoint, LogicalSize, PixelFormat,
        PixelImage, SemanticColorToken, SemanticRole, Text, ThemeController, ThemeDefinition,
        UserPreferences, WidgetTree,
    };

    use super::{Widget, WidgetFrame, WidgetPlacement};

    fn visual_digest(frame: &WidgetFrame, text_system: &mut crate::TextSystem) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        let mut feed = |bytes: &[u8]| {
            for byte in bytes {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        };
        for draw in &frame.text {
            feed(draw.text.as_bytes());
            for value in draw.baseline.into_iter().chain(draw.color) {
                feed(&value.to_bits().to_le_bytes());
            }
            let line = text_system.shape_line_with_style(&draw.text, &draw.style);
            for glyph in &line.glyphs {
                feed(&glyph.start.to_le_bytes());
                feed(&glyph.end.to_le_bytes());
                feed(&glyph.x.to_bits().to_le_bytes());
                feed(&glyph.width.to_bits().to_le_bytes());
                if let Some(image) = text_system.rasterize_glyph(glyph, 1.0) {
                    feed(&image.left.to_le_bytes());
                    feed(&image.top.to_le_bytes());
                    feed(&image.width.to_le_bytes());
                    feed(&image.height.to_le_bytes());
                    feed(&image.data);
                }
            }
        }
        hash
    }

    fn image_visual_digest(frame: &WidgetFrame) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        let mut feed = |bytes: &[u8]| {
            for byte in bytes {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        };
        for draw in &frame.images {
            feed(draw.image.data());
            feed(&draw.image.width().to_le_bytes());
            feed(&draw.image.height().to_le_bytes());
            for value in [
                draw.bounds.origin.x,
                draw.bounds.origin.y,
                draw.bounds.size.width,
                draw.bounds.size.height,
                draw.clip.origin.x,
                draw.clip.origin.y,
                draw.clip.size.width,
                draw.clip.size.height,
            ] {
                feed(&value.to_bits().to_le_bytes());
            }
            feed(&[u8::from(draw.mirror_horizontal)]);
            for value in draw.tint.unwrap_or_default() {
                feed(&value.to_bits().to_le_bytes());
            }
        }
        hash
    }

    fn button_visual_digest(frame: &WidgetFrame) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        let mut feed = |bytes: &[u8]| {
            for byte in bytes {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        };
        for draw in &frame.rectangles {
            for value in draw
                .position
                .into_iter()
                .chain(draw.size)
                .chain(draw.radii)
                .chain(draw.color)
                .chain([draw.border_width])
                .chain(draw.border_color)
            {
                feed(&value.to_bits().to_le_bytes());
            }
        }
        for draw in &frame.text {
            feed(draw.text.as_bytes());
            for value in draw.baseline.into_iter().chain(draw.color) {
                feed(&value.to_bits().to_le_bytes());
            }
        }
        feed(&image_visual_digest(frame).to_le_bytes());
        hash
    }

    fn text_frame(
        content: &str,
        direction: Direction,
        width: f32,
    ) -> (WidgetFrame, crate::TextSystem) {
        let tree = WidgetTree::new(Widget::from(Text::new(content)));
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(11.5, 17.25),
                LogicalConstraints::loose(LogicalSize::new(width, 160.0)),
                direction,
            )
        });
        (frame, text_system)
    }

    #[test]
    fn retained_text_frame_freezes_matching_geometry_semantics_and_paint() {
        let mut tree = WidgetTree::new(Widget::from(Text::new("رابط Mio-GUI")));
        let id = tree.root();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(20.0, 30.0),
                LogicalConstraints::loose(LogicalSize::new(180.0, 100.0)),
                Direction::Rtl,
            )
        });

        let geometry = frame.geometry.get(id).unwrap();
        let semantics = frame.semantics.get(id).unwrap();
        assert_eq!(geometry.bounds.origin, LogicalPoint::new(20.0, 30.0));
        assert_eq!(semantics.semantics.role, SemanticRole::Text);
        assert_eq!(semantics.semantics.name.as_deref(), Some("رابط Mio-GUI"));
        assert_eq!(frame.text.len(), 1);
        assert_eq!(frame.text[0].text, "رابط Mio-GUI");
        assert_eq!(frame.text[0].color, theme.colors.text.to_array());
        assert!(frame.rectangles.is_empty());

        tree.get_mut(id).unwrap().state = Widget::from(Text::new("changed after frame"));
        assert_eq!(frame.text[0].text, "رابط Mio-GUI");
    }

    #[test]
    fn retained_tree_paint_order_is_preserved_in_text_draw_order() {
        let mut tree = WidgetTree::new(Widget::from(Text::new("first")));
        let root = tree.root();
        let second = tree
            .append(root, Widget::from(Text::new("second")))
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
            WidgetPlacement::new(
                LogicalPoint::new(0.0, if id == root { 0.0 } else { 30.0 }),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            )
        });

        assert_eq!(frame.geometry.paint_order(), &[root, second]);
        assert_eq!(frame.text[0].text, "first");
        assert_eq!(frame.text[1].text, "second");
    }

    #[test]
    fn retained_image_and_icon_frame_freezes_geometry_semantics_and_paint_order() {
        let image_source = PixelImage::new(2, 1, PixelFormat::Rgba8, vec![12_u8; 8]).unwrap();
        let icon_source = PixelImage::new(1, 2, PixelFormat::Alpha8, vec![255_u8; 2]).unwrap();
        let mut tree = WidgetTree::new(Widget::from(
            Image::new(image_source).with_alternative_text("Mio logo"),
        ));
        let root = tree.root();
        let text = tree
            .append(root, Widget::from(Text::new("between")))
            .unwrap();
        let mut icon = Icon::new(icon_source)
            .unwrap()
            .with_alternative_text("Open");
        icon.color = SemanticColorToken::Primary;
        let icon_id = tree.append(root, Widget::from(icon)).unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
            WidgetPlacement::new(
                LogicalPoint::new(if id == root { 5.0 } else { 30.0 }, 7.0),
                LogicalConstraints::tight(LogicalSize::new(20.0, 10.0)),
                Direction::Rtl,
            )
        });

        assert_eq!(frame.geometry.paint_order(), &[root, text, icon_id]);
        assert_eq!(frame.images.len(), 2);
        assert_eq!(frame.images[0].image.format(), PixelFormat::Rgba8);
        assert_eq!(frame.images[0].tint, None);
        assert_eq!(frame.images[1].image.format(), PixelFormat::Alpha8);
        assert_eq!(frame.images[1].tint, Some(theme.colors.primary.to_array()));
        assert_eq!(frame.images[0].clip.origin, LogicalPoint::new(5.0, 7.0));
        assert_eq!(frame.images[1].clip.origin, LogicalPoint::new(30.0, 7.0));
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.name.as_deref(),
            Some("Mio logo")
        );
        assert_eq!(
            frame
                .semantics
                .get(icon_id)
                .unwrap()
                .semantics
                .name
                .as_deref(),
            Some("Open")
        );

        tree.get_mut(root).unwrap().state = Widget::from(Text::new("changed after frame"));
        assert_eq!(frame.images[0].image.data(), &[12_u8; 8]);
    }

    #[test]
    fn retained_button_and_icon_button_freeze_semantics_geometry_and_paint() {
        use crate::{AdornmentPlacement, Button, IconButton, SemanticAction, VisualVariant};

        let mask =
            || Icon::new(PixelImage::new(1, 1, PixelFormat::Alpha8, vec![255]).unwrap()).unwrap();
        let mut button = Button::new("Continue").with_icon(mask(), AdornmentPlacement::InlineEnd);
        button.style.variant = VisualVariant::Solid;
        let mut tree = WidgetTree::new(Widget::from(button));
        let root = tree.root();
        let icon_button = tree
            .append(root, Widget::from(IconButton::new(mask(), "Menu")))
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
            WidgetPlacement::new(
                LogicalPoint::new(if id == root { 8.0 } else { 180.0 }, 12.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });

        let semantics = &frame.semantics.get(root).unwrap().semantics;
        assert_eq!(semantics.role, SemanticRole::Button);
        assert!(semantics.supports(SemanticAction::Activate));
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.text.len(), 1);
        assert_eq!(frame.text[0].text, "Continue");
        assert_eq!(frame.images.len(), 2);
        assert_eq!(
            frame.geometry.get(icon_button).unwrap().bounds.size.width,
            frame.geometry.get(icon_button).unwrap().bounds.size.height
        );

        tree.get_mut(root).unwrap().state = Widget::from(Button::new("Changed"));
        assert_eq!(frame.text[0].text, "Continue");
    }

    #[test]
    fn retained_checkbox_freezes_checked_semantics_geometry_and_paint() {
        use crate::{Checkbox, SemanticAction};

        let mut checkbox = Checkbox::new("Remember me");
        checkbox.checked = true;
        let mut tree = WidgetTree::new(Widget::from(checkbox));
        let root = tree.root();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(6.0, 9.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });

        let semantics = &frame.semantics.get(root).unwrap().semantics;
        assert_eq!(semantics.role, SemanticRole::Checkbox);
        assert_eq!(semantics.state.checked, Some(true));
        assert!(semantics.supports(SemanticAction::Activate));
        assert!(frame.geometry.get(root).unwrap().bounds.size.width > 16.0);
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.text.len(), 1);
        assert_eq!(frame.text[0].text, "Remember me");

        tree.get_mut(root).unwrap().state = Widget::from(Checkbox::new("Changed"));
        assert_eq!(frame.text[0].text, "Remember me");
        assert_eq!(frame.rectangles[0].color, theme.colors.primary.to_array());
    }

    #[test]
    fn checkbox_keyboard_activation_respects_disabled_focus_policy() {
        use crate::{
            Checkbox, FocusSnapshot, Key, KeyboardEvent, SemanticAction, semantic_action_for_key,
        };

        let checkbox = Checkbox::new("Remember me");
        let tree = WidgetTree::new(Widget::from(checkbox));
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[tree.root()]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::default(),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            )
        });
        let space = KeyboardEvent::pressed(Key::Space);
        assert_eq!(
            semantic_action_for_key(&frame.semantics, tree.root(), &space, Direction::Ltr)
                .unwrap()
                .action,
            SemanticAction::Activate
        );

        let mut disabled = Checkbox::new("Disabled");
        disabled.disabled = true;
        let tree = WidgetTree::new(Widget::from(disabled));
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert!(focus.tab_order().is_empty());
    }

    #[test]
    fn retained_checkbox_visual_outputs_match_direction_goldens() {
        use crate::Checkbox;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, false, 14149347552930099619_u64),
            (Direction::Rtl, true, 7749473330257813281_u64),
        ];
        for (direction, checked, expected) in fixtures {
            let mut checkbox = Checkbox::new("Remember me");
            checkbox.checked = checked;
            let tree = WidgetTree::new(Widget::from(checkbox));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_radio_supports_focus_keyboard_semantics_and_frozen_paint() {
        use crate::{
            FocusSnapshot, Key, KeyboardEvent, Radio, SemanticAction, semantic_action_for_key,
        };

        let mut radio = Radio::new("Standard delivery");
        radio.selected = true;
        let mut tree = WidgetTree::new(Widget::from(radio));
        let root = tree.root();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(4.0, 7.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            )
        });
        let space = KeyboardEvent::pressed(Key::Space);
        assert_eq!(
            semantic_action_for_key(&frame.semantics, root, &space, Direction::Rtl)
                .unwrap()
                .action,
            SemanticAction::Activate
        );
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.role,
            SemanticRole::Radio
        );
        assert_eq!(
            frame.semantics.get(root).unwrap().semantics.state.checked,
            Some(true)
        );
        assert_eq!(frame.rectangles.len(), 2);
        assert_eq!(frame.text[0].text, "Standard delivery");

        tree.get_mut(root).unwrap().state = Widget::from(Radio::new("Changed"));
        assert_eq!(frame.text[0].text, "Standard delivery");
    }

    #[test]
    fn retained_radio_visual_outputs_match_direction_goldens() {
        use crate::Radio;

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (Direction::Ltr, false, 17449127050552886139_u64),
            (Direction::Rtl, true, 537322983665427384_u64),
        ];
        for (direction, selected, expected) in fixtures {
            let mut radio = Radio::new("Standard delivery");
            radio.selected = selected;
            let tree = WidgetTree::new(Widget::from(radio));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn button_focus_and_keyboard_activation_follow_enabled_semantics() {
        use crate::{
            Button, FocusSnapshot, IconButton, Key, KeyboardEvent, SemanticAction,
            semantic_action_for_key,
        };

        let mask =
            || Icon::new(PixelImage::new(1, 1, PixelFormat::Alpha8, vec![255]).unwrap()).unwrap();
        let mut tree = WidgetTree::new(Widget::from(Button::new("Save")));
        let root = tree.root();
        let mut disabled = IconButton::new(mask(), "Menu");
        disabled.style.state.disabled = true;
        let disabled = tree.append(root, Widget::from(disabled)).unwrap();
        let focus = FocusSnapshot::build(&tree, |_, widget| widget.focus_policy());
        assert_eq!(focus.tab_order(), &[root]);

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::default(),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            )
        });
        let enter = KeyboardEvent::pressed(Key::Enter);
        assert_eq!(
            semantic_action_for_key(&frame.semantics, root, &enter, Direction::Ltr)
                .unwrap()
                .action,
            SemanticAction::Activate
        );
        assert_eq!(
            semantic_action_for_key(&frame.semantics, disabled, &enter, Direction::Ltr),
            None
        );
    }

    #[test]
    fn retained_button_visual_outputs_match_direction_goldens() {
        use crate::{AdornmentPlacement, Button, VisualVariant};

        let mask = || {
            Icon::new(PixelImage::new(2, 1, PixelFormat::Alpha8, vec![255, 96]).unwrap()).unwrap()
        };
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let fixtures = [
            (
                Direction::Ltr,
                VisualVariant::Solid,
                11723903865523465027_u64,
            ),
            (
                Direction::Rtl,
                VisualVariant::Outline,
                14904414208503005046_u64,
            ),
        ];
        for (direction, variant, expected) in fixtures {
            let mut button =
                Button::new("Continue").with_icon(mask(), AdornmentPlacement::InlineEnd);
            button.style.variant = variant;
            let tree = WidgetTree::new(Widget::from(button));
            let mut text_system = crate::TextSystem::new();
            let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
                WidgetPlacement::new(
                    LogicalPoint::new(7.0, 11.0),
                    LogicalConstraints::unconstrained(),
                    direction,
                )
            });
            let actual = button_visual_digest(&frame);
            assert_eq!(actual, expected, "direction={direction:?} actual={actual}");
        }
    }

    #[test]
    fn retained_spacer_and_divider_freeze_geometry_and_theme_resolved_paint() {
        use crate::{Divider, Spacer};

        let mut tree = WidgetTree::new(Widget::from(Spacer::new(LogicalSize::new(12.0, 8.0))));
        let root = tree.root();
        let _divider = tree
            .append(root, Widget::from(Divider::vertical()))
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
            WidgetPlacement::new(
                LogicalPoint::new(if id == root { 4.0 } else { 20.0 }, 6.0),
                LogicalConstraints::tight(LogicalSize::new(12.0, 8.0)),
                Direction::Ltr,
            )
        });

        assert_eq!(
            frame.geometry.get(root).unwrap().bounds.size,
            LogicalSize::new(12.0, 8.0)
        );
        assert_eq!(frame.semantics.get(root).unwrap().semantics.name, None);
        assert_eq!(frame.rectangles.len(), 1);
        assert_eq!(frame.rectangles[0].position, [20.0, 6.0]);
        assert_eq!(frame.rectangles[0].color, theme.colors.border.to_array());
    }

    #[test]
    fn retained_surface_freezes_theme_resolved_rectangle() {
        use crate::Surface;
        let tree = WidgetTree::new(Widget::from(Surface::new(LogicalSize::new(20.0, 10.0))));
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(3.0, 4.0),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            )
        });
        assert_eq!(frame.rectangles[0].position, [3.0, 4.0]);
        assert_eq!(frame.rectangles[0].color, theme.colors.surface.to_array());
    }

    #[test]
    fn retained_container_freezes_constraint_resolved_geometry_without_paint() {
        use crate::{Container, LogicalRect};
        let tree = WidgetTree::new(Widget::from(Container::new(LogicalSize::new(32.0, 18.0))));
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
            WidgetPlacement::new(
                LogicalPoint::new(5.0, 7.0),
                LogicalConstraints::tight(LogicalSize::new(20.0, 10.0)),
                Direction::Ltr,
            )
        });
        assert_eq!(
            frame.geometry.get(tree.root()).unwrap().bounds,
            LogicalRect::new(LogicalPoint::new(5.0, 7.0), LogicalSize::new(20.0, 10.0))
        );
        assert!(frame.rectangles.is_empty());
    }

    #[test]
    fn composed_row_places_retained_children_and_mirrors_geometry_in_rtl() {
        use crate::{Row, Spacer};

        let mut tree = WidgetTree::new(Widget::from(Row::default()));
        let root = tree.root();
        let first = tree
            .append(root, Widget::from(Spacer::new(LogicalSize::new(10.0, 4.0))))
            .unwrap();
        let second = tree
            .append(root, Widget::from(Spacer::new(LogicalSize::new(6.0, 8.0))))
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build_composed(
            &tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::new(5.0, 7.0),
                LogicalConstraints::unconstrained(),
                Direction::Rtl,
            ),
        );

        assert_eq!(frame.geometry.paint_order(), &[root, first, second]);
        assert_eq!(
            frame.geometry.get(root).unwrap().bounds.size,
            LogicalSize::new(16.0, 8.0)
        );
        assert_eq!(
            frame.geometry.get(first).unwrap().bounds.origin,
            LogicalPoint::new(11.0, 7.0)
        );
        assert_eq!(
            frame.geometry.get(second).unwrap().bounds.origin,
            LogicalPoint::new(5.0, 7.0)
        );
    }

    #[test]
    fn composed_stack_overlays_children_in_stable_paint_order() {
        use crate::{Spacer, Stack};

        let mut tree = WidgetTree::new(Widget::from(Stack));
        let root = tree.root();
        let back = tree
            .append(root, Widget::from(Spacer::new(LogicalSize::new(10.0, 8.0))))
            .unwrap();
        let front = tree
            .append(root, Widget::from(Spacer::new(LogicalSize::new(4.0, 3.0))))
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build_composed(
            &tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::new(2.0, 3.0),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            ),
        );

        assert_eq!(frame.geometry.paint_order(), &[root, back, front]);
        assert_eq!(
            frame.geometry.get(back).unwrap().bounds.origin,
            LogicalPoint::new(2.0, 3.0)
        );
        assert_eq!(
            frame.geometry.get(front).unwrap().bounds.origin,
            LogicalPoint::new(2.0, 3.0)
        );
    }

    #[test]
    fn composed_scroll_view_offsets_content_and_clips_descendants() {
        use crate::{ClipRegion, LogicalRect, ScrollOffset, ScrollView, Spacer};

        let scroll = ScrollView {
            viewport: Some(LogicalSize::new(20.0, 10.0)),
            offset: ScrollOffset::new(5.0, 0.0),
            ..ScrollView::default()
        };
        let mut tree = WidgetTree::new(Widget::from(scroll));
        let root = tree.root();
        let content = tree
            .append(
                root,
                Widget::from(Spacer::new(LogicalSize::new(40.0, 10.0))),
            )
            .unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build_composed(
            &tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::new(10.0, 12.0),
                LogicalConstraints::unconstrained(),
                Direction::Ltr,
            ),
        );

        let root_bounds =
            LogicalRect::new(LogicalPoint::new(10.0, 12.0), LogicalSize::new(20.0, 10.0));
        assert_eq!(frame.geometry.get(root).unwrap().bounds, root_bounds);
        assert_eq!(
            frame.geometry.get(content).unwrap().bounds.origin,
            LogicalPoint::new(5.0, 12.0)
        );
        assert_eq!(
            frame.geometry.get(content).unwrap().clip,
            ClipRegion::Rect(root_bounds)
        );
    }

    #[test]
    fn retained_image_and_icon_visual_outputs_match_goldens() {
        let image = PixelImage::new(
            3,
            2,
            PixelFormat::Rgba8,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
                255, 0, 255, 255,
            ],
        )
        .unwrap();
        let mut icon =
            Icon::new(PixelImage::new(2, 2, PixelFormat::Alpha8, vec![0, 255, 255, 0]).unwrap())
                .unwrap();
        icon.mirror_in_rtl = true;
        icon.color = SemanticColorToken::Primary;
        let mut tree = WidgetTree::new(Widget::from(Image::new(image)));
        let root = tree.root();
        tree.append(root, Widget::from(icon)).unwrap();
        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = crate::TextSystem::new();
        let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
            WidgetPlacement::new(
                LogicalPoint::new(if id == root { 2.0 } else { 17.0 }, 3.0),
                LogicalConstraints::tight(LogicalSize::new(10.0, 8.0)),
                Direction::Rtl,
            )
        });
        assert_eq!(image_visual_digest(&frame), 9240581001633210751);
    }

    #[test]
    fn retained_text_visual_outputs_match_bundled_font_goldens() {
        let fixtures = [
            (
                "Mio-GUI text",
                Direction::Ltr,
                240.0,
                7466596128845657332_u64,
            ),
            (
                "رابط کاربری",
                Direction::Rtl,
                240.0,
                9441082396134697323_u64,
            ),
            (
                "نسخه Mio-GUI 2",
                Direction::Rtl,
                240.0,
                9807824100324377841_u64,
            ),
            (
                "متن بلند برای شکستن سطرها",
                Direction::Rtl,
                82.0,
                17325006582298125765_u64,
            ),
        ];

        for (content, direction, width, expected) in fixtures {
            let (frame, mut text_system) = text_frame(content, direction, width);
            let actual = visual_digest(&frame, &mut text_system);
            assert_eq!(actual, expected, "content={content:?} actual={actual}");
        }
    }
}
