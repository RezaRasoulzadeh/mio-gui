// accesskit_adapter.rs

use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Invalid, Node,
    NodeId, Rect, Role, TextPosition, TextSelection, Toggled, Tree, TreeId, TreeUpdate,
};

use crate::{
    FrameSnapshot, ScaleFactor, SemanticAction, SemanticActionRequest, SemanticActionValue,
    SemanticRole, SemanticSnapshot, WidgetId,
};

fn build_tree_update(
    semantics: &SemanticSnapshot,
    frame: &FrameSnapshot,
    scale_factor: ScaleFactor,
    focused: Option<WidgetId>,
) -> Option<TreeUpdate> {
    let root = semantics.root()?;
    let synthetic = synthetic_ids(semantics);
    let mut nodes = Vec::new();
    for id in semantics.order().iter().copied() {
        if let Some(semantic_node) = semantics.get(id) {
            let mut node = Node::new(map_role(semantic_node.semantics.role));
            let mut children = semantic_node
                .children
                .iter()
                .map(|child| NodeId(child.get()))
                .collect::<Vec<_>>();
            if let Some(run) = synthetic.text_runs.get(&id) {
                children.push(*run);
            }
            children.extend(
                semantic_node
                    .semantics
                    .virtual_children()
                    .iter()
                    .enumerate()
                    .filter_map(|(index, _)| synthetic.virtual_children.get(&(id, index)).copied()),
            );
            node.set_children(children);
            if let Some(index) = semantic_node.semantics.active_virtual_child() {
                if let Some(active) = synthetic.virtual_children.get(&(id, index)) {
                    node.set_active_descendant(*active);
                }
            }
            if let Some(name) = &semantic_node.semantics.name {
                node.set_label(name.clone());
            }
            if let Some(value) = &semantic_node.semantics.value {
                node.set_value(value.clone());
            }
            if let Some(placeholder) = &semantic_node.semantics.placeholder {
                node.set_placeholder(placeholder.clone());
            }
            if let Some(numeric) = semantic_node.semantics.numeric_value {
                node.set_numeric_value(numeric.value());
                node.set_min_numeric_value(*numeric.range().start());
                node.set_max_numeric_value(*numeric.range().end());
                if let Some(step) = numeric.step() {
                    node.set_numeric_value_step(step);
                }
            }
            let state = semantic_node.semantics.state;
            if state.disabled {
                node.set_disabled();
            }
            node.set_selected(state.selected);
            if state.read_only {
                node.set_read_only();
            }
            if state.required {
                node.set_required();
            }
            if state.invalid {
                node.set_invalid(Invalid::True);
            }
            if let Some(checked) = state.checked {
                node.set_toggled(Toggled::from(checked));
            }
            if let Some(expanded) = state.expanded {
                node.set_expanded(expanded);
            }
            for action in semantic_node.semantics.actions() {
                node.add_action(map_action(action));
            }
            if let (Some(text), Some(run)) = (
                &semantic_node.semantics.editable_text,
                synthetic.text_runs.get(&id),
            ) {
                node.set_text_selection(TextSelection {
                    anchor: TextPosition {
                        node: *run,
                        character_index: text.character_index(text.anchor()).unwrap(),
                    },
                    focus: TextPosition {
                        node: *run,
                        character_index: text.character_index(text.caret()).unwrap(),
                    },
                });
            }
            let mut physical_bounds = None;
            if let Some(frame_node) = frame.get(id) {
                let bounds = frame_node.bounds.to_physical(scale_factor);
                let bounds = Rect {
                    x0: f64::from(bounds.min_x()),
                    y0: f64::from(bounds.min_y()),
                    x1: f64::from(bounds.max_x()),
                    y1: f64::from(bounds.max_y()),
                };
                node.set_bounds(bounds);
                physical_bounds = Some(bounds);
            }
            nodes.push((NodeId(id.get()), node));
            if let (Some(text), Some(run)) = (
                &semantic_node.semantics.editable_text,
                synthetic.text_runs.get(&id),
            ) {
                let mut text_node = Node::new(Role::TextRun);
                text_node.set_value(text.text());
                text_node.set_character_lengths(text.character_lengths());
                if let Some(bounds) = physical_bounds {
                    text_node.set_bounds(bounds);
                }
                nodes.push((*run, text_node));
            }
            for (index, child) in semantic_node
                .semantics
                .virtual_children()
                .iter()
                .enumerate()
            {
                let child_id = synthetic.virtual_children[&(id, index)];
                let mut child_node = Node::new(map_role(child.role));
                if let Some(name) = &child.name {
                    child_node.set_label(name.clone());
                }
                if let Some(value) = &child.value {
                    child_node.set_value(value.clone());
                }
                if child.state.disabled {
                    child_node.set_disabled();
                }
                child_node.set_selected(child.state.selected);
                if let Some(bounds) = semantic_node.semantics.virtual_child_bounds(index) {
                    let bounds = bounds.to_physical(scale_factor);
                    child_node.set_bounds(Rect {
                        x0: f64::from(bounds.min_x()),
                        y0: f64::from(bounds.min_y()),
                        x1: f64::from(bounds.max_x()),
                        y1: f64::from(bounds.max_y()),
                    });
                }
                for action in child.actions() {
                    child_node.add_action(map_action(action));
                }
                nodes.push((child_id, child_node));
            }
        }
    }
    let mut tree = Tree::new(NodeId(root.get()));
    tree.toolkit_name = Some("Mio-GUI".to_owned());
    tree.toolkit_version = Some(env!("CARGO_PKG_VERSION").to_owned());
    let focus = focused
        .filter(|widget| semantics.get(*widget).is_some())
        .unwrap_or(root);
    Some(TreeUpdate {
        nodes,
        tree: Some(tree),
        tree_id: TreeId::ROOT,
        focus: NodeId(focus.get()),
    })
}

struct SyntheticIds {
    text_runs: std::collections::HashMap<WidgetId, NodeId>,
    virtual_children: std::collections::HashMap<(WidgetId, usize), NodeId>,
}

fn synthetic_ids(semantics: &SemanticSnapshot) -> SyntheticIds {
    let mut used = semantics
        .order()
        .iter()
        .map(|id| id.get())
        .collect::<std::collections::HashSet<_>>();
    let mut next = u64::MAX;
    let mut text_runs = std::collections::HashMap::new();
    let mut virtual_children = std::collections::HashMap::new();
    for id in semantics.order().iter().copied() {
        let node = semantics.get(id).unwrap();
        if node.semantics.editable_text.is_some() {
            while used.contains(&next) {
                next = next.wrapping_sub(1);
            }
            text_runs.insert(id, NodeId(next));
            used.insert(next);
            next = next.wrapping_sub(1);
        }
        for index in 0..node.semantics.virtual_children().len() {
            while used.contains(&next) {
                next = next.wrapping_sub(1);
            }
            virtual_children.insert((id, index), NodeId(next));
            used.insert(next);
            next = next.wrapping_sub(1);
        }
    }
    SyntheticIds {
        text_runs,
        virtual_children,
    }
}

fn translate_action(
    semantics: &SemanticSnapshot,
    request: &ActionRequest,
) -> Option<SemanticActionRequest> {
    let target = semantics
        .order()
        .iter()
        .copied()
        .find(|widget| widget.get() == request.target_node.0);
    let action = unmap_action(request.action)?;
    if target.is_none() {
        let synthetic = synthetic_ids(semantics);
        let ((target, index), _) = synthetic
            .virtual_children
            .iter()
            .find(|(_, id)| **id == request.target_node)?;
        if request.data.is_some() {
            return None;
        }
        return semantics.request_virtual_child_action(*target, *index, action);
    }
    let target = target?;
    if action == SemanticAction::SetTextSelection {
        let accesskit::ActionData::SetTextSelection(selection) = request.data.as_ref()? else {
            return None;
        };
        let text = semantics.get(target)?.semantics.editable_text.as_ref()?;
        let run = *synthetic_ids(semantics).text_runs.get(&target)?;
        if selection.anchor.node != run || selection.focus.node != run {
            return None;
        }
        return semantics.request_action_with_value(
            target,
            action,
            Some(SemanticActionValue::TextSelection {
                anchor: text.byte_offset(selection.anchor.character_index)?,
                caret: text.byte_offset(selection.focus.character_index)?,
            }),
        );
    }
    let value = match request.data.as_ref() {
        Some(accesskit::ActionData::Value(value)) => {
            Some(SemanticActionValue::Text(value.to_string()))
        }
        Some(accesskit::ActionData::NumericValue(value)) => {
            Some(SemanticActionValue::Number(*value))
        }
        Some(_) => return None,
        None => None,
    };
    semantics.request_action_with_value(target, action, value)
}

fn map_role(role: SemanticRole) -> Role {
    match role {
        SemanticRole::Generic => Role::GenericContainer,
        SemanticRole::Button => Role::Button,
        SemanticRole::Checkbox => Role::CheckBox,
        SemanticRole::Radio => Role::RadioButton,
        SemanticRole::Switch => Role::Switch,
        SemanticRole::Slider => Role::Slider,
        SemanticRole::ComboBox => Role::ComboBox,
        SemanticRole::Text => Role::Label,
        SemanticRole::TextField => Role::TextInput,
        SemanticRole::MultilineTextField => Role::MultilineTextInput,
        SemanticRole::SearchField => Role::SearchInput,
        SemanticRole::Link => Role::Link,
        SemanticRole::Image => Role::Image,
        SemanticRole::Heading => Role::Heading,
        SemanticRole::List => Role::List,
        SemanticRole::ListItem => Role::ListItem,
        SemanticRole::Menu => Role::Menu,
        SemanticRole::MenuItem => Role::MenuItem,
        SemanticRole::ListBoxOption => Role::ListBoxOption,
        SemanticRole::Dialog => Role::Dialog,
        SemanticRole::Alert => Role::Alert,
        SemanticRole::Progress => Role::ProgressIndicator,
        SemanticRole::Group => Role::Group,
        SemanticRole::Timer => Role::Timer,
        SemanticRole::Table => Role::Table,
        SemanticRole::Cell => Role::Cell,
        SemanticRole::TabList => Role::TabList,
        SemanticRole::Tab => Role::Tab,
    }
}

fn map_action(action: SemanticAction) -> Action {
    match action {
        SemanticAction::Focus => Action::Focus,
        SemanticAction::Blur => Action::Blur,
        SemanticAction::Activate => Action::Click,
        SemanticAction::Increment => Action::Increment,
        SemanticAction::Decrement => Action::Decrement,
        SemanticAction::SetValue => Action::SetValue,
        SemanticAction::SetTextSelection => Action::SetTextSelection,
        SemanticAction::ShowMenu => Action::ShowContextMenu,
        SemanticAction::ScrollIntoView => Action::ScrollIntoView,
    }
}

fn unmap_action(action: Action) -> Option<SemanticAction> {
    match action {
        Action::Focus => Some(SemanticAction::Focus),
        Action::Blur => Some(SemanticAction::Blur),
        Action::Click => Some(SemanticAction::Activate),
        Action::Increment => Some(SemanticAction::Increment),
        Action::Decrement => Some(SemanticAction::Decrement),
        Action::SetValue => Some(SemanticAction::SetValue),
        Action::SetTextSelection => Some(SemanticAction::SetTextSelection),
        Action::ShowContextMenu => Some(SemanticAction::ShowMenu),
        Action::ScrollIntoView => Some(SemanticAction::ScrollIntoView),
        _ => None,
    }
}

pub struct PlatformAccessibility {
    adapter: accesskit_winit::Adapter,
    receiver: std::sync::mpsc::Receiver<ActionRequest>,
    latest: std::sync::Arc<std::sync::Mutex<Option<TreeUpdate>>>,
}

impl PlatformAccessibility {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        window: &winit::window::Window,
        semantics: &SemanticSnapshot,
        frame: &FrameSnapshot,
        scale_factor: ScaleFactor,
        focused: Option<WidgetId>,
    ) -> Option<Self> {
        let initial = build_tree_update(semantics, frame, scale_factor, focused)?;
        let latest = std::sync::Arc::new(std::sync::Mutex::new(Some(initial)));
        let (sender, receiver) = std::sync::mpsc::channel();
        let adapter = accesskit_winit::Adapter::with_direct_handlers(
            event_loop,
            window,
            InitialTree {
                latest: latest.clone(),
            },
            ActionQueue { sender },
            Deactivation,
        );
        Some(Self {
            adapter,
            receiver,
            latest,
        })
    }

    pub fn process_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) {
        self.adapter.process_event(window, event);
    }

    pub fn update(
        &mut self,
        semantics: &SemanticSnapshot,
        frame: &FrameSnapshot,
        scale_factor: ScaleFactor,
        focused: Option<WidgetId>,
    ) -> bool {
        let Some(update) = build_tree_update(semantics, frame, scale_factor, focused) else {
            return false;
        };
        *self.latest.lock().unwrap() = Some(update.clone());
        self.adapter.update_if_active(|| update);
        true
    }

    pub fn drain_actions(&self, semantics: &SemanticSnapshot) -> Vec<SemanticActionRequest> {
        self.receiver
            .try_iter()
            .filter_map(|request| translate_action(semantics, &request))
            .collect()
    }
}

struct InitialTree {
    latest: std::sync::Arc<std::sync::Mutex<Option<TreeUpdate>>>,
}

impl ActivationHandler for InitialTree {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.latest.lock().unwrap().clone()
    }
}

struct ActionQueue {
    sender: std::sync::mpsc::Sender<ActionRequest>,
}

impl ActionHandler for ActionQueue {
    fn do_action(&mut self, request: ActionRequest) {
        let _ = self.sender.send(request);
    }
}

struct Deactivation;

impl DeactivationHandler for Deactivation {
    fn deactivate_accessibility(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::{build_tree_update, map_role, translate_action};
    use crate::{
        FrameSnapshot, LogicalRect, ScaleFactor, SemanticAction, SemanticRole, SemanticSnapshot,
        Semantics, WidgetGeometry, WidgetTree,
    };
    use accesskit::{Action, ActionRequest, NodeId, Role, TextPosition, TextSelection, TreeId};

    fn fixture() -> (
        SemanticSnapshot,
        FrameSnapshot,
        crate::WidgetId,
        crate::WidgetId,
    ) {
        let mut tree = WidgetTree::new(Semantics::new(SemanticRole::Generic));
        let root = tree.root();
        let button = tree
            .append(
                root,
                Semantics::new(SemanticRole::Button)
                    .with_name("Save")
                    .with_action(SemanticAction::Activate),
            )
            .unwrap();
        let semantics = SemanticSnapshot::build(&tree, |_, semantics| semantics.clone());
        let frame = FrameSnapshot::build(&tree, |id, _| {
            WidgetGeometry::new(if id == root {
                LogicalRect::from_xywh(0.0, 0.0, 100.0, 80.0)
            } else {
                LogicalRect::from_xywh(10.0, 20.0, 30.0, 20.0)
            })
        });
        (semantics, frame, root, button)
    }

    #[test]
    fn full_update_maps_identity_hierarchy_focus_and_physical_bounds() {
        let (semantics, frame, root, button) = fixture();
        let update = build_tree_update(
            &semantics,
            &frame,
            ScaleFactor::new(2.0).unwrap(),
            Some(button),
        )
        .unwrap();

        assert_eq!(update.tree.as_ref().unwrap().root, NodeId(root.get()));
        assert_eq!(update.focus, NodeId(button.get()));
        let (_, node) = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(button.get()))
            .unwrap();
        assert_eq!(node.role(), accesskit::Role::Button);
        assert_eq!(node.label(), Some("Save"));
        assert_eq!(
            node.bounds(),
            Some(accesskit::Rect {
                x0: 20.0,
                y0: 40.0,
                x1: 80.0,
                y1: 80.0,
            })
        );
        assert!(node.supports_action(Action::Click));
    }

    #[test]
    fn native_actions_translate_only_when_supported_by_core_snapshot() {
        let (semantics, _, _, button) = fixture();
        let click = ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: NodeId(button.get()),
            data: None,
        };
        let unsupported = ActionRequest {
            action: Action::Increment,
            ..click.clone()
        };

        assert_eq!(
            translate_action(&semantics, &click).unwrap().action,
            SemanticAction::Activate
        );
        assert_eq!(translate_action(&semantics, &unsupported), None);
    }

    #[test]
    fn native_set_value_payloads_translate_to_core_values() {
        let tree = WidgetTree::new(
            Semantics::new(SemanticRole::Slider).with_action(SemanticAction::SetValue),
        );
        let target = tree.root();
        let semantics = SemanticSnapshot::build(&tree, |_, semantics| semantics.clone());
        let request = ActionRequest {
            action: Action::SetValue,
            target_tree: TreeId::ROOT,
            target_node: NodeId(target.get()),
            data: Some(accesskit::ActionData::NumericValue(72.5)),
        };

        assert_eq!(
            translate_action(&semantics, &request).unwrap().value,
            Some(crate::SemanticActionValue::Number(72.5))
        );
    }

    #[test]
    fn combo_box_role_maps_to_the_native_accessibility_role() {
        assert_eq!(map_role(SemanticRole::ComboBox), accesskit::Role::ComboBox);
    }

    #[test]
    fn specialized_text_roles_map_to_native_accessibility_roles() {
        assert_eq!(
            map_role(SemanticRole::MultilineTextField),
            accesskit::Role::MultilineTextInput
        );
        assert_eq!(
            map_role(SemanticRole::SearchField),
            accesskit::Role::SearchInput
        );
        assert_eq!(
            map_role(SemanticRole::ListBoxOption),
            accesskit::Role::ListBoxOption
        );
        assert_eq!(map_role(SemanticRole::Alert), accesskit::Role::Alert);
        assert_eq!(
            map_role(SemanticRole::Progress),
            accesskit::Role::ProgressIndicator
        );
        assert_eq!(map_role(SemanticRole::Group), accesskit::Role::Group);
        assert_eq!(map_role(SemanticRole::Timer), accesskit::Role::Timer);
        assert_eq!(map_role(SemanticRole::Table), accesskit::Role::Table);
        assert_eq!(map_role(SemanticRole::Cell), accesskit::Role::Cell);
        assert_eq!(map_role(SemanticRole::TabList), accesskit::Role::TabList);
        assert_eq!(map_role(SemanticRole::Tab), accesskit::Role::Tab);
    }

    #[test]
    fn text_placeholder_maps_to_the_native_property() {
        let tree = WidgetTree::new(
            Semantics::new(SemanticRole::TextField)
                .with_name("Name")
                .with_placeholder("Enter name"),
        );
        let target = tree.root();
        let semantics = SemanticSnapshot::build(&tree, |_, semantics| semantics.clone());
        let frame = FrameSnapshot::build(&tree, |_, _| {
            WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 100.0, 20.0))
        });
        let update = build_tree_update(
            &semantics,
            &frame,
            ScaleFactor::new(1.0).unwrap(),
            Some(target),
        )
        .unwrap();

        assert_eq!(update.nodes[0].1.placeholder(), Some("Enter name"));
    }

    #[test]
    fn editable_text_run_and_selection_round_trip_through_native_actions() {
        let mut input = crate::TextInput::with_text("Name", "aمُ👩‍💻");
        input.edit.set_selection_from_anchor(5, 1);
        let tree = WidgetTree::new(input);
        let target = tree.root();
        let semantics = SemanticSnapshot::build(&tree, |_, input| input.semantics());
        let frame = FrameSnapshot::build(&tree, |_, _| {
            WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 160.0, 24.0))
        });
        let update = build_tree_update(
            &semantics,
            &frame,
            ScaleFactor::new(1.0).unwrap(),
            Some(target),
        )
        .unwrap();
        let input_node = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(target.get()))
            .unwrap();
        let run_id = input_node.1.children()[0];
        let run = update.nodes.iter().find(|(id, _)| *id == run_id).unwrap();
        assert_eq!(run.1.role(), Role::TextRun);
        assert_eq!(run.1.character_lengths(), &[1, 4, 11]);
        assert_eq!(
            input_node
                .1
                .text_selection()
                .unwrap()
                .anchor
                .character_index,
            2
        );
        assert_eq!(
            input_node.1.text_selection().unwrap().focus.character_index,
            1
        );

        let request = ActionRequest {
            action: Action::SetTextSelection,
            target_tree: TreeId::ROOT,
            target_node: NodeId(target.get()),
            data: Some(accesskit::ActionData::SetTextSelection(TextSelection {
                anchor: TextPosition {
                    node: run_id,
                    character_index: 1,
                },
                focus: TextPosition {
                    node: run_id,
                    character_index: 3,
                },
            })),
        };
        assert_eq!(
            translate_action(&semantics, &request).unwrap().value,
            Some(crate::SemanticActionValue::TextSelection {
                anchor: 1,
                caret: 16,
            })
        );
    }

    #[test]
    fn slider_numeric_metadata_maps_to_native_properties() {
        let tree = WidgetTree::new(crate::Slider::new("Amount", 0.0..=100.0, 40.0).unwrap());
        let target = tree.root();
        let semantics = SemanticSnapshot::build(&tree, |_, slider| slider.semantics());
        let frame = FrameSnapshot::build(&tree, |_, _| {
            WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 100.0, 20.0))
        });
        let update = build_tree_update(
            &semantics,
            &frame,
            ScaleFactor::new(1.0).unwrap(),
            Some(target),
        )
        .unwrap();
        let node = &update.nodes[0].1;

        assert_eq!(node.numeric_value(), Some(40.0));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(100.0));
        assert_eq!(node.numeric_value_step(), Some(1.0));
    }

    #[test]
    fn virtual_menu_items_map_and_translate_activation() {
        let menu = crate::Menu::new(
            "Actions",
            vec![
                crate::MenuItem::new("Open"),
                crate::MenuItem {
                    disabled: true,
                    ..crate::MenuItem::new("Delete")
                },
            ],
        )
        .unwrap();
        let tree = WidgetTree::new(menu);
        let target = tree.root();
        let mut semantics = SemanticSnapshot::build(&tree, |_, menu| menu.semantics());
        assert!(semantics.set_virtual_child_bounds(
            target,
            0,
            LogicalRect::from_xywh(10.0, 20.0, 80.0, 24.0),
        ));
        let frame = FrameSnapshot::build(&tree, |_, _| {
            WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 120.0, 60.0))
        });
        let update = build_tree_update(
            &semantics,
            &frame,
            ScaleFactor::new(1.0).unwrap(),
            Some(target),
        )
        .unwrap();
        let parent = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(target.get()))
            .unwrap();
        assert_eq!(parent.1.children().len(), 2);
        let first = parent.1.children()[0];
        let second = parent.1.children()[1];
        assert_eq!(parent.1.active_descendant(), Some(first));
        assert_eq!(
            update
                .nodes
                .iter()
                .find(|(id, _)| *id == first)
                .unwrap()
                .1
                .bounds(),
            Some(accesskit::Rect {
                x0: 10.0,
                y0: 20.0,
                x1: 90.0,
                y1: 44.0,
            })
        );
        assert_eq!(
            update
                .nodes
                .iter()
                .find(|(id, _)| *id == first)
                .unwrap()
                .1
                .role(),
            Role::MenuItem
        );

        let request = |target_node| ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node,
            data: None,
        };
        assert_eq!(
            translate_action(&semantics, &request(first)).unwrap().value,
            Some(crate::SemanticActionValue::Index(0))
        );
        assert_eq!(translate_action(&semantics, &request(second)), None);
    }
}
