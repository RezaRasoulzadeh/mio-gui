use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, Pagination, Steps, Tabs, TextSystem,
    ThemeController, ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement,
    WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = WidgetTree::new(Widget::from(Pagination::new("Pages", 4)?));
    tree.append(
        tree.root(),
        Widget::from(Steps::new("Checkout", ["Cart", "Address", "Pay"])?),
    )?;
    tree.append(
        tree.root(),
        Widget::from(Tabs::new("Sections", ["Overview", "Activity"])?),
    )?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        let y = 16.0 + id.get() as f32 * 48.0;
        WidgetPlacement::new(
            LogicalPoint::new(16.0, y),
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
