use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, RadialProgress, TextSystem, ThemeController,
    ThemeDefinition, Toast, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = WidgetTree::new(Widget::from(RadialProgress::new("Download", 0.75)?));
    tree.append(tree.root(), Widget::from(Toast::new("Download complete")))?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, if id == tree.root() { 16.0 } else { 80.0 }),
            LogicalConstraints::unconstrained(),
            Direction::Rtl,
        )
    });
    println!("feedback surfaces={}", frame.rectangles.len());
    Ok(())
}
