use mio_gui::{
    Avatar, Card, Direction, LogicalConstraints, LogicalPoint, LogicalSize, PixelFormat,
    PixelImage, Text, TextSystem, ThemeController, ThemeDefinition, UserPreferences, Widget,
    WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pixels = PixelImage::new(2, 2, PixelFormat::Rgba8, vec![200; 16])?;
    let mut tree = WidgetTree::new(Widget::from(
        Card::new(LogicalSize::new(240.0, 120.0)).with_label("Profile"),
    ));
    tree.append(
        tree.root(),
        Widget::from(Avatar::new(pixels).with_alternative_text("Profile photo")),
    )?;
    tree.append(tree.root(), Widget::from(Text::new("Reza")))?;
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
        "surfaces={} images={}",
        frame.rectangles.len(),
        frame.images.len()
    );
    Ok(())
}
