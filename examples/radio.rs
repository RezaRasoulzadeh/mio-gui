// radio.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, Radio, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() {
    let mut radio = Radio::new("Standard delivery");
    radio.selected = true;
    let tree = WidgetTree::new(Widget::from(radio));
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
        "selected={} indicators={} labels={}",
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
