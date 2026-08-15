use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, Table, TextSystem, ThemeController,
    ThemeDefinition, Timeline, TimelineItem, UserPreferences, Widget, WidgetFrame, WidgetPlacement,
    WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let table = Table::new(["Name", "Role"], [["Mina", "Admin"], ["Reza", "Editor"]])?;
    let timeline = Timeline::new([
        TimelineItem::new("Created", "09:00"),
        TimelineItem::new("Published", "10:30"),
    ]);
    let mut tree = WidgetTree::new(Widget::from(table));
    tree.append(tree.root(), Widget::from(timeline))?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, if id == tree.root() { 16.0 } else { 112.0 }),
            LogicalConstraints::unconstrained(),
            Direction::Rtl,
        )
    });
    println!(
        "lines={} surfaces={}",
        frame.text.len(),
        frame.rectangles.len()
    );
    Ok(())
}
