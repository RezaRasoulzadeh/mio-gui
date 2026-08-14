// event.rs

use std::collections::HashMap;

use crate::{FrameSnapshot, LogicalPoint, PhysicalPoint, ScaleFactor, WidgetId};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PointerId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    pub id: PointerId,
    pub phase: PointerPhase,
    pub position: LogicalPoint,
}

impl PointerEvent {
    pub fn from_physical(
        id: PointerId,
        phase: PointerPhase,
        position: PhysicalPoint,
        scale_factor: ScaleFactor,
    ) -> Self {
        Self {
            id,
            phase,
            position: position.to_logical(scale_factor),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PointerCapture {
    owners: HashMap<PointerId, WidgetId>,
}

impl PointerCapture {
    pub fn capture(
        &mut self,
        snapshot: &FrameSnapshot,
        pointer: PointerId,
        widget: WidgetId,
    ) -> bool {
        if snapshot.get(widget).is_none() {
            return false;
        }
        self.owners.insert(pointer, widget);
        true
    }

    pub fn release(&mut self, pointer: PointerId, widget: WidgetId) -> bool {
        if self.owners.get(&pointer) != Some(&widget) {
            return false;
        }
        self.owners.remove(&pointer);
        true
    }

    pub fn owner(&self, pointer: PointerId) -> Option<WidgetId> {
        self.owners.get(&pointer).copied()
    }

    pub fn retain_valid(&mut self, snapshot: &FrameSnapshot) {
        self.owners
            .retain(|_, widget| snapshot.get(*widget).is_some());
    }

    pub fn dispatch(
        &mut self,
        snapshot: &FrameSnapshot,
        event: &PointerEvent,
        deliver: impl FnMut(WidgetId, EventPhase, &PointerEvent) -> EventControl,
    ) -> EventDispatch {
        let captured = self
            .owner(event.id)
            .filter(|owner| snapshot.get(*owner).is_some());
        if captured.is_none() {
            self.owners.remove(&event.id);
        }
        let dispatch = if let Some(target) = captured {
            snapshot.dispatch_targeted_event(target, event, deliver)
        } else {
            snapshot.dispatch_pointer_event(event.position, event, deliver)
        };
        if matches!(event.phase, PointerPhase::Up | PointerPhase::Cancel) {
            self.owners.remove(&event.id);
        }
        dispatch
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EventPhase {
    Capture,
    Target,
    Bubble,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum EventControl {
    #[default]
    Continue,
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventDelivery {
    pub widget: WidgetId,
    pub phase: EventPhase,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventDispatch {
    pub target: Option<WidgetId>,
    pub deliveries: Vec<EventDelivery>,
    pub stopped: bool,
}

impl FrameSnapshot {
    pub fn dispatch_pointer_event<Event>(
        &self,
        position: LogicalPoint,
        event: &Event,
        deliver: impl FnMut(WidgetId, EventPhase, &Event) -> EventControl,
    ) -> EventDispatch {
        let Some(target) = self.hit_test(position) else {
            return EventDispatch::default();
        };
        self.dispatch_targeted_event(target, event, deliver)
    }

    pub fn dispatch_targeted_event<Event>(
        &self,
        target: WidgetId,
        event: &Event,
        mut deliver: impl FnMut(WidgetId, EventPhase, &Event) -> EventControl,
    ) -> EventDispatch {
        let Some(route) = self.route_to(target) else {
            return EventDispatch::default();
        };
        let mut dispatch = EventDispatch {
            target: Some(target),
            deliveries: Vec::new(),
            stopped: false,
        };

        for widget in route.iter().copied().take(route.len().saturating_sub(1)) {
            if record_delivery(
                &mut dispatch,
                widget,
                EventPhase::Capture,
                event,
                &mut deliver,
            ) {
                return dispatch;
            }
        }
        if record_delivery(
            &mut dispatch,
            target,
            EventPhase::Target,
            event,
            &mut deliver,
        ) {
            return dispatch;
        }
        for widget in route.iter().copied().rev().skip(1) {
            if record_delivery(
                &mut dispatch,
                widget,
                EventPhase::Bubble,
                event,
                &mut deliver,
            ) {
                return dispatch;
            }
        }
        dispatch
    }
}

fn record_delivery<Event>(
    dispatch: &mut EventDispatch,
    widget: WidgetId,
    phase: EventPhase,
    event: &Event,
    deliver: &mut impl FnMut(WidgetId, EventPhase, &Event) -> EventControl,
) -> bool {
    dispatch.deliveries.push(EventDelivery { widget, phase });
    if deliver(widget, phase, event) == EventControl::Stop {
        dispatch.stopped = true;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EventControl, EventDelivery, EventPhase, PointerCapture, PointerEvent, PointerId,
        PointerPhase,
    };
    use crate::{
        FrameSnapshot, LogicalPoint, LogicalRect, LogicalSize, Overflow, PhysicalPoint,
        ScaleFactor, WidgetGeometry, WidgetTree,
    };

    fn fixture() -> (FrameSnapshot, [crate::WidgetId; 3]) {
        let mut tree = WidgetTree::new(());
        let root = tree.root();
        let parent = tree.append(root, ()).unwrap();
        let child = tree.append(parent, ()).unwrap();
        let snapshot = FrameSnapshot::build(&tree, |id, _| WidgetGeometry {
            bounds: if id == root {
                LogicalRect::from_xywh(0.0, 0.0, 100.0, 100.0)
            } else if id == parent {
                LogicalRect::from_xywh(10.0, 10.0, 70.0, 70.0)
            } else {
                LogicalRect::from_xywh(20.0, 20.0, 30.0, 30.0)
            },
            overflow: Overflow::Clip,
        });
        (snapshot, [root, parent, child])
    }

    #[test]
    fn event_runs_capture_target_and_bubble_phases() {
        let (snapshot, [root, parent, child]) = fixture();
        let dispatch =
            snapshot.dispatch_pointer_event(LogicalPoint::new(25.0, 25.0), &"press", |_, _, _| {
                EventControl::Continue
            });

        assert_eq!(dispatch.target, Some(child));
        assert_eq!(
            dispatch.deliveries,
            [
                EventDelivery {
                    widget: root,
                    phase: EventPhase::Capture
                },
                EventDelivery {
                    widget: parent,
                    phase: EventPhase::Capture
                },
                EventDelivery {
                    widget: child,
                    phase: EventPhase::Target
                },
                EventDelivery {
                    widget: parent,
                    phase: EventPhase::Bubble
                },
                EventDelivery {
                    widget: root,
                    phase: EventPhase::Bubble
                },
            ]
        );
    }

    #[test]
    fn propagation_stop_prevents_later_phases() {
        let (snapshot, [root, parent, child]) = fixture();
        let dispatch = snapshot.dispatch_targeted_event(child, &(), |widget, phase, _| {
            if widget == parent && phase == EventPhase::Capture {
                EventControl::Stop
            } else {
                EventControl::Continue
            }
        });

        assert!(dispatch.stopped);
        assert_eq!(
            dispatch.deliveries,
            [
                EventDelivery {
                    widget: root,
                    phase: EventPhase::Capture
                },
                EventDelivery {
                    widget: parent,
                    phase: EventPhase::Capture
                },
            ]
        );
    }

    #[test]
    fn clipped_and_missing_targets_receive_nothing() {
        let (snapshot, [_, _, child]) = fixture();

        assert_eq!(
            snapshot.dispatch_pointer_event(LogicalPoint::new(110.0, 25.0), &(), |_, _, _| {
                EventControl::Continue
            }),
            Default::default()
        );
        assert_eq!(
            snapshot.dispatch_targeted_event(
                crate::WidgetId::from_test_value(child.get() + 99),
                &(),
                |_, _, _| EventControl::Continue
            ),
            Default::default()
        );
    }

    #[test]
    fn captured_pointer_routes_outside_owner_bounds_until_up() {
        let (snapshot, [root, parent, child]) = fixture();
        let pointer = PointerId(7);
        let mut capture = PointerCapture::default();
        assert!(capture.capture(&snapshot, pointer, child));

        let moved = capture.dispatch(
            &snapshot,
            &PointerEvent {
                id: pointer,
                phase: PointerPhase::Move,
                position: LogicalPoint::new(95.0, 95.0),
            },
            |_, _, _| EventControl::Continue,
        );
        assert_eq!(moved.target, Some(child));
        assert_eq!(
            moved.deliveries.first(),
            Some(&EventDelivery {
                widget: root,
                phase: EventPhase::Capture
            })
        );
        assert_eq!(capture.owner(pointer), Some(child));

        let released = capture.dispatch(
            &snapshot,
            &PointerEvent {
                id: pointer,
                phase: PointerPhase::Up,
                position: LogicalPoint::new(95.0, 95.0),
            },
            |_, _, _| EventControl::Continue,
        );
        assert_eq!(released.target, Some(child));
        assert_eq!(capture.owner(pointer), None);
        assert!(
            released
                .deliveries
                .iter()
                .any(|delivery| delivery.widget == parent)
        );
    }

    #[test]
    fn captures_are_pointer_specific_and_owner_guarded() {
        let (snapshot, [_, parent, child]) = fixture();
        let first = PointerId(1);
        let second = PointerId(2);
        let mut capture = PointerCapture::default();

        assert!(capture.capture(&snapshot, first, parent));
        assert!(capture.capture(&snapshot, second, child));
        assert!(!capture.release(first, child));
        assert_eq!(capture.owner(first), Some(parent));
        assert_eq!(capture.owner(second), Some(child));
        assert!(capture.release(first, parent));
        assert_eq!(capture.owner(first), None);
    }

    #[test]
    fn new_snapshot_prunes_removed_capture_owner() {
        let mut tree = WidgetTree::new(());
        let root = tree.root();
        let child = tree.append(root, ()).unwrap();
        let old_snapshot = FrameSnapshot::build(&tree, |_, _| {
            WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 50.0, 50.0))
        });
        let pointer = PointerId(4);
        let mut capture = PointerCapture::default();
        assert!(capture.capture(&old_snapshot, pointer, child));
        tree.remove_subtree(child).unwrap();
        let new_snapshot = FrameSnapshot::build(&tree, |_, _| {
            WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 50.0, 50.0))
        });

        capture.retain_valid(&new_snapshot);

        assert_eq!(capture.owner(pointer), None);
        assert!(!capture.capture(&new_snapshot, pointer, child));
    }

    #[test]
    fn nested_event_route_survives_resize_and_dpi_rebuilds() {
        let mut tree = WidgetTree::new(());
        let root = tree.root();
        let parent = tree.append(root, ()).unwrap();
        let child = tree.append(parent, ()).unwrap();
        let pointer = PointerId(9);

        for (viewport, scale) in [
            (LogicalSize::new(800.0, 600.0), 1.0),
            (LogicalSize::new(1200.0, 900.0), 1.25),
            (LogicalSize::new(640.0, 480.0), 2.0),
        ] {
            let snapshot = FrameSnapshot::build(&tree, |id, _| {
                let bounds = if id == root {
                    LogicalRect::from_xywh(0.0, 0.0, viewport.width, viewport.height)
                } else if id == parent {
                    LogicalRect::from_xywh(
                        viewport.width * 0.25,
                        viewport.height * 0.25,
                        viewport.width * 0.5,
                        viewport.height * 0.5,
                    )
                } else {
                    LogicalRect::from_xywh(
                        viewport.width * 0.375,
                        viewport.height * 0.375,
                        viewport.width * 0.25,
                        viewport.height * 0.25,
                    )
                };
                WidgetGeometry {
                    bounds,
                    overflow: Overflow::Clip,
                }
            });
            let scale_factor = ScaleFactor::new(scale).unwrap();
            let logical_position = LogicalPoint::new(viewport.width * 0.5, viewport.height * 0.5);
            let physical_position = PhysicalPoint::new(
                logical_position.x * scale_factor.get(),
                logical_position.y * scale_factor.get(),
            );
            let event = PointerEvent::from_physical(
                pointer,
                PointerPhase::Down,
                physical_position,
                scale_factor,
            );
            let dispatch = snapshot
                .dispatch_pointer_event(event.position, &event, |_, _, _| EventControl::Continue);

            assert_eq!(event.position, logical_position);
            assert_eq!(dispatch.target, Some(child));
            assert_eq!(
                dispatch.deliveries,
                [
                    EventDelivery {
                        widget: root,
                        phase: EventPhase::Capture,
                    },
                    EventDelivery {
                        widget: parent,
                        phase: EventPhase::Capture,
                    },
                    EventDelivery {
                        widget: child,
                        phase: EventPhase::Target,
                    },
                    EventDelivery {
                        widget: parent,
                        phase: EventPhase::Bubble,
                    },
                    EventDelivery {
                        widget: root,
                        phase: EventPhase::Bubble,
                    },
                ]
            );
        }
    }
}
