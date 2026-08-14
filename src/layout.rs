// layout.rs

use crate::LogicalEdges;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DirectionSetting {
    #[default]
    Inherit,
    Ltr,
    Rtl,
}

impl DirectionSetting {
    pub const fn resolve(self, inherited: Direction) -> Direction {
        match self {
            Self::Inherit => inherited,
            Self::Ltr => Direction::Ltr,
            Self::Rtl => Direction::Rtl,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FlowEdges {
    pub block_start: f32,
    pub inline_end: f32,
    pub block_end: f32,
    pub inline_start: f32,
}

impl FlowEdges {
    pub fn new(block_start: f32, inline_end: f32, block_end: f32, inline_start: f32) -> Self {
        Self {
            block_start: non_negative(block_start),
            inline_end: non_negative(inline_end),
            block_end: non_negative(block_end),
            inline_start: non_negative(inline_start),
        }
    }

    pub fn all(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    pub fn symmetric(block: f32, inline: f32) -> Self {
        Self::new(block, inline, block, inline)
    }

    pub fn inline(self) -> f32 {
        self.inline_start + self.inline_end
    }

    pub fn block(self) -> f32 {
        self.block_start + self.block_end
    }

    pub fn resolve(self, direction: Direction) -> LogicalEdges {
        match direction {
            Direction::Ltr => LogicalEdges::new(
                self.block_start,
                self.inline_end,
                self.block_end,
                self.inline_start,
            ),
            Direction::Rtl => LogicalEdges::new(
                self.block_start,
                self.inline_start,
                self.block_end,
                self.inline_end,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum InlineAlignment {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

impl InlineAlignment {
    pub const fn resolve(self, direction: Direction) -> HorizontalAlignment {
        match (self, direction) {
            (Self::Center, _) => HorizontalAlignment::Center,
            (Self::Start, Direction::Ltr) | (Self::End, Direction::Rtl) => {
                HorizontalAlignment::Left
            }
            (Self::Start, Direction::Rtl) | (Self::End, Direction::Ltr) => {
                HorizontalAlignment::Right
            }
        }
    }

    pub fn offset(self, direction: Direction, available: f32, item: f32) -> f32 {
        let remaining = (non_negative(available) - non_negative(item)).max(0.0);
        match self.resolve(direction) {
            HorizontalAlignment::Left => 0.0,
            HorizontalAlignment::Center => remaining * 0.5,
            HorizontalAlignment::Right => remaining,
        }
    }
}

fn non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, DirectionSetting, FlowEdges, HorizontalAlignment, InlineAlignment};
    use crate::LogicalEdges;

    #[test]
    fn direction_inherits_and_overrides_at_each_level() {
        let root = DirectionSetting::Rtl.resolve(Direction::Ltr);
        let inherited_child = DirectionSetting::Inherit.resolve(root);
        let ltr_child = DirectionSetting::Ltr.resolve(root);
        let inherited_grandchild = DirectionSetting::Inherit.resolve(ltr_child);
        let rtl_grandchild = DirectionSetting::Rtl.resolve(ltr_child);

        assert_eq!(root, Direction::Rtl);
        assert_eq!(inherited_child, Direction::Rtl);
        assert_eq!(ltr_child, Direction::Ltr);
        assert_eq!(inherited_grandchild, Direction::Ltr);
        assert_eq!(rtl_grandchild, Direction::Rtl);
    }

    #[test]
    fn flow_edges_mirror_only_inline_sides() {
        let edges = FlowEdges::new(1.0, 2.0, 3.0, 4.0);

        assert_eq!(
            edges.resolve(Direction::Ltr),
            LogicalEdges::new(1.0, 2.0, 3.0, 4.0)
        );
        assert_eq!(
            edges.resolve(Direction::Rtl),
            LogicalEdges::new(1.0, 4.0, 3.0, 2.0)
        );
        assert_eq!(edges.inline(), 6.0);
        assert_eq!(edges.block(), 4.0);
    }

    #[test]
    fn logical_alignment_resolves_for_both_directions() {
        assert_eq!(
            InlineAlignment::Start.resolve(Direction::Ltr),
            HorizontalAlignment::Left
        );
        assert_eq!(
            InlineAlignment::Start.resolve(Direction::Rtl),
            HorizontalAlignment::Right
        );
        assert_eq!(
            InlineAlignment::End.resolve(Direction::Ltr),
            HorizontalAlignment::Right
        );
        assert_eq!(
            InlineAlignment::End.resolve(Direction::Rtl),
            HorizontalAlignment::Left
        );
        assert_eq!(
            InlineAlignment::Center.resolve(Direction::Rtl),
            HorizontalAlignment::Center
        );
    }

    #[test]
    fn alignment_offsets_are_mirrored_and_bounded() {
        assert_eq!(
            InlineAlignment::Start.offset(Direction::Ltr, 100.0, 30.0),
            0.0
        );
        assert_eq!(
            InlineAlignment::Start.offset(Direction::Rtl, 100.0, 30.0),
            70.0
        );
        assert_eq!(
            InlineAlignment::End.offset(Direction::Ltr, 100.0, 30.0),
            70.0
        );
        assert_eq!(
            InlineAlignment::End.offset(Direction::Rtl, 100.0, 30.0),
            0.0
        );
        assert_eq!(
            InlineAlignment::Center.offset(Direction::Rtl, 100.0, 30.0),
            35.0
        );
        assert_eq!(InlineAlignment::End.offset(Direction::Ltr, 20.0, 30.0), 0.0);
    }

    #[test]
    fn physical_edges_remain_explicit_and_direction_independent() {
        let physical = LogicalEdges::new(1.0, 2.0, 3.0, 4.0);

        assert_eq!(physical, LogicalEdges::new(1.0, 2.0, 3.0, 4.0));
        assert_ne!(
            FlowEdges::new(1.0, 2.0, 3.0, 4.0).resolve(Direction::Rtl),
            physical
        );
    }
}
