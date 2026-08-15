// spacer_divider.rs

use mio_gui::{Divider, LogicalConstraints, LogicalPoint, LogicalSize, Spacer, Widget, WidgetTree};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = WidgetTree::new(Widget::from(Spacer::new(LogicalSize::new(24.0, 8.0))));
    tree.append(tree.root(), Widget::from(Divider::horizontal()))?;
    let divider = Divider::horizontal();
    let size = divider.layout(LogicalConstraints::tight(LogicalSize::new(120.0, 1.0)));
    let draw = divider.draw(LogicalPoint::new(16.0, 32.0), size, [0.5; 4]);
    println!("widgets={} divider={:?}", tree.len(), draw.size);
    Ok(())
}
