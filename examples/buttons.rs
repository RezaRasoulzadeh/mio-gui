// buttons.rs

use mio_gui::{
    AdornmentPlacement, Button, Direction, Icon, IconButton, LogicalConstraints, LogicalPoint,
    PixelFormat, PixelImage, Row, TextSystem, ThemeController, ThemeDefinition, UserPreferences,
    VisualVariant, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let icon = || -> Result<Icon, Box<dyn std::error::Error>> {
        Ok(Icon::new(PixelImage::new(
            2,
            2,
            PixelFormat::Alpha8,
            vec![0, 255, 255, 0],
        )?)?)
    };
    let mut primary = Button::new("Continue").with_icon(icon()?, AdornmentPlacement::InlineEnd);
    primary.style.variant = VisualVariant::Solid;
    let mut tree = WidgetTree::new(Widget::from(Row::default()));
    let root = tree.root();
    tree.append(root, Widget::from(primary))?;
    tree.append(root, Widget::from(IconButton::new(icon()?, "Open menu")))?;

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
        "widgets={} backgrounds={} labels={} icons={}",
        frame.geometry.paint_order().len(),
        frame.rectangles.len(),
        frame.text.len(),
        frame.images.len()
    );
    Ok(())
}
