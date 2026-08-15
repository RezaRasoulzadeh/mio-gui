use mio_gui::{
    Accordion, Carousel, Direction, LogicalConstraints, LogicalPoint, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut accordion = Accordion::new("Details", "More information");
    accordion.open = true;
    let carousel = Carousel::new("Gallery", ["First", "Second", "Third"])?;
    let mut tree = WidgetTree::new(Widget::from(accordion));
    tree.append(tree.root(), Widget::from(carousel))?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, if id == tree.root() { 16.0 } else { 88.0 }),
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
