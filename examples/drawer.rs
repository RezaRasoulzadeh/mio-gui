// drawer.rs

use mio_gui::{
    Direction, Drawer, LogicalConstraints, LogicalPoint, LogicalSize, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() {
    let mut drawer = Drawer::new("Navigation", 240.0);
    drawer.open = true;
    let tree = WidgetTree::new(Widget::from(drawer));
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
        WidgetPlacement::new(
            LogicalPoint::default(),
            LogicalConstraints::tight(LogicalSize::new(800.0, 600.0)),
            Direction::Rtl,
        )
    });
    println!(
        "role={:?} overlay={:?} layers={}",
        frame.semantics.get(tree.root()).unwrap().semantics.role,
        frame.geometry.get(tree.root()).unwrap().bounds.size,
        frame.rectangles.len()
    );
}
