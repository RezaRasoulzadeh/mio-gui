// swap_theme.rs

use mio_gui::{
    Column, Direction, LogicalConstraints, LogicalPoint, Swap, TextSystem, ThemeController,
    ThemeDefinition, ThemeSwitcher, UserPreferences, Widget, WidgetFrame, WidgetPlacement,
    WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = WidgetTree::new(Widget::from(Column::default()));
    let root = tree.root();
    tree.append(root, Widget::from(Swap::new("Playback", "Play", "Pause")))?;
    tree.append(root, Widget::from(ThemeSwitcher::new("Theme")))?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build_composed(
        &tree,
        &mut text_system,
        &theme,
        WidgetPlacement::new(
            LogicalPoint::new(16.0, 16.0),
            LogicalConstraints::unconstrained(),
            Direction::Rtl,
        ),
    );
    println!(
        "controls={} labels={} backgrounds={}",
        frame.semantics.order().len() - 1,
        frame.text.len(),
        frame.rectangles.len()
    );
    Ok(())
}
