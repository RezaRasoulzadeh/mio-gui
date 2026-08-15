use mio_gui::{
    Direction, Loading, LogicalConstraints, LogicalPoint, LogicalSize, Skeleton, TextSystem,
    ThemeController, ThemeDefinition, UserPreferences, Widget, WidgetFrame, WidgetPlacement,
    WidgetTree,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut loading = Loading::new("Loading profile");
    loading.set_phase(0.4);
    let mut skeleton = Skeleton::new(LogicalSize::new(180.0, 24.0));
    skeleton.radius = 8.0;
    let mut tree = WidgetTree::new(Widget::from(loading));
    tree.append(tree.root(), Widget::from(skeleton))?;
    let theme =
        ThemeDefinition::default().resolve(ThemeController::default(), UserPreferences::default());
    let mut text_system = TextSystem::new();
    let frame = WidgetFrame::build(&tree, &mut text_system, &theme, |id, _| {
        WidgetPlacement::new(
            LogicalPoint::new(16.0, if id == tree.root() { 16.0 } else { 48.0 }),
            LogicalConstraints::unconstrained(),
            Direction::Rtl,
        )
    });
    println!("placeholder surfaces={}", frame.rectangles.len());
    Ok(())
}
