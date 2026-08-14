// accessibility.rs

use std::collections::BTreeSet;

use crate::{WidgetId, WidgetTree};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticRole {
    #[default]
    Generic,
    Button,
    Checkbox,
    Radio,
    Switch,
    Slider,
    Text,
    TextField,
    Link,
    Image,
    Heading,
    List,
    ListItem,
    Menu,
    MenuItem,
    Dialog,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticAction {
    Focus,
    Blur,
    Activate,
    Increment,
    Decrement,
    SetValue,
    ShowMenu,
    ScrollIntoView,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SemanticState {
    pub disabled: bool,
    pub hidden: bool,
    pub focused: bool,
    pub checked: Option<bool>,
    pub selected: bool,
    pub expanded: Option<bool>,
    pub read_only: bool,
    pub required: bool,
    pub invalid: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Semantics {
    pub role: SemanticRole,
    pub name: Option<String>,
    pub value: Option<String>,
    pub state: SemanticState,
    actions: BTreeSet<SemanticAction>,
}

impl Semantics {
    pub fn new(role: SemanticRole) -> Self {
        Self {
            role,
            ..Self::default()
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.set_name(name);
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_state(mut self, state: SemanticState) -> Self {
        self.state = state;
        self
    }

    pub fn with_action(mut self, action: SemanticAction) -> Self {
        self.actions.insert(action);
        self
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.name = (!name.trim().is_empty()).then_some(name);
    }

    pub fn set_value(&mut self, value: Option<String>) {
        self.value = value;
    }

    pub fn add_action(&mut self, action: SemanticAction) -> bool {
        self.actions.insert(action)
    }

    pub fn remove_action(&mut self, action: SemanticAction) -> bool {
        self.actions.remove(&action)
    }

    pub fn supports(&self, action: SemanticAction) -> bool {
        self.actions.contains(&action)
    }

    pub fn actions(&self) -> impl ExactSizeIterator<Item = SemanticAction> + '_ {
        self.actions.iter().copied()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticNode {
    pub id: WidgetId,
    pub parent: Option<WidgetId>,
    pub children: Vec<WidgetId>,
    pub semantics: Semantics,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticActionRequest {
    pub target: WidgetId,
    pub action: SemanticAction,
}

#[derive(Clone, Debug, Default)]
pub struct SemanticSnapshot {
    root: Option<WidgetId>,
    nodes: std::collections::HashMap<WidgetId, SemanticNode>,
    order: Vec<WidgetId>,
}

impl SemanticSnapshot {
    pub fn build<State>(
        tree: &WidgetTree<State>,
        mut semantics: impl FnMut(WidgetId, &State) -> Semantics,
    ) -> Self {
        let mut snapshot = Self {
            root: Some(tree.root()),
            nodes: std::collections::HashMap::with_capacity(tree.len()),
            order: Vec::with_capacity(tree.len()),
        };
        let mut stack = vec![(tree.root(), None, false)];

        while let Some((id, visible_parent, ancestor_hidden)) = stack.pop() {
            let tree_node = tree.get(id).unwrap();
            let mut properties = semantics(id, &tree_node.state);
            properties.state.hidden |= ancestor_hidden;
            if properties.state.hidden {
                if id == tree.root() {
                    snapshot.root = None;
                }
                continue;
            }
            snapshot.nodes.insert(
                id,
                SemanticNode {
                    id,
                    parent: visible_parent,
                    children: Vec::new(),
                    semantics: properties,
                },
            );
            if let Some(parent) = visible_parent {
                snapshot.nodes.get_mut(&parent).unwrap().children.push(id);
            }
            snapshot.order.push(id);
            stack.extend(
                tree_node
                    .children()
                    .iter()
                    .rev()
                    .map(|child| (*child, Some(id), false)),
            );
        }
        snapshot
    }

    pub fn root(&self) -> Option<WidgetId> {
        self.root
    }

    pub fn get(&self, widget: WidgetId) -> Option<&SemanticNode> {
        self.nodes.get(&widget)
    }

    pub fn order(&self) -> &[WidgetId] {
        &self.order
    }

    pub fn request_action(
        &self,
        target: WidgetId,
        action: SemanticAction,
    ) -> Option<SemanticActionRequest> {
        let semantics = &self.get(target)?.semantics;
        let mutable_value_action = matches!(
            action,
            SemanticAction::Increment | SemanticAction::Decrement | SemanticAction::SetValue
        );
        (semantics.supports(action)
            && !semantics.state.disabled
            && !(semantics.state.read_only && mutable_value_action))
            .then_some(SemanticActionRequest { target, action })
    }
}

#[cfg(test)]
mod tests {
    use super::{SemanticAction, SemanticRole, SemanticSnapshot, SemanticState, Semantics};
    use crate::WidgetTree;

    #[test]
    fn semantics_preserve_role_name_value_and_state() {
        let semantics = Semantics::new(SemanticRole::Checkbox)
            .with_name("Receive updates")
            .with_value("weekly")
            .with_state(SemanticState {
                checked: Some(true),
                required: true,
                ..SemanticState::default()
            });

        assert_eq!(semantics.role, SemanticRole::Checkbox);
        assert_eq!(semantics.name.as_deref(), Some("Receive updates"));
        assert_eq!(semantics.value.as_deref(), Some("weekly"));
        assert_eq!(semantics.state.checked, Some(true));
        assert!(semantics.state.required);
    }

    #[test]
    fn actions_are_deduplicated_and_have_stable_order() {
        let semantics = Semantics::new(SemanticRole::Slider)
            .with_action(SemanticAction::SetValue)
            .with_action(SemanticAction::Increment)
            .with_action(SemanticAction::Decrement)
            .with_action(SemanticAction::Increment);

        assert_eq!(
            semantics.actions().collect::<Vec<_>>(),
            [
                SemanticAction::Increment,
                SemanticAction::Decrement,
                SemanticAction::SetValue,
            ]
        );
    }

    #[test]
    fn blank_accessible_names_are_treated_as_absent() {
        let mut semantics = Semantics::new(SemanticRole::Button).with_name("   ");
        assert_eq!(semantics.name, None);

        semantics.set_name("Save");
        assert_eq!(semantics.name.as_deref(), Some("Save"));
    }

    #[test]
    fn supported_actions_can_change_with_widget_state() {
        let mut semantics = Semantics::new(SemanticRole::Button);
        assert!(semantics.add_action(SemanticAction::Activate));
        assert!(!semantics.add_action(SemanticAction::Activate));
        assert!(semantics.supports(SemanticAction::Activate));
        assert!(semantics.remove_action(SemanticAction::Activate));
        assert!(!semantics.supports(SemanticAction::Activate));
    }

    #[test]
    fn semantic_snapshot_freezes_tree_order_and_hierarchy() {
        let mut tree = WidgetTree::new(Semantics::new(SemanticRole::Generic));
        let root = tree.root();
        let first = tree
            .append(
                root,
                Semantics::new(SemanticRole::Button).with_name("First"),
            )
            .unwrap();
        let second = tree
            .append(
                root,
                Semantics::new(SemanticRole::Button).with_name("Second"),
            )
            .unwrap();
        let nested = tree
            .append(first, Semantics::new(SemanticRole::Text).with_name("Label"))
            .unwrap();
        let snapshot = SemanticSnapshot::build(&tree, |_, semantics| semantics.clone());

        assert_eq!(snapshot.root(), Some(root));
        assert_eq!(snapshot.order(), &[root, first, nested, second]);
        assert_eq!(snapshot.get(root).unwrap().children, [first, second]);
        assert_eq!(snapshot.get(first).unwrap().children, [nested]);
        assert_eq!(snapshot.get(nested).unwrap().parent, Some(first));
    }

    #[test]
    fn hidden_semantic_subtrees_are_excluded() {
        let mut tree = WidgetTree::new(Semantics::new(SemanticRole::Generic));
        let root = tree.root();
        let hidden = tree
            .append(
                root,
                Semantics::new(SemanticRole::Generic).with_state(SemanticState {
                    hidden: true,
                    ..SemanticState::default()
                }),
            )
            .unwrap();
        let descendant = tree
            .append(hidden, Semantics::new(SemanticRole::Button))
            .unwrap();
        let visible = tree
            .append(root, Semantics::new(SemanticRole::Button))
            .unwrap();
        let snapshot = SemanticSnapshot::build(&tree, |_, semantics| semantics.clone());

        assert_eq!(snapshot.order(), &[root, visible]);
        assert_eq!(snapshot.get(root).unwrap().children, [visible]);
        assert!(snapshot.get(hidden).is_none());
        assert!(snapshot.get(descendant).is_none());
    }

    #[test]
    fn action_requests_validate_capability_and_effective_state() {
        let mut tree = WidgetTree::new(Semantics::new(SemanticRole::Generic));
        let root = tree.root();
        let button = tree
            .append(
                root,
                Semantics::new(SemanticRole::Button).with_action(SemanticAction::Activate),
            )
            .unwrap();
        let read_only_slider = tree
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
            snapshot.request_action(button, SemanticAction::Activate),
            Some(super::SemanticActionRequest {
                target: button,
                action: SemanticAction::Activate,
            })
        );
        assert_eq!(
            snapshot.request_action(button, SemanticAction::SetValue),
            None
        );
        assert_eq!(
            snapshot.request_action(read_only_slider, SemanticAction::Increment),
            None
        );
    }
}
