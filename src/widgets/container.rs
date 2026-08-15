// container.rs

use crate::{LogicalConstraints, LogicalSize};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Container { pub size: LogicalSize }
impl Container {
    pub const fn new(size: LogicalSize) -> Self { Self { size } }
    pub fn layout(self, constraints: LogicalConstraints) -> LogicalSize { constraints.constrain(self.size) }
}
