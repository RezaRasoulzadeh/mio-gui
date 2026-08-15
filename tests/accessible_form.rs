// accessible_form.rs

use mio_gui::{
    ArrowKey, Button, Checkbox, Column, Direction, FocusManager, FocusPolicy, FocusSnapshot,
    FrameSnapshot, Key, KeyboardEvent, LogicalConstraints, LogicalPoint, LogicalRect, LogicalSize,
    Radio, SearchInput, Select, SelectOption, SemanticAction, SemanticRole, SemanticSnapshot,
    Semantics, Slider, Switch, TextArea, TextInput, TextSystem, ThemeController, ThemeDefinition,
    UserPreferences, Widget, WidgetFrame, WidgetGeometry, WidgetPlacement, WidgetTree,
    apply_focus_navigation, semantic_action_for_key,
};

#[derive(Clone)]
struct FormNode {
    focus: FocusPolicy,
    semantics: Semantics,
}

#[test]
fn retained_widgets_form_preserves_keyboard_and_semantic_order_in_both_directions() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        let mut column = Column::default();
        column.layout.gap = 12.0;
        let mut tree = WidgetTree::new(Widget::from(column));
        let root = tree.root();
        let name = tree
            .append(root, Widget::from(TextInput::new("Name")))
            .unwrap();
        let search = tree
            .append(root, Widget::from(SearchInput::new("Search")))
            .unwrap();
        let notes = tree
            .append(root, Widget::from(TextArea::new("Notes")))
            .unwrap();
        let country = tree
            .append(
                root,
                Widget::from(
                    Select::new(
                        "Country",
                        vec![
                            SelectOption::new("Iran", "ir"),
                            SelectOption::new("Japan", "jp"),
                        ],
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        let standard = tree
            .append(
                root,
                Widget::from(Radio::new("Standard delivery").with_group("delivery", "standard")),
            )
            .unwrap();
        let express = tree
            .append(
                root,
                Widget::from(Radio::new("Express delivery").with_group("delivery", "express")),
            )
            .unwrap();
        tree.select_radio(standard);
        let updates = tree
            .append(root, Widget::from(Checkbox::new("Receive updates")))
            .unwrap();
        let alerts = tree
            .append(root, Widget::from(Switch::new("Alerts")))
            .unwrap();
        let amount = tree
            .append(
                root,
                Widget::from(Slider::new("Amount", 0.0..=100.0, 50.0).unwrap()),
            )
            .unwrap();
        let submit = tree
            .append(root, Widget::from(Button::new("Submit")))
            .unwrap();
        let focus_snapshot = FocusSnapshot::build(&tree, |id, widget| {
            let mut policy = widget.focus_policy();
            if let Widget::Radio(radio) = widget {
                if let Some(group) = radio.group() {
                    policy.skip_tab_order = tree.radio_tab_stop(group) != Some(id);
                }
            }
            policy
        });
        let tab_controls = [
            name, search, notes, country, standard, updates, alerts, amount, submit,
        ];
        let semantic_controls = [
            name, search, notes, country, standard, express, updates, alerts, amount, submit,
        ];
        assert_eq!(focus_snapshot.tab_order(), &tab_controls);

        let theme = ThemeDefinition::default()
            .resolve(ThemeController::default(), UserPreferences::default());
        let mut text_system = TextSystem::new();
        let frame = WidgetFrame::build_composed(
            &tree,
            &mut text_system,
            &theme,
            WidgetPlacement::new(
                LogicalPoint::new(16.0, 16.0),
                LogicalConstraints::loose(LogicalSize::new(400.0, 480.0)),
                direction,
            ),
        );
        assert_eq!(
            semantic_controls.map(|id| frame.semantics.get(id).unwrap().semantics.role),
            [
                SemanticRole::TextField,
                SemanticRole::SearchField,
                SemanticRole::MultilineTextField,
                SemanticRole::ComboBox,
                SemanticRole::Radio,
                SemanticRole::Radio,
                SemanticRole::Checkbox,
                SemanticRole::Switch,
                SemanticRole::Slider,
                SemanticRole::Button,
            ]
        );
        assert!(
            semantic_controls.into_iter().all(|id| frame
                .geometry
                .get(id)
                .unwrap()
                .bounds
                .size
                .width
                > 0.0)
        );

        let mut focus = FocusManager::default();
        let tab = KeyboardEvent::pressed(Key::Tab);
        for expected in tab_controls.into_iter().chain([name]) {
            assert_eq!(
                apply_focus_navigation(&mut focus, &focus_snapshot, &tab, direction),
                Some(expected)
            );
        }
    }
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
