// accesskit_adapter.rs

use accesskit::{
    Action, ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, Invalid, Node,
    NodeId, Rect, Role, Toggled, Tree, TreeId, TreeUpdate,
};

use crate::{
    FrameSnapshot, ScaleFactor, SemanticAction, SemanticActionRequest, SemanticRole,
    SemanticSnapshot, WidgetId,
};

fn build_tree_update(
    semantics: &SemanticSnapshot,
    frame: &FrameSnapshot,
    scale_factor: ScaleFactor,
    focused: Option<WidgetId>,
) -> Option<TreeUpdate> {
    let root = semantics.root()?;
    let nodes = semantics
        .order()
        .iter()
        .filter_map(|id| semantics.get(*id).map(|node| (*id, node)))
        .map(|(id, semantic_node)| {
            let mut node = Node::new(map_role(semantic_node.semantics.role));
            node.set_children(
                semantic_node
                    .children
                    .iter()
                    .map(|child| NodeId(child.get()))
                    .collect::<Vec<_>>(),
            );
            if let Some(name) = &semantic_node.semantics.name {
                node.set_label(name.clone());
            }
            if let Some(value) = &semantic_node.semantics.value {
                node.set_value(value.clone());
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
            if let Some(frame_node) = frame.get(id) {
                let bounds = frame_node.bounds.to_physical(scale_factor);
                node.set_bounds(Rect {
                    x0: f64::from(bounds.min_x()),
                    y0: f64::from(bounds.min_y()),
                    x1: f64::from(bounds.max_x()),
                    y1: f64::from(bounds.max_y()),
                });
            }
            (NodeId(id.get()), node)
        })
        .collect();
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

fn translate_action(
    semantics: &SemanticSnapshot,
    request: &ActionRequest,
) -> Option<SemanticActionRequest> {
    let target = semantics
        .order()
        .iter()
        .copied()
        .find(|widget| widget.get() == request.target_node.0)?;
    semantics.request_action(target, unmap_action(request.action)?)
}

fn map_role(role: SemanticRole) -> Role {
    match role {
        SemanticRole::Generic => Role::GenericContainer,
        SemanticRole::Button => Role::Button,
        SemanticRole::Checkbox => Role::CheckBox,
        SemanticRole::Radio => Role::RadioButton,
        SemanticRole::Switch => Role::Switch,
        SemanticRole::Slider => Role::Slider,
        SemanticRole::Text => Role::Label,
        SemanticRole::TextField => Role::TextInput,
        SemanticRole::Link => Role::Link,
        SemanticRole::Image => Role::Image,
        SemanticRole::Heading => Role::Heading,
        SemanticRole::List => Role::List,
        SemanticRole::ListItem => Role::ListItem,
        SemanticRole::Menu => Role::Menu,
        SemanticRole::MenuItem => Role::MenuItem,
        SemanticRole::Dialog => Role::Dialog,
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
    use super::{build_tree_update, translate_action};
    use crate::{
        FrameSnapshot, LogicalRect, ScaleFactor, SemanticAction, SemanticRole, SemanticSnapshot,
        Semantics, WidgetGeometry, WidgetTree,
    };
    use accesskit::{Action, ActionRequest, NodeId, TreeId};

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
}
