// context_menu.rs

use mio_gui::{
    ContextMenu, Direction, LogicalConstraints, LogicalPoint, LogicalSize, Menu, MenuItem,
    TextSystem, ThemeController, ThemeDefinition, UserPreferences, Widget, WidgetFrame,
    WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = ContextMenu::new(Menu::new(
        "Context actions",
        vec![MenuItem::new("Copy"), MenuItem::new("Delete")],
    )?);
    context.open_at(LogicalPoint::new(210.0, 120.0));
    let tree = WidgetTree::new(Widget::from(context));
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
        WidgetPlacement::new(
            LogicalPoint::default(),
            LogicalConstraints::tight(LogicalSize::new(240.0, 160.0)),
            Direction::Rtl,
        )
    });
    println!(
        "items={} rectangles={} origin={:?}",
        frame.text.len(),
        frame.rectangles.len(),
        frame.rectangles[0].position
    );
    Ok(())
}
