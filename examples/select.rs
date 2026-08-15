// select.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, Select, SelectOption, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut select = Select::new(
        "Size",
        vec![
            SelectOption::new("Small", "sm"),
            SelectOption::new("Large", "lg"),
        ],
    )?;
    select.select(1)?;
    let tree = WidgetTree::new(Widget::from(select));
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, 16.0),
            LogicalConstraints::unconstrained(),
            Direction::Rtl,
        )
    });
    println!(
        "value={} backgrounds={} icons={}",
        frame
            .semantics
            .get(tree.root())
            .unwrap()
            .semantics
            .value
            .as_deref()
            .unwrap(),
        frame.rectangles.len(),
        frame.images.len()
    );
    Ok(())
}
