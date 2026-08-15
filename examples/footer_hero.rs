use mio_gui::{
    Badge, Direction, Footer, Hero, Indicator, List, LogicalConstraints, LogicalPoint, LogicalSize,
    Spacer, Text, TextSystem, ThemeController, ThemeDefinition, UserPreferences, Widget,
    WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = WidgetTree::new(Widget::from(Footer::default()));
    tree.append(tree.root(), Widget::from(Text::new("Mio-GUI footer")))?;
    let hero = tree.append(tree.root(), Widget::from(Hero))?;
    tree.append(
        hero,
        Widget::from(Spacer::new(LogicalSize::new(120.0, 48.0))),
    )?;
    let list = tree.append(tree.root(), Widget::from(List::default()))?;
    tree.append(list, Widget::from(Text::new("First item")))?;
    tree.append(list, Widget::from(Text::new("Second item")))?;
    let indicator = tree.append(tree.root(), Widget::from(Indicator))?;
    tree.append(
        indicator,
        Widget::from(Spacer::new(LogicalSize::new(80.0, 40.0))),
    )?;
    tree.append(indicator, Widget::from(Badge::new("2")))?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build_composed(
        &tree,
        &mut text_system,
        &theme,
        WidgetPlacement::new(
            LogicalPoint::new(16.0, 16.0),
            LogicalConstraints::tight(LogicalSize::new(320.0, 180.0)),
            Direction::Rtl,
        ),
    );
    println!(
        "labels={} controls={}",
        frame.text.len(),
        frame.rectangles.len()
    );
    Ok(())
}
