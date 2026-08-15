// search_input.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, SearchInput, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() {
    let search = SearchInput::with_text("Site search", "جستجو");
    let tree = WidgetTree::new(Widget::from(search));
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
        "value={} backgrounds={} icons={}",
        frame
            .semantics
            .get(tree.root())
            .unwrap()
            .semantics
            .value
            .as_deref()
            .unwrap(),
        frame.rectangles.len(),
        frame.images.len()
    );
}
