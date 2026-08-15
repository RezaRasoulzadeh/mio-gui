use mio_gui::{
    Alert, Direction, LogicalConstraints, LogicalPoint, Progress, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = WidgetTree::new(Widget::from(Alert::new("Changes saved")));
    tree.append(tree.root(), Widget::from(Progress::new("Upload", 0.65)?))?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, if id == tree.root() { 16.0 } else { 72.0 }),
            LogicalConstraints::unconstrained(),
            Direction::Ltr,
        )
    });
    println!(
        "feedback={} surfaces={}",
        tree.len(),
        frame.rectangles.len()
    );
    Ok(())
}
