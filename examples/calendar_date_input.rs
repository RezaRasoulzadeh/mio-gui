use mio_gui::{
    Calendar, CivilDate, DateInput, Direction, LogicalConstraints, LogicalPoint, TextSystem,
    ThemeController, ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement,
    WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let selected = CivilDate::new(2026, 8, 16)?;
    let mut tree = WidgetTree::new(Widget::from(Calendar::new("Calendar", selected)));
    let mut input = DateInput::new("Appointment", selected);
    input.open = true;
    tree.append(tree.root(), Widget::from(input))?;
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
        "labels={} controls={}",
        frame.text.len(),
        frame.rectangles.len()
    );
    Ok(())
}
