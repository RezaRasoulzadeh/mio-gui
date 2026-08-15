// switch.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, Switch, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() {
    let mut switch = Switch::new("Enable notifications");
    switch.checked = true;
    let tree = WidgetTree::new(Widget::from(switch));
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, 16.0),
            LogicalConstraints::unconstrained(),
            Direction::Rtl,
        )
    });
    println!(
        "checked={} control_parts={} labels={}",
        frame
            .semantics
            .get(tree.root())
            .unwrap()
            .semantics
            .state
            .checked
            .unwrap(),
        frame.rectangles.len(),
        frame.text.len()
    );
}
