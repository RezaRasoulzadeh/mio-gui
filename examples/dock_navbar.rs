use mio_gui::{
    Direction, Dock, LogicalConstraints, LogicalPoint, Navbar, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dock = Dock::new("Primary", ["Home", "Search", "Profile"])?;
    let navbar = Navbar::new("Site", ["Docs", "Examples", "About"])?;
    let mut tree = WidgetTree::new(Widget::from(navbar));
    tree.append(tree.root(), Widget::from(dock))?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, if id == tree.root() { 16.0 } else { 72.0 }),
            LogicalConstraints::unconstrained(),
            Direction::Rtl,
        )
    });
    println!(
        "labels={} controls={}",
        frame.text.len(),
        frame.rectangles.len()
    );
    Ok(())
}
