// stack.rs

use crate::{
    BlockAlignment, Direction, HorizontalAlignment, InlineAlignment, LogicalConstraints,
    LogicalPoint, LogicalRect, LogicalSize,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StackChild {
    pub preferred: LogicalSize,
    pub constraints: LogicalConstraints,
    pub inline_alignment: InlineAlignment,
    pub block_alignment: BlockAlignment,
}

impl StackChild {
    pub fn new(preferred: LogicalSize) -> Self {
        Self {
            preferred,
            constraints: LogicalConstraints::default(),
            inline_alignment: InlineAlignment::Start,
            block_alignment: BlockAlignment::Start,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StackLayout {
    pub size: LogicalSize,
    pub children: Vec<LogicalRect>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stack;

impl Stack {
    pub fn layout(
        self,
        direction: Direction,
        children: &[StackChild],
        constraints: LogicalConstraints,
    ) -> StackLayout {
        let measured = children
            .iter()
            .map(|child| child.constraints.constrain(child.preferred))
            .collect::<Vec<_>>();
        let preferred = measured.iter().fold(LogicalSize::default(), |size, child| {
            LogicalSize::new(size.width.max(child.width), size.height.max(child.height))
        });
        let size = constraints.constrain(preferred);
        let children = children
            .iter()
            .zip(measured)
            .map(|(child, measured)| {
                let measured = LogicalSize::new(
                    measured.width.min(size.width),
                    measured.height.min(size.height),
                );
                let x = match child.inline_alignment.resolve(direction) {
                    HorizontalAlignment::Left => 0.0,
                    HorizontalAlignment::Center => (size.width - measured.width) * 0.5,
                    HorizontalAlignment::Right => size.width - measured.width,
                };
                let y = match child.block_alignment {
                    BlockAlignment::Start => 0.0,
                    BlockAlignment::Center => (size.height - measured.height) * 0.5,
                    BlockAlignment::End => size.height - measured.height,
                };
                LogicalRect::new(LogicalPoint::new(x, y), measured)
            })
            .collect();
        StackLayout { size, children }
    }
}

#[cfg(test)]
mod tests {
    use super::{Stack, StackChild};
    use crate::{Direction, InlineAlignment, LogicalConstraints, LogicalPoint, LogicalSize};

    #[test]
    fn stack_overlays_children_and_mirrors_logical_inline_alignment() {
        let mut child = StackChild::new(LogicalSize::new(4.0, 3.0));
        child.inline_alignment = InlineAlignment::Start;
        let constraints = LogicalConstraints::tight(LogicalSize::new(10.0, 8.0));
        let ltr = Stack.layout(Direction::Ltr, &[child], constraints);
        let rtl = Stack.layout(Direction::Rtl, &[child], constraints);
        assert_eq!(ltr.children[0].origin, LogicalPoint::new(0.0, 0.0));
        assert_eq!(rtl.children[0].origin, LogicalPoint::new(6.0, 0.0));
    }

    #[test]
    fn stack_size_is_largest_child_then_outer_constraints() {
        let children = [
            StackChild::new(LogicalSize::new(4.0, 9.0)),
            StackChild::new(LogicalSize::new(12.0, 3.0)),
        ];
        let result = Stack.layout(
            Direction::Ltr,
            &children,
            LogicalConstraints::loose(LogicalSize::new(10.0, 8.0)),
        );
        assert_eq!(result.size, LogicalSize::new(10.0, 8.0));
    }
}
