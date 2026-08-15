// accessibility.rs

use std::collections::BTreeSet;
use std::ops::RangeInclusive;

use crate::{LogicalRect, WidgetId, WidgetTree};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticRole {
    #[default]
    Generic,
    Button,
    Checkbox,
    Radio,
    Switch,
    Slider,
    ComboBox,
    Text,
    TextField,
    MultilineTextField,
    SearchField,
    Link,
    Image,
    Heading,
    List,
    ListItem,
    Menu,
    MenuItem,
    ListBoxOption,
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
    SetTextSelection,
    ShowMenu,
    ScrollIntoView,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SemanticActionValue {
    Text(String),
    Number(f64),
    TextSelection { anchor: usize, caret: usize },
    Index(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticEditableText {
    text: String,
    character_lengths: Vec<u8>,
    anchor: usize,
    caret: usize,
}

impl SemanticEditableText {
    pub fn new(text: impl Into<String>, anchor: usize, caret: usize) -> Option<Self> {
        let text = text.into();
        let character_lengths = text
            .graphemes(true)
            .map(|grapheme| u8::try_from(grapheme.len()))
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        let is_boundary = |offset| {
            offset <= text.len()
                && (offset == text.len()
                    || text
                        .grapheme_indices(true)
                        .any(|(boundary, _)| boundary == offset))
        };
        (is_boundary(anchor) && is_boundary(caret)).then_some(Self {
            text,
            character_lengths,
            anchor,
            caret,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn character_lengths(&self) -> &[u8] {
        &self.character_lengths
    }

    pub fn anchor(&self) -> usize {
        self.anchor
    }

    pub fn caret(&self) -> usize {
        self.caret
    }

    pub fn character_index(&self, byte_offset: usize) -> Option<usize> {
        let mut boundary = 0;
        for (index, length) in self.character_lengths.iter().enumerate() {
            if boundary == byte_offset {
                return Some(index);
            }
            boundary += usize::from(*length);
        }
        (boundary == byte_offset).then_some(self.character_lengths.len())
    }

    pub fn byte_offset(&self, character_index: usize) -> Option<usize> {
        (character_index <= self.character_lengths.len()).then(|| {
            self.character_lengths[..character_index]
                .iter()
                .map(|length| usize::from(*length))
                .sum()
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SemanticNumericValue {
    value: f64,
    minimum: f64,
    maximum: f64,
    step: Option<f64>,
}

impl SemanticNumericValue {
    pub fn new(value: f64, minimum: f64, maximum: f64, step: Option<f64>) -> Option<Self> {
        (value.is_finite()
            && minimum.is_finite()
            && maximum.is_finite()
            && minimum <= maximum
            && (minimum..=maximum).contains(&value)
            && step.is_none_or(|step| step.is_finite() && step > 0.0))
        .then_some(Self {
            value,
            minimum,
            maximum,
            step,
        })
    }

    pub fn value(self) -> f64 {
        self.value
    }

    pub fn range(self) -> RangeInclusive<f64> {
        self.minimum..=self.maximum
    }

    pub fn step(self) -> Option<f64> {
        self.step
    }
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Semantics {
    pub role: SemanticRole,
    pub name: Option<String>,
    pub value: Option<String>,
    pub placeholder: Option<String>,
    pub numeric_value: Option<SemanticNumericValue>,
    pub editable_text: Option<SemanticEditableText>,
    pub state: SemanticState,
    actions: BTreeSet<SemanticAction>,
    virtual_children: Vec<Semantics>,
    virtual_child_bounds: Vec<Option<LogicalRect>>,
    active_virtual_child: Option<usize>,
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

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.set_placeholder(placeholder);
        self
    }

    pub fn with_numeric_value(mut self, value: SemanticNumericValue) -> Self {
        self.numeric_value = Some(value);
        self
    }

    pub fn with_editable_text(mut self, text: SemanticEditableText) -> Self {
        self.editable_text = Some(text);
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

    pub fn with_virtual_child(mut self, child: Semantics) -> Self {
        self.virtual_children.push(child);
        self.virtual_child_bounds.push(None);
        self
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.name = (!name.trim().is_empty()).then_some(name);
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        let placeholder = placeholder.into();
        self.placeholder = (!placeholder.trim().is_empty()).then_some(placeholder);
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

    pub fn virtual_children(&self) -> &[Semantics] {
        &self.virtual_children
    }

    pub fn set_active_virtual_child(&mut self, index: usize) -> bool {
        if index >= self.virtual_children.len() {
            return false;
        }
        self.active_virtual_child = Some(index);
        true
    }

    pub fn active_virtual_child(&self) -> Option<usize> {
        self.active_virtual_child
    }

    pub fn virtual_child_bounds(&self, index: usize) -> Option<LogicalRect> {
        self.virtual_child_bounds.get(index).copied().flatten()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticNode {
    pub id: WidgetId,
    pub parent: Option<WidgetId>,
    pub children: Vec<WidgetId>,
    pub semantics: Semantics,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticActionRequest {
    pub target: WidgetId,
    pub action: SemanticAction,
    pub value: Option<SemanticActionValue>,
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

    pub(crate) fn set_virtual_child_bounds(
        &mut self,
        target: WidgetId,
        child_index: usize,
        bounds: LogicalRect,
    ) -> bool {
        let Some(bounds_slot) = self
            .nodes
            .get_mut(&target)
            .and_then(|node| node.semantics.virtual_child_bounds.get_mut(child_index))
        else {
            return false;
        };
        *bounds_slot = Some(bounds);
        true
    }

    pub fn request_action(
        &self,
        target: WidgetId,
        action: SemanticAction,
    ) -> Option<SemanticActionRequest> {
        self.request_action_with_value(target, action, None)
    }

    pub fn request_action_with_value(
        &self,
        target: WidgetId,
        action: SemanticAction,
        value: Option<SemanticActionValue>,
    ) -> Option<SemanticActionRequest> {
        let semantics = &self.get(target)?.semantics;
        let mutable_value_action = matches!(
            action,
            SemanticAction::Increment
                | SemanticAction::Decrement
                | SemanticAction::SetValue
                | SemanticAction::SetTextSelection
        );
        let valid_value = match action {
            SemanticAction::SetValue => matches!(
                value,
                Some(SemanticActionValue::Text(_) | SemanticActionValue::Number(_))
            ),
            SemanticAction::SetTextSelection => {
                matches!(value, Some(SemanticActionValue::TextSelection { .. }))
            }
            _ => value.is_none(),
        };
        (semantics.supports(action)
            && !semantics.state.disabled
            && !(semantics.state.read_only && mutable_value_action)
            && valid_value)
            .then_some(SemanticActionRequest {
                target,
                action,
                value,
            })
    }

    pub fn request_virtual_child_action(
        &self,
        target: WidgetId,
        child_index: usize,
        action: SemanticAction,
    ) -> Option<SemanticActionRequest> {
        let parent = &self.get(target)?.semantics;
        let child = parent.virtual_children.get(child_index)?;
        (child.supports(action)
            && !parent.state.disabled
            && !parent.state.hidden
            && !child.state.disabled
            && !child.state.hidden)
            .then_some(SemanticActionRequest {
                target,
                action,
                value: Some(SemanticActionValue::Index(child_index)),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SemanticAction, SemanticActionValue, SemanticRole, SemanticSnapshot, SemanticState,
        Semantics,
    };
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
    fn numeric_semantics_validate_range_value_and_step() {
        let numeric = super::SemanticNumericValue::new(40.0, 0.0, 100.0, Some(5.0)).unwrap();
        assert_eq!(numeric.value(), 40.0);
        assert_eq!(numeric.range(), 0.0..=100.0);
        assert_eq!(numeric.step(), Some(5.0));
        assert!(super::SemanticNumericValue::new(f64::NAN, 0.0, 1.0, None).is_none());
        assert!(super::SemanticNumericValue::new(2.0, 0.0, 1.0, None).is_none());
        assert!(super::SemanticNumericValue::new(0.5, 1.0, 0.0, None).is_none());
        assert!(super::SemanticNumericValue::new(0.5, 0.0, 1.0, Some(0.0)).is_none());
    }

    #[test]
    fn editable_text_maps_grapheme_boundaries_to_character_indices() {
        let text = "aمُ👩‍💻";
        let editable = super::SemanticEditableText::new(text, 1, 5).unwrap();
        assert_eq!(editable.character_lengths(), &[1, 4, 11]);
        assert_eq!(editable.character_index(1), Some(1));
        assert_eq!(editable.character_index(5), Some(2));
        assert_eq!(editable.byte_offset(3), Some(text.len()));
        assert!(super::SemanticEditableText::new(text, 2, 5).is_none());
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
    fn blank_placeholders_are_treated_as_absent() {
        let mut semantics = Semantics::new(SemanticRole::TextField).with_placeholder("   ");
        assert_eq!(semantics.placeholder, None);

        semantics.set_placeholder("Enter name");
        assert_eq!(semantics.placeholder.as_deref(), Some("Enter name"));
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
    fn active_virtual_child_must_reference_an_existing_child() {
        let mut semantics = Semantics::new(SemanticRole::Menu)
            .with_virtual_child(Semantics::new(SemanticRole::MenuItem));
        assert!(!semantics.set_active_virtual_child(1));
        assert_eq!(semantics.active_virtual_child(), None);
        assert!(semantics.set_active_virtual_child(0));
        assert_eq!(semantics.active_virtual_child(), Some(0));
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
                value: None,
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

    #[test]
    fn value_action_requests_preserve_backend_neutral_payloads() {
        let tree = WidgetTree::new(
            Semantics::new(SemanticRole::TextField).with_action(SemanticAction::SetValue),
        );
        let target = tree.root();
        let snapshot = SemanticSnapshot::build(&tree, |_, semantics| semantics.clone());

        assert_eq!(
            snapshot.request_action_with_value(
                target,
                SemanticAction::SetValue,
                Some(SemanticActionValue::Text("سلام Mio".into())),
            ),
            Some(super::SemanticActionRequest {
                target,
                action: SemanticAction::SetValue,
                value: Some(SemanticActionValue::Text("سلام Mio".into())),
            })
        );
    }
}
