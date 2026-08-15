// keyboard.rs

use crate::{
    ArrowKey, Direction, EventControl, EventDispatch, EventPhase, FocusManager, FocusSnapshot,
    FrameSnapshot, SemanticAction, SemanticActionRequest, SemanticRole, SemanticSnapshot, WidgetId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Key {
    Tab,
    Enter,
    Space,
    Escape,
    Arrow(ArrowKey),
    Home,
    End,
    PageUp,
    PageDown,
    Backspace,
    Delete,
    Character(String),
    Unidentified,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyboardEvent {
    pub key: Key,
    pub state: KeyState,
    pub modifiers: KeyModifiers,
    pub repeat: bool,
}

impl KeyboardEvent {
    pub fn pressed(key: Key) -> Self {
        Self {
            key,
            state: KeyState::Pressed,
            modifiers: KeyModifiers::default(),
            repeat: false,
        }
    }
}

pub fn dispatch_keyboard_event(
    frame: &FrameSnapshot,
    focus: &FocusManager,
    event: &KeyboardEvent,
    deliver: impl FnMut(WidgetId, EventPhase, &KeyboardEvent) -> EventControl,
) -> EventDispatch {
    focus
        .focused()
        .map(|target| frame.dispatch_targeted_event(target, event, deliver))
        .unwrap_or_default()
}

pub fn apply_focus_navigation(
    focus: &mut FocusManager,
    snapshot: &FocusSnapshot,
    event: &KeyboardEvent,
    direction: Direction,
) -> Option<WidgetId> {
    if event.state != KeyState::Pressed || event.repeat {
        return focus.focused();
    }
    match &event.key {
        Key::Tab if event.modifiers.shift => {
            focus.traverse(snapshot, crate::FocusTraversal::Backward)
        }
        Key::Tab => focus.traverse(snapshot, crate::FocusTraversal::Forward),
        Key::Arrow(key) => focus.navigate_arrow(snapshot, *key, direction),
        _ => focus.focused(),
    }
}

pub fn semantic_action_for_key(
    semantics: &SemanticSnapshot,
    target: WidgetId,
    event: &KeyboardEvent,
    direction: Direction,
) -> Option<SemanticActionRequest> {
    if event.state != KeyState::Pressed || event.repeat {
        return None;
    }
    let role = semantics.get(target)?.semantics.role;
    let action = match (&event.key, role) {
        (Key::Enter, SemanticRole::Button | SemanticRole::Link | SemanticRole::MenuItem) => {
            SemanticAction::Activate
        }
        (
            Key::Space,
            SemanticRole::Button
            | SemanticRole::Checkbox
            | SemanticRole::Radio
            | SemanticRole::Switch,
        ) => SemanticAction::Activate,
        (Key::Enter | Key::Space, SemanticRole::ComboBox) => SemanticAction::ShowMenu,
        (Key::Arrow(ArrowKey::Up), SemanticRole::Slider) => SemanticAction::Increment,
        (Key::Arrow(ArrowKey::Down), SemanticRole::Slider) => SemanticAction::Decrement,
        (Key::Arrow(ArrowKey::Right), SemanticRole::Slider) if direction == Direction::Ltr => {
            SemanticAction::Increment
        }
        (Key::Arrow(ArrowKey::Right), SemanticRole::Slider) => SemanticAction::Decrement,
        (Key::Arrow(ArrowKey::Left), SemanticRole::Slider) if direction == Direction::Ltr => {
            SemanticAction::Decrement
        }
        (Key::Arrow(ArrowKey::Left), SemanticRole::Slider) => SemanticAction::Increment,
        _ => return None,
    };
    semantics.request_action(target, action)
}

#[cfg(test)]
mod tests {
    use super::{
        Key, KeyModifiers, KeyState, KeyboardEvent, apply_focus_navigation,
        dispatch_keyboard_event, semantic_action_for_key,
    };
    use crate::{
        ArrowKey, Direction, EventControl, EventDelivery, EventPhase, FocusManager, FocusPolicy,
        FocusSnapshot, FrameSnapshot, LogicalRect, SemanticAction, SemanticRole, SemanticSnapshot,
        SemanticState, Semantics, WidgetGeometry, WidgetTree,
    };

    fn fixture() -> (
        WidgetTree<FocusPolicy>,
        FocusSnapshot,
        FrameSnapshot,
        [crate::WidgetId; 3],
    ) {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let first = tree.append(root, FocusPolicy::focusable()).unwrap();
        let second = tree.append(root, FocusPolicy::focusable()).unwrap();
        let focus = FocusSnapshot::build(&tree, |_, policy| *policy);
        let frame = FrameSnapshot::build(&tree, |_, _| {
            WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 100.0, 100.0))
        });
        (tree, focus, frame, [root, first, second])
    }

    #[test]
    fn keyboard_events_route_through_focused_widget_ancestry() {
        let (_, focus_snapshot, frame, [root, first, _]) = fixture();
        let mut focus = FocusManager::default();
        assert!(focus.focus(&focus_snapshot, first));
        let event = KeyboardEvent::pressed(Key::Enter);

        let dispatch =
            dispatch_keyboard_event(&frame, &focus, &event, |_, _, _| EventControl::Continue);

        assert_eq!(dispatch.target, Some(first));
        assert_eq!(
            dispatch.deliveries,
            [
                EventDelivery {
                    widget: root,
                    phase: EventPhase::Capture,
                },
                EventDelivery {
                    widget: first,
                    phase: EventPhase::Target,
                },
                EventDelivery {
                    widget: root,
                    phase: EventPhase::Bubble,
                },
            ]
        );
    }

    #[test]
    fn tab_and_shift_tab_move_in_semantic_order() {
        let (_, snapshot, _, [_, first, second]) = fixture();
        let mut focus = FocusManager::default();
        let tab = KeyboardEvent::pressed(Key::Tab);
        assert_eq!(
            apply_focus_navigation(&mut focus, &snapshot, &tab, Direction::Ltr),
            Some(first)
        );
        assert_eq!(
            apply_focus_navigation(&mut focus, &snapshot, &tab, Direction::Rtl),
            Some(second)
        );
        let shift_tab = KeyboardEvent {
            modifiers: KeyModifiers {
                shift: true,
                ..KeyModifiers::default()
            },
            ..KeyboardEvent::pressed(Key::Tab)
        };
        assert_eq!(
            apply_focus_navigation(&mut focus, &snapshot, &shift_tab, Direction::Rtl),
            Some(first)
        );
    }

    #[test]
    fn horizontal_arrows_use_local_direction() {
        let (_, snapshot, _, [_, first, second]) = fixture();
        let mut focus = FocusManager::default();
        assert!(focus.focus(&snapshot, first));
        let right = KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right));
        assert_eq!(
            apply_focus_navigation(&mut focus, &snapshot, &right, Direction::Ltr),
            Some(second)
        );
        assert_eq!(
            apply_focus_navigation(&mut focus, &snapshot, &right, Direction::Rtl),
            Some(first)
        );
    }

    #[test]
    fn released_and_repeated_navigation_keys_do_not_move_focus() {
        let (_, snapshot, _, [_, first, _]) = fixture();
        let mut focus = FocusManager::default();
        assert!(focus.focus(&snapshot, first));
        let released = KeyboardEvent {
            state: KeyState::Released,
            ..KeyboardEvent::pressed(Key::Tab)
        };
        let repeated = KeyboardEvent {
            repeat: true,
            ..KeyboardEvent::pressed(Key::Tab)
        };

        assert_eq!(
            apply_focus_navigation(&mut focus, &snapshot, &released, Direction::Ltr),
            Some(first)
        );
        assert_eq!(
            apply_focus_navigation(&mut focus, &snapshot, &repeated, Direction::Ltr),
            Some(first)
        );
    }

    #[test]
    fn activation_keys_require_matching_roles_and_declared_actions() {
        let mut tree = WidgetTree::new(Semantics::new(SemanticRole::Generic));
        let root = tree.root();
        let button = tree
            .append(
                root,
                Semantics::new(SemanticRole::Button).with_action(SemanticAction::Activate),
            )
            .unwrap();
        let link = tree
            .append(
                root,
                Semantics::new(SemanticRole::Link).with_action(SemanticAction::Activate),
            )
            .unwrap();
        let snapshot = SemanticSnapshot::build(&tree, |_, semantics| semantics.clone());

        assert!(
            semantic_action_for_key(
                &snapshot,
                button,
                &KeyboardEvent::pressed(Key::Space),
                Direction::Ltr,
            )
            .is_some()
        );
        assert!(
            semantic_action_for_key(
                &snapshot,
                link,
                &KeyboardEvent::pressed(Key::Enter),
                Direction::Ltr,
            )
            .is_some()
        );
        assert_eq!(
            semantic_action_for_key(
                &snapshot,
                link,
                &KeyboardEvent::pressed(Key::Space),
                Direction::Ltr,
            ),
            None
        );
    }

    #[test]
    fn slider_horizontal_actions_mirror_in_rtl() {
        let mut tree = WidgetTree::new(Semantics::new(SemanticRole::Generic));
        let root = tree.root();
        let slider = tree
            .append(
                root,
                Semantics::new(SemanticRole::Slider)
                    .with_action(SemanticAction::Increment)
                    .with_action(SemanticAction::Decrement),
            )
            .unwrap();
        let snapshot = SemanticSnapshot::build(&tree, |_, semantics| semantics.clone());
        let right = KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right));

        assert_eq!(
            semantic_action_for_key(&snapshot, slider, &right, Direction::Ltr)
                .unwrap()
                .action,
            SemanticAction::Increment
        );
        assert_eq!(
            semantic_action_for_key(&snapshot, slider, &right, Direction::Rtl)
                .unwrap()
                .action,
            SemanticAction::Decrement
        );
    }

    #[test]
    fn combo_box_opening_keys_request_the_declared_menu_action() {
        let mut tree = WidgetTree::new(Semantics::new(SemanticRole::Generic));
        let root = tree.root();
        let combo_box = tree
            .append(
                root,
                Semantics::new(SemanticRole::ComboBox).with_action(SemanticAction::ShowMenu),
            )
            .unwrap();
        let snapshot = SemanticSnapshot::build(&tree, |_, semantics| semantics.clone());

        for key in [Key::Enter, Key::Space] {
            assert_eq!(
                semantic_action_for_key(
                    &snapshot,
                    combo_box,
                    &KeyboardEvent::pressed(key),
                    Direction::Ltr,
                )
                .unwrap()
                .action,
                SemanticAction::ShowMenu
            );
        }
    }

    #[test]
    fn disabled_and_read_only_controls_reject_keyboard_actions() {
        let mut tree = WidgetTree::new(Semantics::new(SemanticRole::Generic));
        let root = tree.root();
        let disabled = tree
            .append(
                root,
                Semantics::new(SemanticRole::Button)
                    .with_action(SemanticAction::Activate)
                    .with_state(SemanticState {
                        disabled: true,
                        ..SemanticState::default()
                    }),
            )
            .unwrap();
        let read_only = tree
            .append(
                root,
                Semantics::new(SemanticRole::Slider)
                    .with_action(SemanticAction::Increment)
                    .with_state(SemanticState {
                        read_only: true,
                        ..SemanticState::default()
                    }),
            )
            .unwrap();
        let snapshot = SemanticSnapshot::build(&tree, |_, semantics| semantics.clone());

        assert_eq!(
            semantic_action_for_key(
                &snapshot,
                disabled,
                &KeyboardEvent::pressed(Key::Enter),
                Direction::Ltr,
            ),
            None
        );
        assert_eq!(
            semantic_action_for_key(
                &snapshot,
                read_only,
                &KeyboardEvent::pressed(Key::Arrow(ArrowKey::Up)),
                Direction::Ltr,
            ),
            None
        );
    }
}
