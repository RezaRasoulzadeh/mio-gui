// text_area.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, LogicalSize, TextArea, TextSystem,
    ThemeController, ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement,
    WidgetTree,
};

fn main() {
    let mut area = TextArea::with_text("Notes", "سطر اول\nسطر دوم");
    area.set_minimum_lines(4);
    let tree = WidgetTree::new(Widget::from(area));
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, 16.0),
            LogicalConstraints::loose(LogicalSize::new(240.0, 400.0)),
            Direction::Rtl,
        )
    });
    println!(
        "lines={} backgrounds={} value={}",
        frame.text.len(),
        frame.rectangles.len(),
        frame
            .semantics
            .get(tree.root())
            .unwrap()
            .semantics
            .value
            .as_deref()
            .unwrap()
    );
}
