// modal.rs

use mio_gui::{
    Direction, LogicalConstraints, LogicalPoint, LogicalSize, Modal, TextSystem, ThemeController,
    ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() {
    let mut modal = Modal::new("Confirm deletion", LogicalSize::new(180.0, 80.0));
    modal.open = true;
    let tree = WidgetTree::new(Widget::from(modal));
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |_, _| {
        WidgetPlacement::new(
            LogicalPoint::default(),
            LogicalConstraints::tight(LogicalSize::new(320.0, 200.0)),
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
