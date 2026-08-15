use mio_gui::{
    ChatBubble, Diff, Direction, LogicalConstraints, LogicalPoint, PixelFormat, PixelImage,
    TextSystem, ThemeController, ThemeDefinition, UserPreferences, Widget, WidgetFrame,
    WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let before = PixelImage::new(4, 2, PixelFormat::Rgba8, vec![64; 32])?;
    let after = PixelImage::new(4, 2, PixelFormat::Rgba8, vec![220; 32])?;
    let mut bubble = ChatBubble::new("Mina", "The new version is ready");
    bubble.outgoing = true;
    let mut tree = WidgetTree::new(Widget::from(bubble));
    tree.append(
        tree.root(),
        Widget::from(Diff::new("Before and after", before, after, 0.5)?),
    )?;
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
    println!("labels={} images={}", frame.text.len(), frame.images.len());
    Ok(())
}
