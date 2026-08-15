// linear.rs

use crate::{Axis, Direction, LayoutChild, LinearLayout, LinearLayoutResult, LogicalConstraints};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Row {
    pub layout: LinearLayout,
}

impl Row {
    pub fn layout(
        self,
        direction: Direction,
        children: &[LayoutChild],
        constraints: LogicalConstraints,
    ) -> LinearLayoutResult {
        let mut layout = self.layout;
        layout.axis = Axis::Row;
        layout.direction = direction;
        layout.layout(children, constraints)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Column {
    pub layout: LinearLayout,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            layout: LinearLayout {
                axis: Axis::Column,
                ..LinearLayout::default()
            },
        }
    }
}

impl Column {
    pub fn layout(
        self,
        direction: Direction,
        children: &[LayoutChild],
        constraints: LogicalConstraints,
    ) -> LinearLayoutResult {
        let mut layout = self.layout;
        layout.axis = Axis::Column;
        layout.direction = direction;
        layout.layout(children, constraints)
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, Row};
    use crate::{Direction, LayoutChild, LogicalConstraints, LogicalPoint, LogicalSize};

    fn children() -> [LayoutChild; 2] {
        [
            LayoutChild::new(LogicalSize::new(10.0, 4.0)),
            LayoutChild::new(LogicalSize::new(6.0, 8.0)),
        ]
    }

    #[test]
    fn row_preserves_semantic_order_while_mirroring_rtl_positions() {
        let ltr = Row::default().layout(
            Direction::Ltr,
            &children(),
            LogicalConstraints::unconstrained(),
        );
        let rtl = Row::default().layout(
            Direction::Rtl,
            &children(),
            LogicalConstraints::unconstrained(),
        );
        assert_eq!(ltr.children[0].origin, LogicalPoint::new(0.0, 0.0));
        assert_eq!(rtl.children[0].origin, LogicalPoint::new(6.0, 0.0));
        assert_eq!(rtl.children[1].origin, LogicalPoint::new(0.0, 0.0));
    }

    #[test]
    fn column_keeps_block_order_and_mirrors_cross_axis_in_rtl() {
        let result = Column::default().layout(
            Direction::Rtl,
            &children(),
            LogicalConstraints::unconstrained(),
        );
        assert_eq!(result.children[0].origin, LogicalPoint::new(0.0, 0.0));
        assert_eq!(result.children[1].origin, LogicalPoint::new(4.0, 4.0));
    }
}
