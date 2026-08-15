// dropdown.rs

use mio_gui::{
    Button, Direction, Dropdown, LogicalConstraints, LogicalPoint, Menu, MenuItem, TextSystem,
    ThemeController, ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement,
    WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let menu = Menu::new(
        "Actions",
        vec![MenuItem::new("Open"), MenuItem::new("Delete")],
    )?;
    let mut dropdown = Dropdown::new(Button::new("Actions"), menu);
    dropdown.open = true;
    let tree = WidgetTree::new(Widget::from(dropdown));
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
        "expanded={} rectangles={} text_runs={}",
        frame
            .semantics
            .get(tree.root())
            .unwrap()
            .semantics
            .state
            .expanded
            .unwrap(),
        frame.rectangles.len(),
        frame.text.len()
    );
    Ok(())
}
