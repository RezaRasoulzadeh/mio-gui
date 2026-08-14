// linear_layout.rs

use crate::{
    Direction, FlowEdges, InlineAlignment, LogicalConstraints, LogicalEdges, LogicalPoint,
    LogicalRect, LogicalSize,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Axis {
    #[default]
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CrossAlignment {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutChild {
    pub preferred: LogicalSize,
    pub constraints: LogicalConstraints,
    pub margin: FlowEdges,
}

impl LayoutChild {
    pub fn new(preferred: LogicalSize) -> Self {
        Self {
            preferred,
            constraints: LogicalConstraints::default(),
            margin: FlowEdges::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearLayout {
    pub axis: Axis,
    pub direction: Direction,
    pub gap: f32,
    pub padding: FlowEdges,
    pub main_alignment: InlineAlignment,
    pub cross_alignment: CrossAlignment,
}

impl Default for LinearLayout {
    fn default() -> Self {
        Self {
            axis: Axis::Row,
            direction: Direction::Ltr,
            gap: 0.0,
            padding: FlowEdges::default(),
            main_alignment: InlineAlignment::Start,
            cross_alignment: CrossAlignment::Start,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearLayoutResult {
    pub size: LogicalSize,
    pub children: Vec<LogicalRect>,
}

impl LinearLayout {
    pub fn layout(
        self,
        children: &[LayoutChild],
        constraints: LogicalConstraints,
    ) -> LinearLayoutResult {
        let padding = self.padding.resolve(self.direction);
        let gap = valid(self.gap);
        let measured = children
            .iter()
            .map(|child| child.constraints.constrain(child.preferred))
            .collect::<Vec<_>>();
        let margins = children
            .iter()
            .map(|child| child.margin.resolve(self.direction))
            .collect::<Vec<_>>();
        let content_main = measured
            .iter()
            .zip(&margins)
            .map(|(size, edges)| main_size(self.axis, *size) + main_edges(self.axis, *edges))
            .sum::<f32>()
            + gap * children.len().saturating_sub(1) as f32;
        let content_cross = measured
            .iter()
            .zip(&margins)
            .map(|(size, edges)| cross_size(self.axis, *size) + cross_edges(self.axis, *edges))
            .reduce(f32::max)
            .unwrap_or(0.0);
        let size = constraints.constrain(make_size(
            self.axis,
            content_main + main_edges(self.axis, padding),
            content_cross + cross_edges(self.axis, padding),
        ));
        let inner_main = (main_size(self.axis, size) - main_edges(self.axis, padding)).max(0.0);
        let inner_cross = (cross_size(self.axis, size) - cross_edges(self.axis, padding)).max(0.0);
        let free_main = (inner_main - content_main).max(0.0);
        let offset = match self.main_alignment {
            InlineAlignment::Start => 0.0,
            InlineAlignment::Center => free_main * 0.5,
            InlineAlignment::End => free_main,
        };
        let reverse = self.axis == Axis::Row && self.direction == Direction::Rtl;
        let mut cursor = if reverse {
            main_size(self.axis, size) - main_end(self.axis, padding) - offset
        } else {
            main_start(self.axis, padding) + offset
        };
        let mut frames = Vec::with_capacity(children.len());
        for (size, margin) in measured.into_iter().zip(margins) {
            let (before, after) = if reverse {
                (main_end(self.axis, margin), main_start(self.axis, margin))
            } else {
                (main_start(self.axis, margin), main_end(self.axis, margin))
            };
            let item_main = main_size(self.axis, size);
            let available_cross = (inner_cross - cross_edges(self.axis, margin)).max(0.0);
            let item_cross = if self.cross_alignment == CrossAlignment::Stretch {
                available_cross
            } else {
                cross_size(self.axis, size).min(available_cross)
            };
            let free_cross = (available_cross - item_cross).max(0.0);
            let cross_reverse = self.axis == Axis::Column && self.direction == Direction::Rtl;
            let cross_offset = match (self.cross_alignment, cross_reverse) {
                (CrossAlignment::Stretch, _)
                | (CrossAlignment::Start, false)
                | (CrossAlignment::End, true) => 0.0,
                (CrossAlignment::Center, _) => free_cross * 0.5,
                (CrossAlignment::End, false) | (CrossAlignment::Start, true) => free_cross,
            };
            let cross =
                cross_start(self.axis, padding) + cross_start(self.axis, margin) + cross_offset;
            let main = if reverse {
                cursor -= before + item_main;
                cursor
            } else {
                cursor += before;
                cursor
            };
            frames.push(make_rect(self.axis, main, cross, item_main, item_cross));
            if reverse {
                cursor -= after + gap;
            } else {
                cursor += item_main + after + gap;
            }
        }
        LinearLayoutResult {
            size,
            children: frames,
        }
    }
}

fn main_size(axis: Axis, size: LogicalSize) -> f32 {
    if axis == Axis::Row {
        size.width
    } else {
        size.height
    }
}
fn cross_size(axis: Axis, size: LogicalSize) -> f32 {
    if axis == Axis::Row {
        size.height
    } else {
        size.width
    }
}
fn main_start(axis: Axis, edges: LogicalEdges) -> f32 {
    if axis == Axis::Row {
        edges.left
    } else {
        edges.top
    }
}
fn main_end(axis: Axis, edges: LogicalEdges) -> f32 {
    if axis == Axis::Row {
        edges.right
    } else {
        edges.bottom
    }
}
fn cross_start(axis: Axis, edges: LogicalEdges) -> f32 {
    if axis == Axis::Row {
        edges.top
    } else {
        edges.left
    }
}
fn main_edges(axis: Axis, edges: LogicalEdges) -> f32 {
    main_start(axis, edges) + main_end(axis, edges)
}
fn cross_edges(axis: Axis, edges: LogicalEdges) -> f32 {
    if axis == Axis::Row {
        edges.vertical()
    } else {
        edges.horizontal()
    }
}
fn make_size(axis: Axis, main: f32, cross: f32) -> LogicalSize {
    if axis == Axis::Row {
        LogicalSize::new(main, cross)
    } else {
        LogicalSize::new(cross, main)
    }
}
fn make_rect(axis: Axis, main: f32, cross: f32, main_size: f32, cross_size: f32) -> LogicalRect {
    if axis == Axis::Row {
        LogicalRect::new(
            LogicalPoint::new(main, cross),
            LogicalSize::new(main_size, cross_size),
        )
    } else {
        LogicalRect::new(
            LogicalPoint::new(cross, main),
            LogicalSize::new(cross_size, main_size),
        )
    }
}
fn valid(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Axis, CrossAlignment, LayoutChild, LinearLayout};
    use crate::{
        ClipRegion, Direction, DirectionSetting, FlowEdges, LogicalConstraints, LogicalPoint,
        LogicalRect, LogicalSize, Overflow,
    };

    #[test]
    fn row_rtl_mirrors_positions_without_reordering_results() {
        let children = [
            LayoutChild::new(LogicalSize::new(20.0, 10.0)),
            LayoutChild::new(LogicalSize::new(30.0, 20.0)),
        ];
        let constraints = LogicalConstraints::tight(LogicalSize::new(100.0, 40.0));
        let base = LinearLayout {
            gap: 5.0,
            padding: FlowEdges::symmetric(5.0, 10.0),
            ..LinearLayout::default()
        };
        let ltr = base.layout(&children, constraints);
        let rtl = LinearLayout {
            direction: Direction::Rtl,
            ..base
        }
        .layout(&children, constraints);
        assert_eq!(
            ltr.children,
            [
                LogicalRect::from_xywh(10.0, 5.0, 20.0, 10.0),
                LogicalRect::from_xywh(35.0, 5.0, 30.0, 20.0)
            ]
        );
        assert_eq!(
            rtl.children,
            [
                LogicalRect::from_xywh(70.0, 5.0, 20.0, 10.0),
                LogicalRect::from_xywh(35.0, 5.0, 30.0, 20.0)
            ]
        );
    }

    #[test]
    fn column_uses_gap_padding_margin_and_cross_alignment() {
        let mut first = LayoutChild::new(LogicalSize::new(20.0, 10.0));
        first.margin = FlowEdges::new(2.0, 3.0, 4.0, 5.0);
        let result = LinearLayout {
            axis: Axis::Column,
            direction: Direction::Rtl,
            gap: 6.0,
            padding: FlowEdges::new(10.0, 12.0, 14.0, 16.0),
            cross_alignment: CrossAlignment::End,
            ..LinearLayout::default()
        }
        .layout(
            &[first, LayoutChild::new(LogicalSize::new(30.0, 15.0))],
            LogicalConstraints::tight(LogicalSize::new(100.0, 80.0)),
        );
        assert_eq!(
            result.children,
            [
                LogicalRect::from_xywh(15.0, 12.0, 20.0, 10.0),
                LogicalRect::from_xywh(12.0, 32.0, 30.0, 15.0)
            ]
        );
    }

    #[test]
    fn constraints_and_stretch_are_applied() {
        let child = LayoutChild {
            preferred: LogicalSize::new(200.0, 2.0),
            constraints: LogicalConstraints::new(
                LogicalSize::new(10.0, 15.0),
                LogicalSize::new(40.0, 30.0),
            ),
            margin: FlowEdges::symmetric(3.0, 0.0),
        };
        let result = LinearLayout {
            padding: FlowEdges::symmetric(5.0, 10.0),
            cross_alignment: CrossAlignment::Stretch,
            ..LinearLayout::default()
        }
        .layout(
            &[child],
            LogicalConstraints::tight(LogicalSize::new(80.0, 40.0)),
        );
        assert_eq!(
            result.children[0],
            LogicalRect::from_xywh(10.0, 8.0, 40.0, 24.0)
        );
    }

    #[test]
    fn complete_rtl_fixture_is_horizontal_mirror_of_ltr() {
        let children = [
            LayoutChild {
                preferred: LogicalSize::new(24.5, 15.25),
                constraints: LogicalConstraints::default(),
                margin: FlowEdges::new(1.0, 3.5, 2.0, 4.25),
            },
            LayoutChild {
                preferred: LogicalSize::new(31.75, 22.5),
                constraints: LogicalConstraints::default(),
                margin: FlowEdges::new(2.0, 5.0, 1.0, 2.25),
            },
            LayoutChild::new(LogicalSize::new(18.25, 12.75)),
        ];
        let constraints = LogicalConstraints::tight(LogicalSize::new(160.5, 52.25));
        let layout = LinearLayout {
            gap: 7.25,
            padding: FlowEdges::new(4.0, 9.5, 6.0, 11.25),
            cross_alignment: CrossAlignment::Center,
            ..LinearLayout::default()
        };
        let ltr = layout.layout(&children, constraints);
        let rtl = LinearLayout {
            direction: Direction::Rtl,
            ..layout
        }
        .layout(&children, constraints);

        assert_eq!(ltr.size, rtl.size);
        for (ltr_frame, rtl_frame) in ltr.children.iter().zip(&rtl.children) {
            assert_eq!(ltr_frame.size, rtl_frame.size);
            assert_eq!(ltr_frame.origin.y, rtl_frame.origin.y);
            assert_eq!(
                rtl_frame.origin.x,
                ltr.size.width - ltr_frame.origin.x - ltr_frame.size.width
            );
        }
    }

    #[test]
    fn nested_override_uses_local_direction_without_changing_parent_order() {
        let root_direction = DirectionSetting::Rtl.resolve(Direction::Ltr);
        let nested_direction = DirectionSetting::Ltr.resolve(root_direction);
        let root = LinearLayout {
            direction: root_direction,
            gap: 8.0,
            ..LinearLayout::default()
        }
        .layout(
            &[
                LayoutChild::new(LogicalSize::new(70.0, 30.0)),
                LayoutChild::new(LogicalSize::new(20.0, 30.0)),
            ],
            LogicalConstraints::tight(LogicalSize::new(120.0, 30.0)),
        );
        let nested = LinearLayout {
            direction: nested_direction,
            gap: 5.0,
            ..LinearLayout::default()
        }
        .layout(
            &[
                LayoutChild::new(LogicalSize::new(20.0, 10.0)),
                LayoutChild::new(LogicalSize::new(25.0, 10.0)),
            ],
            LogicalConstraints::tight(root.children[0].size),
        );
        let nested_global = nested
            .children
            .iter()
            .map(|frame| {
                LogicalRect::new(
                    LogicalPoint::new(
                        root.children[0].origin.x + frame.origin.x,
                        root.children[0].origin.y + frame.origin.y,
                    ),
                    frame.size,
                )
            })
            .collect::<Vec<_>>();

        assert!(root.children[0].origin.x > root.children[1].origin.x);
        assert!(nested_global[0].origin.x < nested_global[1].origin.x);
        assert_eq!(nested_global[0].size, LogicalSize::new(20.0, 10.0));
        assert_eq!(nested_global[1].size, LogicalSize::new(25.0, 10.0));
    }

    #[test]
    fn constrained_fractional_overflow_is_clipped_without_relayout() {
        let result = LinearLayout {
            gap: 2.25,
            padding: FlowEdges::all(1.5),
            ..LinearLayout::default()
        }
        .layout(
            &[
                LayoutChild::new(LogicalSize::new(40.75, 12.5)),
                LayoutChild::new(LogicalSize::new(35.5, 12.5)),
            ],
            LogicalConstraints::tight(LogicalSize::new(60.25, 20.5)),
        );
        let bounds = LogicalRect::new(LogicalPoint::new(0.0, 0.0), result.size);
        let clip = ClipRegion::from_overflow(bounds, Overflow::Clip);

        assert_eq!(result.size, LogicalSize::new(60.25, 20.5));
        assert_eq!(result.children[0].origin.x, 1.5);
        assert_eq!(result.children[1].origin.x, 44.5);
        assert!(result.children[1].max_x() > result.size.width);
        assert_eq!(
            clip.intersect(ClipRegion::Rect(result.children[1])),
            ClipRegion::Rect(LogicalRect::from_xywh(44.5, 1.5, 15.75, 12.5))
        );
    }
}
