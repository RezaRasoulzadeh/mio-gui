// widget.rs

use std::collections::HashMap;

use crate::{
    Direction, FrameNode, FrameSnapshot, ImageDraw, LogicalConstraints, LogicalPoint, LogicalRect,
    RectDraw, ResolvedTheme, SemanticSnapshot, Semantics, TextDraw, TextSystem, WidgetGeometry,
    WidgetId, WidgetTree,
};

use super::{Container, Divider, Icon, IconLayout, Image, ImageLayout, Spacer, Surface, Text, TextLayout};

#[derive(Clone, Debug, PartialEq)]
pub enum Widget {
    Text(Text),
    Image(Image),
    Icon(Icon),
    Spacer(Spacer),
    Divider(Divider),
    Surface(Surface),
    Container(Container),
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
impl From<Container> for Widget { fn from(value: Container) -> Self { Self::Container(value) } }

impl Widget {
    pub fn semantics(&self) -> Semantics {
        match self {
            Self::Text(text) => text.semantics(),
            Self::Image(image) => image.semantics(),
            Self::Icon(icon) => icon.semantics(),
            Self::Spacer(_) | Self::Divider(_) | Self::Surface(_) | Self::Container(_) => Semantics::default(),
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
                Widget::Container(container) => WidgetLayout::Container(container.layout(placement.constraints)),
            };
            let size = match &layout {
                WidgetLayout::Text(layout) => layout.size,
                WidgetLayout::Image(layout) => layout.size,
                WidgetLayout::Icon(layout) => layout.size(),
                WidgetLayout::Spacer(size)
                | WidgetLayout::Divider(size)
                | WidgetLayout::Surface(size)
                | WidgetLayout::Container(size) => *size,
            };
            layouts.insert(id, (placement.origin, layout));
            WidgetGeometry::new(LogicalRect::new(placement.origin, size))
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
