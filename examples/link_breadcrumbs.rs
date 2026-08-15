use mio_gui::{
    Breadcrumbs, Direction, Link, LogicalConstraints, LogicalPoint, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let breadcrumbs = Breadcrumbs::new(["Home", "Account", "Settings"])?;
    let mut tree = WidgetTree::new(Widget::from(breadcrumbs));
    tree.append(
        tree.root(),
        Widget::from(Link::new("Open settings", "settings")),
    )?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, if id == tree.root() { 16.0 } else { 64.0 }),
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
