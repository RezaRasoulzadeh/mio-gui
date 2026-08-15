use mio_gui::{
    Direction, Fieldset, LogicalConstraints, LogicalPoint, Rating, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fieldset = Fieldset::new("Review", "How was your experience?");
    fieldset.set_validation_message(Some("A rating is required".into()));
    let mut tree = WidgetTree::new(Widget::from(fieldset));
    tree.append(tree.root(), Widget::from(Rating::new("Quality", 5, 3)?))?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, if id == tree.root() { 16.0 } else { 104.0 }),
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
