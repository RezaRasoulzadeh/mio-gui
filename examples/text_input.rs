// text_input.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, TextInput, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() {
    let mut input = TextInput::with_text("Name", "رضا");
    input.set_placeholder("Enter your name");
    input.required = true;
    let tree = WidgetTree::new(Widget::from(input));
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
        "value={} backgrounds={} text_runs={}",
        frame
            .semantics
            .get(tree.root())
            .unwrap()
            .semantics
            .value
            .as_deref()
            .unwrap(),
        frame.rectangles.len(),
        frame.text.len()
    );
}
