// tooltip.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, LogicalSize, TextSystem, ThemeController,
    ThemeDefinition, Tooltip, TooltipPlacement, UserPreferences, Widget, WidgetFrame,
    WidgetPlacement, WidgetTree,
};

fn main() {
    let mut tooltip = Tooltip::new("Keyboard shortcut");
    tooltip.visible = true;
    tooltip.placement = TooltipPlacement::InlineStart;
    let tree = WidgetTree::new(Widget::from(tooltip));
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
        WidgetPlacement::new(
            LogicalPoint::new(100.0, 50.0),
            LogicalConstraints::tight(LogicalSize::new(240.0, 120.0)),
            Direction::Rtl,
        )
    });
    println!(
        "origin={:?} backgrounds={} text_runs={}",
        frame.geometry.get(tree.root()).unwrap().bounds.origin,
        frame.rectangles.len(),
        frame.text.len()
    );
}
