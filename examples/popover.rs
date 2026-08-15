// popover.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, LogicalSize, Popover, TextSystem, ThemeController,
    ThemeDefinition, TooltipPlacement, UserPreferences, Widget, WidgetFrame, WidgetPlacement,
    WidgetTree,
};

fn main() {
    let mut popover = Popover::new("Formatting", LogicalSize::new(120.0, 80.0));
    popover.open = true;
    popover.placement = TooltipPlacement::InlineStart;
    let tree = WidgetTree::new(Widget::from(popover));
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
        WidgetPlacement::new(
            LogicalPoint::new(140.0, 70.0),
            LogicalConstraints::tight(LogicalSize::new(260.0, 160.0)),
            Direction::Rtl,
        )
    });
    println!(
        "role={:?} overlay={:?} panels={}",
        frame.semantics.get(tree.root()).unwrap().semantics.role,
        frame.geometry.get(tree.root()).unwrap().bounds.size,
        frame.rectangles.len()
    );
}
