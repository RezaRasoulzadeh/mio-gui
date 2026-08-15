// container_surface.rs

use mio_gui::{
    Container, Direction, LogicalConstraints, LogicalPoint, LogicalSize, Surface, TextSystem,
    ThemeController, ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement,
    WidgetTree,
};

fn main() {
    let mut tree = WidgetTree::new(Widget::from(Container::new(LogicalSize::new(120.0, 72.0))));
    tree.append(
        tree.root(),
        Widget::from(Surface::new(LogicalSize::new(96.0, 48.0))),
    )
    .unwrap();
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(if id == tree.root() { 16.0 } else { 28.0 }, 16.0),
            LogicalConstraints::unconstrained(),
            Direction::Ltr,
        )
    });
    println!(
        "widgets={} surfaces={}",
        frame.geometry.paint_order().len(),
        frame.rectangles.len()
    );
}
