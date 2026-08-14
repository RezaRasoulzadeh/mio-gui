// widget_tree.rs

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WidgetId(u64);

impl WidgetId {
    pub const fn get(self) -> u64 {
        self.0
    }

    #[cfg(test)]
    pub(crate) const fn from_test_value(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WidgetNode<State> {
    pub state: State,
    parent: Option<WidgetId>,
    children: Vec<WidgetId>,
}

impl<State> WidgetNode<State> {
    pub fn parent(&self) -> Option<WidgetId> {
        self.parent
    }

    pub fn children(&self) -> &[WidgetId] {
        &self.children
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WidgetTreeError {
    MissingWidget(WidgetId),
    CannotMoveRoot,
    Cycle,
    IndexOutOfBounds,
    IdExhausted,
}

impl Display for WidgetTreeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingWidget(id) => write!(formatter, "widget {} does not exist", id.get()),
            Self::CannotMoveRoot => formatter.write_str("the root widget cannot be moved"),
            Self::Cycle => formatter.write_str("a widget cannot become its own ancestor"),
            Self::IndexOutOfBounds => formatter.write_str("child index is out of bounds"),
            Self::IdExhausted => formatter.write_str("widget identity space is exhausted"),
        }
    }
}

impl Error for WidgetTreeError {}

#[derive(Clone, Debug)]
pub struct WidgetTree<State> {
    root: WidgetId,
    next_id: u64,
    nodes: HashMap<WidgetId, WidgetNode<State>>,
}

impl<State> WidgetTree<State> {
    pub fn new(root_state: State) -> Self {
        let root = WidgetId(1);
        let mut nodes = HashMap::new();
        nodes.insert(
            root,
            WidgetNode {
                state: root_state,
                parent: None,
                children: Vec::new(),
            },
        );
        Self {
            root,
            next_id: 2,
            nodes,
        }
    }

    pub fn root(&self) -> WidgetId {
        self.root
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn contains(&self, id: WidgetId) -> bool {
        self.nodes.contains_key(&id)
    }

    pub fn get(&self, id: WidgetId) -> Option<&WidgetNode<State>> {
        self.nodes.get(&id)
    }

    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut WidgetNode<State>> {
        self.nodes.get_mut(&id)
    }

    pub fn append(&mut self, parent: WidgetId, state: State) -> Result<WidgetId, WidgetTreeError> {
        let index = self
            .nodes
            .get(&parent)
            .ok_or(WidgetTreeError::MissingWidget(parent))?
            .children
            .len();
        self.insert(parent, index, state)
    }

    pub fn insert(
        &mut self,
        parent: WidgetId,
        index: usize,
        state: State,
    ) -> Result<WidgetId, WidgetTreeError> {
        let child_count = self
            .nodes
            .get(&parent)
            .ok_or(WidgetTreeError::MissingWidget(parent))?
            .children
            .len();
        if index > child_count {
            return Err(WidgetTreeError::IndexOutOfBounds);
        }
        let id = WidgetId(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(WidgetTreeError::IdExhausted)?;
        self.nodes.insert(
            id,
            WidgetNode {
                state,
                parent: Some(parent),
                children: Vec::new(),
            },
        );
        self.nodes
            .get_mut(&parent)
            .unwrap()
            .children
            .insert(index, id);
        Ok(id)
    }

    pub fn reparent(
        &mut self,
        id: WidgetId,
        new_parent: WidgetId,
        index: usize,
    ) -> Result<(), WidgetTreeError> {
        if id == self.root {
            return Err(WidgetTreeError::CannotMoveRoot);
        }
        let old_parent = self
            .nodes
            .get(&id)
            .ok_or(WidgetTreeError::MissingWidget(id))?
            .parent
            .unwrap();
        let new_child_count = self
            .nodes
            .get(&new_parent)
            .ok_or(WidgetTreeError::MissingWidget(new_parent))?
            .children
            .len();
        if index > new_child_count {
            return Err(WidgetTreeError::IndexOutOfBounds);
        }
        if id == new_parent || self.ancestors(new_parent).any(|ancestor| ancestor == id) {
            return Err(WidgetTreeError::Cycle);
        }

        let old_index = self.nodes[&old_parent]
            .children
            .iter()
            .position(|child| *child == id)
            .unwrap();
        self.nodes
            .get_mut(&old_parent)
            .unwrap()
            .children
            .remove(old_index);
        let adjusted_index = if old_parent == new_parent && old_index < index {
            index - 1
        } else {
            index
        };
        self.nodes
            .get_mut(&new_parent)
            .unwrap()
            .children
            .insert(adjusted_index, id);
        self.nodes.get_mut(&id).unwrap().parent = Some(new_parent);
        Ok(())
    }

    pub fn remove_subtree(&mut self, id: WidgetId) -> Result<Vec<State>, WidgetTreeError> {
        if id == self.root {
            return Err(WidgetTreeError::CannotMoveRoot);
        }
        let parent = self
            .nodes
            .get(&id)
            .ok_or(WidgetTreeError::MissingWidget(id))?
            .parent
            .unwrap();
        self.nodes
            .get_mut(&parent)
            .unwrap()
            .children
            .retain(|child| *child != id);
        let ids = self.depth_first(id).collect::<Vec<_>>();
        Ok(ids
            .into_iter()
            .rev()
            .map(|removed| self.nodes.remove(&removed).unwrap().state)
            .collect())
    }

    pub fn ancestors(&self, id: WidgetId) -> Ancestors<'_, State> {
        Ancestors {
            tree: self,
            next: self.nodes.get(&id).and_then(|node| node.parent),
        }
    }

    pub fn depth_first(&self, root: WidgetId) -> DepthFirst<'_, State> {
        DepthFirst {
            tree: self,
            stack: self.contains(root).then_some(root).into_iter().collect(),
        }
    }
}

pub struct Ancestors<'a, State> {
    tree: &'a WidgetTree<State>,
    next: Option<WidgetId>,
}

impl<State> Iterator for Ancestors<'_, State> {
    type Item = WidgetId;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.next?;
        self.next = self.tree.nodes[&id].parent;
        Some(id)
    }
}

pub struct DepthFirst<'a, State> {
    tree: &'a WidgetTree<State>,
    stack: Vec<WidgetId>,
}

impl<State> Iterator for DepthFirst<'_, State> {
    type Item = WidgetId;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.stack.pop()?;
        self.stack
            .extend(self.tree.nodes[&id].children.iter().rev());
        Some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::{WidgetTree, WidgetTreeError};

    #[test]
    fn insertion_preserves_deterministic_semantic_order() {
        let mut tree = WidgetTree::new("root");
        let root = tree.root();
        let first = tree.append(root, "first").unwrap();
        let third = tree.append(root, "third").unwrap();
        let second = tree.insert(root, 1, "second").unwrap();

        assert_eq!(tree.get(root).unwrap().children(), &[first, second, third]);
        assert_eq!(
            tree.depth_first(root)
                .map(|id| tree.get(id).unwrap().state)
                .collect::<Vec<_>>(),
            ["root", "first", "second", "third"]
        );
    }

    #[test]
    fn reparenting_preserves_identity_and_rejects_cycles() {
        let mut tree = WidgetTree::new("root");
        let root = tree.root();
        let parent = tree.append(root, "parent").unwrap();
        let child = tree.append(parent, "child").unwrap();
        let sibling = tree.append(root, "sibling").unwrap();

        tree.reparent(child, root, 1).unwrap();
        assert_eq!(tree.get(child).unwrap().parent(), Some(root));
        assert_eq!(
            tree.get(root).unwrap().children(),
            &[parent, child, sibling]
        );
        assert_eq!(
            tree.reparent(parent, parent, 0),
            Err(WidgetTreeError::Cycle)
        );
        assert_eq!(
            tree.reparent(root, child, 0),
            Err(WidgetTreeError::CannotMoveRoot)
        );
    }

    #[test]
    fn removing_subtree_invalidates_all_descendant_ids() {
        let mut tree = WidgetTree::new("root");
        let root = tree.root();
        let parent = tree.append(root, "parent").unwrap();
        let child = tree.append(parent, "child").unwrap();
        let grandchild = tree.append(child, "grandchild").unwrap();

        assert_eq!(
            tree.remove_subtree(parent).unwrap(),
            ["grandchild", "child", "parent"]
        );
        assert_eq!(tree.len(), 1);
        assert!(!tree.contains(parent));
        assert!(!tree.contains(child));
        assert!(!tree.contains(grandchild));
        assert_eq!(
            tree.append(parent, "stale"),
            Err(WidgetTreeError::MissingWidget(parent))
        );
    }

    #[test]
    fn ids_are_monotonic_and_never_reused() {
        let mut tree = WidgetTree::new(());
        let root = tree.root();
        let removed = tree.append(root, ()).unwrap();
        tree.remove_subtree(removed).unwrap();
        let replacement = tree.append(root, ()).unwrap();

        assert!(replacement.get() > removed.get());
        assert!(!tree.is_empty());
    }
}
