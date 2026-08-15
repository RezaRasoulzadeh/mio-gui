// slider.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, Slider, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut slider = Slider::new("Volume", 0.0..=100.0, 35.0)?;
    slider.set_step(5.0)?;
    let tree = WidgetTree::new(Widget::from(slider));
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
        "value={} track_parts={}",
        frame
            .semantics
            .get(tree.root())
            .unwrap()
            .semantics
            .value
            .as_deref()
            .unwrap(),
        frame.rectangles.len()
    );
    Ok(())
}
