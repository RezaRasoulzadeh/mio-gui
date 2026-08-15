use crate::{
    Direction, LayoutChild, LinearLayout, LinearLayoutResult, LogicalConstraints, Semantics,
    StackChild,
};

use super::{Column, Stack, StackLayout};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct List {
    pub layout: LinearLayout,
}

impl Default for List {
    fn default() -> Self {
        let mut layout = Column::default().layout;
        layout.gap = 4.0;
        Self { layout }
    }
}

impl List {
    pub fn layout(
        self,
        direction: Direction,
        children: &[LayoutChild],
        constraints: LogicalConstraints,
    ) -> LinearLayoutResult {
        Column {
            layout: self.layout,
        }
        .layout(direction, children, constraints)
    }

    pub fn semantics(&self) -> Semantics {
        Semantics::new(crate::SemanticRole::List)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Indicator;

impl Indicator {
    pub fn layout(
        self,
        direction: Direction,
        children: &[StackChild],
        constraints: LogicalConstraints,
    ) -> StackLayout {
        let mut children = children.to_vec();
        for child in children.iter_mut().skip(1) {
            child.inline_alignment = crate::InlineAlignment::End;
            child.block_alignment = crate::BlockAlignment::Start;
        }
        Stack.layout(direction, &children, constraints)
    }

    pub fn semantics(&self) -> Semantics {
        Semantics::new(crate::SemanticRole::Group).with_name("Indicator")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Footer {
    pub layout: LinearLayout,
}

impl Default for Footer {
    fn default() -> Self {
        Self {
            layout: Column::default().layout,
        }
    }
}

impl Footer {
    pub fn layout(
        self,
        direction: Direction,
        children: &[LayoutChild],
        constraints: LogicalConstraints,
    ) -> LinearLayoutResult {
        Column {
            layout: self.layout,
        }
        .layout(direction, children, constraints)
    }
    pub fn semantics(&self) -> Semantics {
        Semantics::new(crate::SemanticRole::Group).with_name("Footer")
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hero;

impl Hero {
    pub fn layout(
        self,
        direction: Direction,
        children: &[StackChild],
        constraints: LogicalConstraints,
    ) -> StackLayout {
        let mut children = children.to_vec();
        for child in &mut children {
            child.inline_alignment = crate::InlineAlignment::Center;
            child.block_alignment = crate::BlockAlignment::Center;
        }
        Stack.layout(direction, &children, constraints)
    }
    pub fn semantics(&self) -> Semantics {
        Semantics::new(crate::SemanticRole::Group).with_name("Hero")
    }
}

#[cfg(test)]
mod tests {
    use super::{Footer, Hero, Indicator, List};
    use crate::{
        Direction, LayoutChild, LogicalConstraints, LogicalPoint, LogicalSize, StackChild,
    };

    #[test]
    fn footer_flows_content_and_hero_centers_it() {
        let footer = Footer::default().layout(
            Direction::Rtl,
            &[LayoutChild::new(LogicalSize::new(20.0, 8.0))],
            LogicalConstraints::tight(LogicalSize::new(40.0, 24.0)),
        );
        assert_eq!(footer.children[0].origin, LogicalPoint::new(20.0, 0.0));
        let hero = Hero.layout(
            Direction::Ltr,
            &[StackChild::new(LogicalSize::new(20.0, 8.0))],
            LogicalConstraints::tight(LogicalSize::new(40.0, 24.0)),
        );
        assert_eq!(hero.children[0].origin, LogicalPoint::new(10.0, 8.0));
    }

    #[test]
    fn list_flows_items_and_indicator_mirrors_marker_edge() {
        let list = List::default().layout(
            Direction::Ltr,
            &[
                LayoutChild::new(LogicalSize::new(12.0, 4.0)),
                LayoutChild::new(LogicalSize::new(8.0, 6.0)),
            ],
            LogicalConstraints::unconstrained(),
        );
        assert_eq!(list.children[1].origin.y, 8.0);
        let children = [
            StackChild::new(LogicalSize::new(40.0, 24.0)),
            StackChild::new(LogicalSize::new(8.0, 8.0)),
        ];
        let constraints = LogicalConstraints::tight(LogicalSize::new(40.0, 24.0));
        let ltr = Indicator.layout(Direction::Ltr, &children, constraints);
        let rtl = Indicator.layout(Direction::Rtl, &children, constraints);
        assert_eq!(ltr.children[1].origin.x, 32.0);
        assert_eq!(rtl.children[1].origin.x, 0.0);
    }
}
