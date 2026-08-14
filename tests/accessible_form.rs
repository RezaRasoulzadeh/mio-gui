// accessible_form.rs

use mio_gui::{
    ArrowKey, Direction, FocusManager, FocusPolicy, FocusSnapshot, FrameSnapshot, Key,
    KeyboardEvent, LogicalRect, SemanticAction, SemanticRole, SemanticSnapshot, Semantics,
    WidgetGeometry, WidgetTree, apply_focus_navigation, semantic_action_for_key,
};

#[derive(Clone)]
struct FormNode {
    focus: FocusPolicy,
    semantics: Semantics,
}

fn node(focusable: bool, semantics: Semantics) -> FormNode {
    FormNode {
        focus: if focusable {
            FocusPolicy::focusable()
        } else {
            FocusPolicy::default()
        },
        semantics,
    }
}

#[test]
fn representative_form_has_equivalent_keyboard_and_semantic_behavior_in_both_directions() {
    let mut tree = WidgetTree::new(node(false, Semantics::new(SemanticRole::Generic)));
    let root = tree.root();
    let name = tree
        .append(
            root,
            node(
                true,
                Semantics::new(SemanticRole::TextField).with_name("Name"),
            ),
        )
        .unwrap();
    let updates = tree
        .append(
            root,
            node(
                true,
                Semantics::new(SemanticRole::Checkbox)
                    .with_name("Receive updates")
                    .with_action(SemanticAction::Activate),
            ),
        )
        .unwrap();
    let amount = tree
        .append(
            root,
            node(
                true,
                Semantics::new(SemanticRole::Slider)
                    .with_name("Amount")
                    .with_value("50")
                    .with_action(SemanticAction::Increment)
                    .with_action(SemanticAction::Decrement),
            ),
        )
        .unwrap();
    let submit = tree
        .append(
            root,
            node(
                true,
                Semantics::new(SemanticRole::Button)
                    .with_name("Submit")
                    .with_action(SemanticAction::Activate),
            ),
        )
        .unwrap();
    let focus_snapshot = FocusSnapshot::build(&tree, |_, node| node.focus);
    let semantic_snapshot = SemanticSnapshot::build(&tree, |_, node| node.semantics.clone());

    assert_eq!(focus_snapshot.tab_order(), &[name, updates, amount, submit]);
    assert_eq!(
        semantic_snapshot.order(),
        &[root, name, updates, amount, submit]
    );

    for direction in [Direction::Ltr, Direction::Rtl] {
        let frame = FrameSnapshot::build(&tree, |id, _| {
            let index = semantic_snapshot
                .order()
                .iter()
                .position(|widget| *widget == id)
                .unwrap();
            let width = if id == root { 400.0 } else { 100.0 };
            let x = if id == root || direction == Direction::Ltr {
                0.0
            } else {
                300.0
            };
            WidgetGeometry::new(LogicalRect::from_xywh(x, index as f32 * 40.0, width, 32.0))
        });
        let mut focus = FocusManager::default();
        let tab = KeyboardEvent::pressed(Key::Tab);

        assert_eq!(
            apply_focus_navigation(&mut focus, &focus_snapshot, &tab, direction),
            Some(name)
        );
        assert_eq!(
            apply_focus_navigation(&mut focus, &focus_snapshot, &tab, direction),
            Some(updates)
        );
        assert_eq!(frame.get(updates).unwrap().bounds.size.width, 100.0);
        assert_eq!(
            semantic_action_for_key(
                &semantic_snapshot,
                updates,
                &KeyboardEvent::pressed(Key::Space),
                direction
            )
            .unwrap()
            .action,
            SemanticAction::Activate
        );
        assert_eq!(
            apply_focus_navigation(&mut focus, &focus_snapshot, &tab, direction),
            Some(amount)
        );
        assert_eq!(
            semantic_action_for_key(
                &semantic_snapshot,
                amount,
                &KeyboardEvent::pressed(Key::Arrow(ArrowKey::Right)),
                direction
            )
            .unwrap()
            .action,
            if direction == Direction::Ltr {
                SemanticAction::Increment
            } else {
                SemanticAction::Decrement
            }
        );
        assert_eq!(
            apply_focus_navigation(&mut focus, &focus_snapshot, &tab, direction),
            Some(submit)
        );
        assert_eq!(
            semantic_action_for_key(
                &semantic_snapshot,
                submit,
                &KeyboardEvent::pressed(Key::Enter),
                direction
            )
            .unwrap()
            .action,
            SemanticAction::Activate
        );
    }
}
