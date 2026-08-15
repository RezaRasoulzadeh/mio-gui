// layout_views.rs

use mio_gui::{
    Column, Direction, LogicalConstraints, LogicalPoint, LogicalSize, Row, ScrollOffset,
    ScrollView, Spacer, Stack, TextSystem, ThemeController, ThemeDefinition, UserPreferences,
    Widget, WidgetFrame, WidgetPlacement, WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = WidgetTree::new(Widget::from(Column::default()));
    let root = tree.root();
    let row = tree.append(root, Widget::from(Row::default()))?;
    tree.append(row, Widget::from(Spacer::new(LogicalSize::new(16.0, 8.0))))?;
    tree.append(row, Widget::from(Spacer::new(LogicalSize::new(24.0, 8.0))))?;
    let stack = tree.append(root, Widget::from(Stack))?;
    tree.append(
        stack,
        Widget::from(Spacer::new(LogicalSize::new(40.0, 12.0))),
    )?;
    tree.append(
        stack,
        Widget::from(Spacer::new(LogicalSize::new(12.0, 6.0))),
    )?;
    let scroll = tree.append(
        root,
        Widget::from(ScrollView {
            viewport: Some(LogicalSize::new(32.0, 10.0)),
            offset: ScrollOffset::new(8.0, 0.0),
            ..ScrollView::default()
        }),
    )?;
    tree.append(
        scroll,
        Widget::from(Spacer::new(LogicalSize::new(64.0, 10.0))),
    )?;

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
        "widgets={} viewport={:?}",
        frame.geometry.paint_order().len(),
        frame.geometry.get(scroll).unwrap().bounds
    );
    Ok(())
}
