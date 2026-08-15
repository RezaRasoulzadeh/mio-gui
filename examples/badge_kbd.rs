// badge_kbd.rs

use mio_gui::{
    Badge, Direction, Kbd, LogicalConstraints, LogicalPoint, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = WidgetTree::new(Widget::from(Badge::new("New")));
    tree.append(tree.root(), Widget::from(Kbd::new("Ctrl K")))?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, if id == tree.root() { 16.0 } else { 56.0 }),
            LogicalConstraints::unconstrained(),
            Direction::Rtl,
        )
    });
    println!(
        "labels={} surfaces={}",
        frame.text.len(),
        frame.rectangles.len()
    );
    Ok(())
}
