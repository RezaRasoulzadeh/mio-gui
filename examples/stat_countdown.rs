use mio_gui::{
    Countdown, Direction, LogicalConstraints, LogicalPoint, Stat, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = WidgetTree::new(Widget::from(Stat::new("Revenue", "$42,000")));
    tree.append(
        tree.root(),
        Widget::from(Countdown::new("Offer ends", 3661)),
    )?;
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
    println!(
        "labels={} surfaces={}",
        frame.text.len(),
        frame.rectangles.len()
    );
    Ok(())
}
