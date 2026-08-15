// menu.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, Menu, MenuItem, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let menu = Menu::new(
        "Actions",
        vec![
            MenuItem::new("Open"),
            MenuItem::new("Rename"),
            MenuItem::new("Delete"),
        ],
    )?;
    let tree = WidgetTree::new(Widget::from(menu));
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
        "items={} rectangles={} role={:?}",
        frame.text.len(),
        frame.rectangles.len(),
        frame.semantics.get(tree.root()).unwrap().semantics.role
    );
    Ok(())
}
