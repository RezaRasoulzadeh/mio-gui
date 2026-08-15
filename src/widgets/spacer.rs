// spacer.rs

use crate::{LogicalConstraints, LogicalSize};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spacer {
    pub size: LogicalSize,
}

impl Spacer {
    pub const fn new(size: LogicalSize) -> Self {
        Self { size }
    }

    pub fn layout(self, constraints: LogicalConstraints) -> LogicalSize {
        constraints.constrain(self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::Spacer;
    use crate::{LogicalConstraints, LogicalSize};

    #[test]
    fn spacer_respects_its_preferred_size_and_constraints() {
        let spacer = Spacer::new(LogicalSize::new(24.0, 12.0));
        assert_eq!(
            spacer.layout(LogicalConstraints::unconstrained()),
            spacer.size
        );
        assert_eq!(
            spacer.layout(LogicalConstraints::tight(LogicalSize::new(8.0, 9.0))),
            LogicalSize::new(8.0, 9.0)
        );
    }
}
