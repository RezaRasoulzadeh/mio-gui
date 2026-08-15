// image_icon.rs

use mio_gui::{
    Direction, Icon, Image, LogicalConstraints, LogicalPoint, LogicalSize, Mask, MaskShape,
    PixelFormat, PixelImage, TextSystem, ThemeController, ThemeDefinition, UserPreferences, Widget,
    WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image = PixelImage::new(
        2,
        1,
        PixelFormat::Rgba8,
        vec![255, 176, 0, 255, 31, 31, 35, 255],
    )?;
    let mut icon = Icon::new(PixelImage::new(
        2,
        2,
        PixelFormat::Alpha8,
        vec![0, 255, 255, 0],
    )?)?;
    icon.mirror_in_rtl = true;
    let mut tree = WidgetTree::new(Widget::from(Image::new(image).with_alternative_text("Mio")));
    tree.append(
        tree.root(),
        Widget::from(icon.with_alternative_text("Open")),
    )?;
    let mask_source = PixelImage::new(8, 8, PixelFormat::Rgba8, vec![255; 8 * 8 * 4])?;
    tree.append(
        tree.root(),
        Widget::from(Mask::new(mask_source, MaskShape::Circle).with_alternative_text("Profile")),
    )?;

    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(if id == tree.root() { 16.0 } else { 56.0 }, 16.0),
            LogicalConstraints::tight(LogicalSize::new(24.0, 24.0)),
            Direction::Rtl,
        )
    });

    println!(
        "widgets={} image_draws={} mirrored_icon={}",
        frame.geometry.paint_order().len(),
        frame.images.len(),
        frame.images[1].mirror_horizontal,
    );
    Ok(())
}
