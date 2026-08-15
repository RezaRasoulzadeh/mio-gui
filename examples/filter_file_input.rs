use mio_gui::{
    Direction, FileInput, Filter, FilterOption, LogicalConstraints, LogicalPoint, TextSystem,
    ThemeController, ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement,
    WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut filter = Filter::new(
        "Topics",
        [FilterOption::new("Rust"), FilterOption::new("GUI")],
    )?;
    filter.toggle_active();
    let mut tree = WidgetTree::new(Widget::from(filter));
    let mut input = FileInput::new("Attachment");
    input.set_files(["report.pdf"])?;
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
