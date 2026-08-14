// widget.rs

use std::collections::HashMap;

use crate::{
    Direction, FrameNode, FrameSnapshot, LogicalConstraints, LogicalPoint, LogicalRect, RectDraw,
    ResolvedTheme, SemanticSnapshot, Semantics, TextDraw, TextSystem, WidgetGeometry, WidgetId,
    WidgetTree,
};

use super::{Text, TextLayout};

#[derive(Clone, Debug, PartialEq)]
pub enum Widget {
    Text(Text),
}

impl From<Text> for Widget {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}

impl Widget {
    pub fn semantics(&self) -> Semantics {
        match self {
            Self::Text(text) => text.semantics(),
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
}

#[derive(Clone, Debug)]
pub struct WidgetFrame {
    pub geometry: FrameSnapshot,
    pub semantics: SemanticSnapshot,
    pub rectangles: Vec<RectDraw>,
    pub text: Vec<TextDraw>,
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
            };
            let size = match &layout {
                WidgetLayout::Text(layout) => layout.size,
            };
            layouts.insert(id, (placement.origin, layout));
            WidgetGeometry::new(LogicalRect::new(placement.origin, size))
        });
        let semantics = SemanticSnapshot::build(tree, |_, widget| widget.semantics());
        let mut rectangles = Vec::new();
        let mut text = Vec::new();
        geometry.paint(|node| {
            let (origin, layout) = layouts.get(&node.id).unwrap();
            match layout {
                WidgetLayout::Text(layout) => {
                    let widget = &tree.get(node.id).unwrap().state;
                    let Widget::Text(widget) = widget;
                    text.extend(layout.draws(
                        widget.content(),
                        *origin,
                        theme.colors.resolve(layout.color).to_array(),
                    ));
                }
            }
        });
        rectangles.shrink_to_fit();

        Self {
            geometry,
            semantics,
            rectangles,
            text,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Direction, LogicalConstraints, LogicalPoint, LogicalSize, SemanticRole, Text,
        ThemeController, ThemeDefinition, UserPreferences, WidgetTree,
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
