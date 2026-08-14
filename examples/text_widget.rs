// text_widget.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, LogicalSize, Text, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() {
    let text = Text::new("رابط کاربری Mio-GUI");
    let tree = WidgetTree::new(Widget::from(text));
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, 16.0),
            LogicalConstraints::loose(LogicalSize::new(240.0, 120.0)),
            Direction::Rtl,
        )
    });

    println!(
        "widgets={} text_draws={} bounds={:?}",
        frame.geometry.paint_order().len(),
        frame.text.len(),
        frame.geometry.get(tree.root()).unwrap().bounds,
    );
}
