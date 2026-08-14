// interaction.rs

use std::collections::HashMap;

use crate::{FrameSnapshot, LogicalPoint, PointerId, WidgetId};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum PointerKind {
    #[default]
    Mouse,
    Touch,
    Pen,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractionInput {
    Move {
        id: PointerId,
        kind: PointerKind,
        position: LogicalPoint,
    },
    Down {
        id: PointerId,
        kind: PointerKind,
        position: LogicalPoint,
    },
    Up {
        id: PointerId,
        kind: PointerKind,
        position: LogicalPoint,
    },
    Cancel {
        id: PointerId,
        kind: PointerKind,
        position: LogicalPoint,
    },
    Scroll {
        position: LogicalPoint,
        delta: LogicalPoint,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractionEvent {
    HoverEnter,
    HoverLeave,
    Press,
    Release { inside: bool },
    DragStart,
    DragMove { delta: LogicalPoint },
    DragEnd { cancelled: bool },
    Scroll { delta: LogicalPoint },
    TouchStart,
    TouchMove { delta: LogicalPoint },
    TouchEnd { cancelled: bool },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetedInteraction {
    pub target: WidgetId,
    pub pointer: Option<PointerId>,
    pub position: LogicalPoint,
    pub event: InteractionEvent,
}

#[derive(Clone, Copy, Debug)]
struct PressState {
    target: WidgetId,
    last: LogicalPoint,
    origin: LogicalPoint,
    dragging: bool,
    kind: PointerKind,
}

#[derive(Clone, Debug)]
pub struct InteractionState {
    drag_threshold: f32,
    hovered: Option<WidgetId>,
    pressed: HashMap<PointerId, PressState>,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self::new(4.0)
    }
}

impl InteractionState {
    pub fn new(drag_threshold: f32) -> Self {
        Self {
            drag_threshold: if drag_threshold.is_finite() {
                drag_threshold.max(0.0)
            } else {
                4.0
            },
            hovered: None,
            pressed: HashMap::new(),
        }
    }

    pub fn process(
        &mut self,
        snapshot: &FrameSnapshot,
        input: InteractionInput,
    ) -> Vec<TargetedInteraction> {
        match input {
            InteractionInput::Move { id, kind, position } => {
                self.move_pointer(snapshot, id, kind, position)
            }
            InteractionInput::Down { id, kind, position } => {
                self.down(snapshot, id, kind, position)
            }
            InteractionInput::Up {
                id,
                kind: _,
                position,
            } => self.finish(snapshot, id, position, false),
            InteractionInput::Cancel {
                id,
                kind: _,
                position,
            } => self.finish(snapshot, id, position, true),
            InteractionInput::Scroll { position, delta } => snapshot
                .hit_test(position)
                .map(|target| {
                    vec![TargetedInteraction {
                        target,
                        pointer: None,
                        position,
                        event: InteractionEvent::Scroll { delta },
                    }]
                })
                .unwrap_or_default(),
        }
    }

    pub fn retain_valid(&mut self, snapshot: &FrameSnapshot) {
        self.hovered = self
            .hovered
            .filter(|target| snapshot.get(*target).is_some());
        self.pressed
            .retain(|_, press| snapshot.get(press.target).is_some());
    }

    fn move_pointer(
        &mut self,
        snapshot: &FrameSnapshot,
        id: PointerId,
        kind: PointerKind,
        position: LogicalPoint,
    ) -> Vec<TargetedInteraction> {
        let mut output = Vec::new();
        if kind == PointerKind::Mouse {
            let hit = snapshot.hit_test(position);
            if hit != self.hovered {
                if let Some(target) = self.hovered {
                    output.push(targeted(
                        target,
                        Some(id),
                        position,
                        InteractionEvent::HoverLeave,
                    ));
                }
                if let Some(target) = hit {
                    output.push(targeted(
                        target,
                        Some(id),
                        position,
                        InteractionEvent::HoverEnter,
                    ));
                }
                self.hovered = hit;
            }
        }
        let Some(press) = self.pressed.get_mut(&id) else {
            return output;
        };
        let delta = LogicalPoint::new(position.x - press.last.x, position.y - press.last.y);
        press.last = position;
        if press.kind == PointerKind::Touch {
            output.push(targeted(
                press.target,
                Some(id),
                position,
                InteractionEvent::TouchMove { delta },
            ));
            return output;
        }
        let distance_squared =
            (position.x - press.origin.x).powi(2) + (position.y - press.origin.y).powi(2);
        if !press.dragging && distance_squared >= self.drag_threshold.powi(2) {
            press.dragging = true;
            output.push(targeted(
                press.target,
                Some(id),
                position,
                InteractionEvent::DragStart,
            ));
        }
        if press.dragging {
            output.push(targeted(
                press.target,
                Some(id),
                position,
                InteractionEvent::DragMove { delta },
            ));
        }
        output
    }

    fn down(
        &mut self,
        snapshot: &FrameSnapshot,
        id: PointerId,
        kind: PointerKind,
        position: LogicalPoint,
    ) -> Vec<TargetedInteraction> {
        let Some(target) = snapshot.hit_test(position) else {
            return Vec::new();
        };
        self.pressed.insert(
            id,
            PressState {
                target,
                last: position,
                origin: position,
                dragging: false,
                kind,
            },
        );
        vec![targeted(
            target,
            Some(id),
            position,
            if kind == PointerKind::Touch {
                InteractionEvent::TouchStart
            } else {
                InteractionEvent::Press
            },
        )]
    }

    fn finish(
        &mut self,
        snapshot: &FrameSnapshot,
        id: PointerId,
        position: LogicalPoint,
        cancelled: bool,
    ) -> Vec<TargetedInteraction> {
        let Some(press) = self.pressed.remove(&id) else {
            return Vec::new();
        };
        let event = if press.kind == PointerKind::Touch {
            InteractionEvent::TouchEnd { cancelled }
        } else if press.dragging {
            InteractionEvent::DragEnd { cancelled }
        } else {
            InteractionEvent::Release {
                inside: !cancelled && snapshot.hit_test(position) == Some(press.target),
            }
        };
        vec![targeted(press.target, Some(id), position, event)]
    }
}

fn targeted(
    target: WidgetId,
    pointer: Option<PointerId>,
    position: LogicalPoint,
    event: InteractionEvent,
) -> TargetedInteraction {
    TargetedInteraction {
        target,
        pointer,
        position,
        event,
    }
}

#[cfg(test)]
mod tests {
    use super::{InteractionEvent, InteractionInput, InteractionState, PointerKind};
    use crate::{FrameSnapshot, LogicalPoint, LogicalRect, PointerId, WidgetGeometry, WidgetTree};

    fn fixture() -> (FrameSnapshot, crate::WidgetId) {
        let mut tree = WidgetTree::new(());
        let root = tree.root();
        let child = tree.append(root, ()).unwrap();
        let snapshot = FrameSnapshot::build(&tree, |id, _| {
            WidgetGeometry::new(if id == root {
                LogicalRect::from_xywh(0.0, 0.0, 100.0, 100.0)
            } else {
                LogicalRect::from_xywh(20.0, 20.0, 40.0, 40.0)
            })
        });
        (snapshot, child)
    }

    #[test]
    fn mouse_hover_transitions_only_when_target_changes() {
        let (snapshot, child) = fixture();
        let mut state = InteractionState::default();
        let move_to = |position| InteractionInput::Move {
            id: PointerId(1),
            kind: PointerKind::Mouse,
            position,
        };

        assert_eq!(
            state.process(&snapshot, move_to(LogicalPoint::new(25.0, 25.0)))[0].event,
            InteractionEvent::HoverEnter
        );
        assert!(
            state
                .process(&snapshot, move_to(LogicalPoint::new(30.0, 30.0)))
                .is_empty()
        );
        let leave = state.process(&snapshot, move_to(LogicalPoint::new(80.0, 80.0)));
        assert_eq!(leave[0].target, child);
        assert_eq!(leave[0].event, InteractionEvent::HoverLeave);
    }

    #[test]
    fn drag_starts_after_threshold_and_keeps_press_target() {
        let (snapshot, child) = fixture();
        let mut state = InteractionState::new(4.0);
        let id = PointerId(2);
        state.process(
            &snapshot,
            InteractionInput::Down {
                id,
                kind: PointerKind::Mouse,
                position: LogicalPoint::new(25.0, 25.0),
            },
        );
        let below_threshold = state.process(
            &snapshot,
            InteractionInput::Move {
                id,
                kind: PointerKind::Mouse,
                position: LogicalPoint::new(27.0, 25.0),
            },
        );
        assert!(below_threshold.iter().all(|interaction| !matches!(
            interaction.event,
            InteractionEvent::DragStart | InteractionEvent::DragMove { .. }
        )));
        let drag = state.process(
            &snapshot,
            InteractionInput::Move {
                id,
                kind: PointerKind::Mouse,
                position: LogicalPoint::new(70.0, 25.0),
            },
        );
        assert!(drag.iter().any(|interaction| {
            interaction.target == child && interaction.event == InteractionEvent::DragStart
        }));
        assert!(drag.iter().any(|interaction| {
            interaction.target == child
                && matches!(interaction.event, InteractionEvent::DragMove { .. })
        }));
        assert_eq!(
            state.process(
                &snapshot,
                InteractionInput::Up {
                    id,
                    kind: PointerKind::Mouse,
                    position: LogicalPoint::new(70.0, 25.0)
                }
            )[0]
            .event,
            InteractionEvent::DragEnd { cancelled: false }
        );
    }

    #[test]
    fn release_reports_whether_pointer_ended_inside_press_target() {
        let (snapshot, _) = fixture();
        let mut state = InteractionState::default();
        let id = PointerId(3);
        state.process(
            &snapshot,
            InteractionInput::Down {
                id,
                kind: PointerKind::Pen,
                position: LogicalPoint::new(25.0, 25.0),
            },
        );
        let release = state.process(
            &snapshot,
            InteractionInput::Up {
                id,
                kind: PointerKind::Pen,
                position: LogicalPoint::new(80.0, 80.0),
            },
        );
        assert_eq!(
            release[0].event,
            InteractionEvent::Release { inside: false }
        );
    }

    #[test]
    fn touch_and_scroll_are_targeted_independently() {
        let (snapshot, child) = fixture();
        let mut state = InteractionState::default();
        let id = PointerId(4);
        assert_eq!(
            state.process(
                &snapshot,
                InteractionInput::Down {
                    id,
                    kind: PointerKind::Touch,
                    position: LogicalPoint::new(25.0, 25.0)
                }
            )[0]
            .event,
            InteractionEvent::TouchStart
        );
        assert!(matches!(
            state.process(
                &snapshot,
                InteractionInput::Move {
                    id,
                    kind: PointerKind::Touch,
                    position: LogicalPoint::new(30.0, 35.0)
                }
            )[0]
            .event,
            InteractionEvent::TouchMove { .. }
        ));
        assert_eq!(
            state.process(
                &snapshot,
                InteractionInput::Cancel {
                    id,
                    kind: PointerKind::Touch,
                    position: LogicalPoint::new(30.0, 35.0)
                }
            )[0]
            .event,
            InteractionEvent::TouchEnd { cancelled: true }
        );
        let scroll = state.process(
            &snapshot,
            InteractionInput::Scroll {
                position: LogicalPoint::new(25.0, 25.0),
                delta: LogicalPoint::new(0.0, -12.0),
            },
        );
        assert_eq!(scroll[0].target, child);
        assert!(matches!(scroll[0].event, InteractionEvent::Scroll { .. }));
    }

    #[test]
    fn completion_preserves_the_pointer_kind_captured_on_press() {
        let (snapshot, _) = fixture();
        let mut state = InteractionState::default();
        let id = PointerId(5);
        state.process(
            &snapshot,
            InteractionInput::Down {
                id,
                kind: PointerKind::Touch,
                position: LogicalPoint::new(25.0, 25.0),
            },
        );

        let completed = state.process(
            &snapshot,
            InteractionInput::Up {
                id,
                kind: PointerKind::Mouse,
                position: LogicalPoint::new(25.0, 25.0),
            },
        );

        assert_eq!(
            completed[0].event,
            InteractionEvent::TouchEnd { cancelled: false }
        );
    }
}
