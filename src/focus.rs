// focus.rs

use std::collections::HashMap;

use crate::{ClipRegion, Direction, FrameSnapshot, LogicalRect, WidgetId, WidgetTree};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusIndicatorStyle {
    pub color: [f32; 4],
    pub width: f32,
    pub offset: f32,
    pub radius: f32,
}

impl Default for FocusIndicatorStyle {
    fn default() -> Self {
        Self {
            color: [0.15, 0.45, 1.0, 1.0],
            width: 2.0,
            offset: 2.0,
            radius: 6.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusIndicator {
    pub bounds: LogicalRect,
    pub clip: ClipRegion<crate::Logical>,
    pub color: [f32; 4],
    pub width: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FocusPolicy {
    pub focusable: bool,
    pub skip_tab_order: bool,
    pub disabled: bool,
    pub hidden: bool,
    pub inert: bool,
}

impl FocusPolicy {
    pub const fn focusable() -> Self {
        Self {
            focusable: true,
            skip_tab_order: false,
            disabled: false,
            hidden: false,
            inert: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectiveFocusPolicy {
    pub focusable: bool,
    pub skip_tab_order: bool,
    pub disabled: bool,
    pub hidden: bool,
    pub inert: bool,
}

impl EffectiveFocusPolicy {
    pub const fn accepts_focus(self) -> bool {
        self.focusable && !self.disabled && !self.hidden && !self.inert
    }
}

#[derive(Clone, Debug)]
pub struct FocusSnapshot {
    policies: HashMap<WidgetId, EffectiveFocusPolicy>,
    semantic_order: Vec<WidgetId>,
    tab_order: Vec<WidgetId>,
}

impl FocusSnapshot {
    pub fn build<State>(
        tree: &WidgetTree<State>,
        mut policy: impl FnMut(WidgetId, &State) -> FocusPolicy,
    ) -> Self {
        let mut policies = HashMap::with_capacity(tree.len());
        let mut semantic_order = Vec::with_capacity(tree.len());
        let root = tree.root();
        let mut stack = vec![(root, false, false, false)];

        while let Some((id, ancestor_disabled, ancestor_hidden, ancestor_inert)) = stack.pop() {
            let node = tree.get(id).unwrap();
            let local = policy(id, &node.state);
            let effective = EffectiveFocusPolicy {
                focusable: local.focusable,
                skip_tab_order: local.skip_tab_order,
                disabled: ancestor_disabled || local.disabled,
                hidden: ancestor_hidden || local.hidden,
                inert: ancestor_inert || local.inert,
            };
            policies.insert(id, effective);
            semantic_order.push(id);
            stack.extend(node.children().iter().rev().map(|child| {
                (
                    *child,
                    effective.disabled,
                    effective.hidden,
                    effective.inert,
                )
            }));
        }

        let tab_order = semantic_order
            .iter()
            .copied()
            .filter(|widget| policies[widget].accepts_focus() && !policies[widget].skip_tab_order)
            .collect();
        Self {
            policies,
            semantic_order,
            tab_order,
        }
    }

    pub fn policy(&self, widget: WidgetId) -> Option<EffectiveFocusPolicy> {
        self.policies.get(&widget).copied()
    }

    pub fn accepts_focus(&self, widget: WidgetId) -> bool {
        self.policy(widget)
            .is_some_and(EffectiveFocusPolicy::accepts_focus)
    }

    pub fn semantic_order(&self) -> &[WidgetId] {
        &self.semantic_order
    }

    pub fn tab_order(&self) -> &[WidgetId] {
        &self.tab_order
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FocusTraversal {
    #[default]
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrowKey {
    Left,
    Right,
    Up,
    Down,
}

impl ArrowKey {
    pub const fn traversal(self, direction: Direction) -> FocusTraversal {
        match (self, direction) {
            (Self::Left, Direction::Ltr) | (Self::Right, Direction::Rtl) | (Self::Up, _) => {
                FocusTraversal::Backward
            }
            (Self::Right, Direction::Ltr) | (Self::Left, Direction::Rtl) | (Self::Down, _) => {
                FocusTraversal::Forward
            }
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FocusManager {
    focused: Option<WidgetId>,
    known_order: Vec<WidgetId>,
}

impl FocusManager {
    pub fn focused(&self) -> Option<WidgetId> {
        self.focused
    }

    pub fn focus(&mut self, snapshot: &FocusSnapshot, widget: WidgetId) -> bool {
        if !snapshot.accepts_focus(widget) {
            return false;
        }
        self.focused = Some(widget);
        self.known_order = snapshot.tab_order().to_vec();
        true
    }

    pub fn clear(&mut self) -> Option<WidgetId> {
        self.known_order.clear();
        self.focused.take()
    }

    pub fn reconcile(&mut self, snapshot: &FocusSnapshot) -> Option<WidgetId> {
        let removed = self
            .focused
            .filter(|widget| !snapshot.accepts_focus(*widget));
        if removed.is_some() {
            self.focused = None;
        }
        self.known_order = snapshot.tab_order().to_vec();
        removed
    }

    pub fn restore_after_rebuild(&mut self, snapshot: &FocusSnapshot) -> Option<WidgetId> {
        let Some(previous) = self.focused else {
            self.known_order = snapshot.tab_order().to_vec();
            return None;
        };
        if snapshot.accepts_focus(previous) {
            self.known_order = snapshot.tab_order().to_vec();
            return Some(previous);
        }

        let previous_index = self
            .known_order
            .iter()
            .position(|widget| *widget == previous);
        let restored = previous_index
            .and_then(|index| {
                self.known_order[index + 1..]
                    .iter()
                    .chain(self.known_order[..index].iter().rev())
                    .copied()
                    .find(|widget| snapshot.accepts_focus(*widget))
            })
            .or_else(|| snapshot.tab_order().first().copied());
        self.focused = restored;
        self.known_order = snapshot.tab_order().to_vec();
        restored
    }

    pub fn traverse(
        &mut self,
        snapshot: &FocusSnapshot,
        traversal: FocusTraversal,
    ) -> Option<WidgetId> {
        let order = snapshot.tab_order();
        if order.is_empty() {
            self.focused = None;
            return None;
        }
        let current = self
            .focused
            .and_then(|focused| order.iter().position(|widget| *widget == focused));
        let index = match (traversal, current) {
            (FocusTraversal::Forward, Some(index)) => (index + 1) % order.len(),
            (FocusTraversal::Backward, Some(0)) => order.len() - 1,
            (FocusTraversal::Backward, Some(index)) => index - 1,
            (FocusTraversal::Forward, None) => 0,
            (FocusTraversal::Backward, None) => order.len() - 1,
        };
        let focused = order[index];
        self.focused = Some(focused);
        self.known_order = order.to_vec();
        Some(focused)
    }

    pub fn navigate_arrow(
        &mut self,
        snapshot: &FocusSnapshot,
        key: ArrowKey,
        direction: Direction,
    ) -> Option<WidgetId> {
        self.traverse(snapshot, key.traversal(direction))
    }

    pub fn indicator(
        &self,
        frame: &FrameSnapshot,
        style: FocusIndicatorStyle,
    ) -> Option<FocusIndicator> {
        let node = frame.get(self.focused?)?;
        let width = finite_non_negative(style.width);
        let offset = finite_non_negative(style.offset);
        let expansion = width + offset;
        let bounds = LogicalRect::from_xywh(
            node.bounds.origin.x - expansion,
            node.bounds.origin.y - expansion,
            node.bounds.size.width + expansion * 2.0,
            node.bounds.size.height + expansion * 2.0,
        );
        Some(FocusIndicator {
            bounds,
            clip: node.clip,
            color: style.color.map(normalize_channel),
            width,
            radius: finite_non_negative(style.radius)
                .min(bounds.size.width.min(bounds.size.height) * 0.5),
        })
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn normalize_channel(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArrowKey, FocusIndicatorStyle, FocusManager, FocusPolicy, FocusSnapshot, FocusTraversal,
    };
    use crate::{
        ClipRegion, Direction, FrameSnapshot, LogicalRect, Overflow, WidgetGeometry, WidgetTree,
    };

    #[test]
    fn disabled_hidden_and_inert_states_cascade_through_subtrees() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let disabled = tree
            .append(
                root,
                FocusPolicy {
                    disabled: true,
                    ..FocusPolicy::default()
                },
            )
            .unwrap();
        let disabled_child = tree.append(disabled, FocusPolicy::focusable()).unwrap();
        let hidden = tree
            .append(
                root,
                FocusPolicy {
                    hidden: true,
                    ..FocusPolicy::default()
                },
            )
            .unwrap();
        let hidden_child = tree.append(hidden, FocusPolicy::focusable()).unwrap();
        let inert = tree
            .append(
                root,
                FocusPolicy {
                    inert: true,
                    ..FocusPolicy::default()
                },
            )
            .unwrap();
        let inert_child = tree.append(inert, FocusPolicy::focusable()).unwrap();
        let enabled = tree.append(root, FocusPolicy::focusable()).unwrap();
        let snapshot = FocusSnapshot::build(&tree, |_, policy| *policy);

        assert!(!snapshot.accepts_focus(disabled_child));
        assert!(!snapshot.accepts_focus(hidden_child));
        assert!(!snapshot.accepts_focus(inert_child));
        assert!(snapshot.accepts_focus(enabled));
    }

    #[test]
    fn focus_rejects_ineligible_and_unknown_widgets() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let eligible = tree.append(root, FocusPolicy::focusable()).unwrap();
        let ineligible = tree.append(root, FocusPolicy::default()).unwrap();
        let snapshot = FocusSnapshot::build(&tree, |_, policy| *policy);
        let mut manager = FocusManager::default();

        assert!(!manager.focus(&snapshot, ineligible));
        assert!(!manager.focus(&snapshot, crate::WidgetId::from_test_value(999)));
        assert!(manager.focus(&snapshot, eligible));
        assert_eq!(manager.focused(), Some(eligible));
    }

    #[test]
    fn skipped_tab_stop_still_accepts_programmatic_focus() {
        let policy = FocusPolicy {
            focusable: true,
            skip_tab_order: true,
            ..FocusPolicy::default()
        };
        let tree = WidgetTree::new(policy);
        let root = tree.root();
        let snapshot = FocusSnapshot::build(&tree, |_, policy| *policy);
        let mut manager = FocusManager::default();

        assert!(snapshot.accepts_focus(root));
        assert!(snapshot.tab_order().is_empty());
        assert!(manager.focus(&snapshot, root));
    }

    #[test]
    fn reconcile_clears_focus_after_removal_or_policy_change() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let child = tree.append(root, FocusPolicy::focusable()).unwrap();
        let first = FocusSnapshot::build(&tree, |_, policy| *policy);
        let mut manager = FocusManager::default();
        assert!(manager.focus(&first, child));

        tree.get_mut(child).unwrap().state.disabled = true;
        let disabled = FocusSnapshot::build(&tree, |_, policy| *policy);
        assert_eq!(manager.reconcile(&disabled), Some(child));
        assert_eq!(manager.focused(), None);

        tree.get_mut(child).unwrap().state.disabled = false;
        let restored = FocusSnapshot::build(&tree, |_, policy| *policy);
        assert!(manager.focus(&restored, child));
        tree.remove_subtree(child).unwrap();
        let removed = FocusSnapshot::build(&tree, |_, policy| *policy);
        assert_eq!(manager.reconcile(&removed), Some(child));
        assert_eq!(manager.focused(), None);
    }

    #[test]
    fn snapshot_keeps_semantic_tree_order() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let first = tree.append(root, FocusPolicy::focusable()).unwrap();
        let second = tree.append(root, FocusPolicy::focusable()).unwrap();
        let nested = tree.append(first, FocusPolicy::focusable()).unwrap();
        let snapshot = FocusSnapshot::build(&tree, |_, policy| *policy);

        assert_eq!(snapshot.semantic_order(), &[root, first, nested, second]);
    }

    #[test]
    fn tab_traversal_uses_semantic_order_and_skips_ineligible_nodes() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let first = tree.append(root, FocusPolicy::focusable()).unwrap();
        let skipped = tree
            .append(
                root,
                FocusPolicy {
                    focusable: true,
                    disabled: true,
                    ..FocusPolicy::default()
                },
            )
            .unwrap();
        let second = tree.append(root, FocusPolicy::focusable()).unwrap();
        let nested = tree.append(first, FocusPolicy::focusable()).unwrap();
        let snapshot = FocusSnapshot::build(&tree, |_, policy| *policy);
        let mut manager = FocusManager::default();

        assert_eq!(snapshot.tab_order(), &[first, nested, second]);
        assert!(!snapshot.tab_order().contains(&skipped));
        assert_eq!(
            manager.traverse(&snapshot, FocusTraversal::Forward),
            Some(first)
        );
        assert_eq!(
            manager.traverse(&snapshot, FocusTraversal::Forward),
            Some(nested)
        );
        assert_eq!(
            manager.traverse(&snapshot, FocusTraversal::Forward),
            Some(second)
        );
        assert_eq!(
            manager.traverse(&snapshot, FocusTraversal::Forward),
            Some(first)
        );
    }

    #[test]
    fn backward_tab_traversal_wraps_without_layout_direction() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let first = tree.append(root, FocusPolicy::focusable()).unwrap();
        let second = tree.append(root, FocusPolicy::focusable()).unwrap();
        let snapshot = FocusSnapshot::build(&tree, |_, policy| *policy);
        let mut manager = FocusManager::default();

        assert_eq!(
            manager.traverse(&snapshot, FocusTraversal::Backward),
            Some(second)
        );
        assert_eq!(
            manager.traverse(&snapshot, FocusTraversal::Backward),
            Some(first)
        );
        assert_eq!(
            manager.traverse(&snapshot, FocusTraversal::Backward),
            Some(second)
        );
    }

    #[test]
    fn empty_tab_order_clears_stale_focus() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let child = tree.append(root, FocusPolicy::focusable()).unwrap();
        let initial = FocusSnapshot::build(&tree, |_, policy| *policy);
        let mut manager = FocusManager::default();
        assert!(manager.focus(&initial, child));

        tree.get_mut(child).unwrap().state.disabled = true;
        let empty = FocusSnapshot::build(&tree, |_, policy| *policy);
        assert_eq!(manager.traverse(&empty, FocusTraversal::Forward), None);
        assert_eq!(manager.focused(), None);
    }

    #[test]
    fn horizontal_arrow_navigation_mirrors_with_local_direction() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let first = tree.append(root, FocusPolicy::focusable()).unwrap();
        let second = tree.append(root, FocusPolicy::focusable()).unwrap();
        let third = tree.append(root, FocusPolicy::focusable()).unwrap();
        let snapshot = FocusSnapshot::build(&tree, |_, policy| *policy);

        let mut ltr = FocusManager::default();
        assert!(ltr.focus(&snapshot, second));
        assert_eq!(
            ltr.navigate_arrow(&snapshot, ArrowKey::Right, Direction::Ltr),
            Some(third)
        );
        assert!(ltr.focus(&snapshot, second));
        assert_eq!(
            ltr.navigate_arrow(&snapshot, ArrowKey::Left, Direction::Ltr),
            Some(first)
        );

        let mut rtl = FocusManager::default();
        assert!(rtl.focus(&snapshot, second));
        assert_eq!(
            rtl.navigate_arrow(&snapshot, ArrowKey::Right, Direction::Rtl),
            Some(first)
        );
        assert!(rtl.focus(&snapshot, second));
        assert_eq!(
            rtl.navigate_arrow(&snapshot, ArrowKey::Left, Direction::Rtl),
            Some(third)
        );
    }

    #[test]
    fn vertical_arrow_navigation_is_direction_independent() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let first = tree.append(root, FocusPolicy::focusable()).unwrap();
        let second = tree.append(root, FocusPolicy::focusable()).unwrap();
        let third = tree.append(root, FocusPolicy::focusable()).unwrap();
        let snapshot = FocusSnapshot::build(&tree, |_, policy| *policy);

        for direction in [Direction::Ltr, Direction::Rtl] {
            let mut manager = FocusManager::default();
            assert!(manager.focus(&snapshot, second));
            assert_eq!(
                manager.navigate_arrow(&snapshot, ArrowKey::Up, direction),
                Some(first)
            );
            assert!(manager.focus(&snapshot, second));
            assert_eq!(
                manager.navigate_arrow(&snapshot, ArrowKey::Down, direction),
                Some(third)
            );
        }
    }

    #[test]
    fn focus_indicator_is_a_themeable_rounded_border_description() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let child = tree.append(root, FocusPolicy::focusable()).unwrap();
        let focus = FocusSnapshot::build(&tree, |_, policy| *policy);
        let frame = FrameSnapshot::build(&tree, |id, _| {
            WidgetGeometry::new(if id == root {
                LogicalRect::from_xywh(0.0, 0.0, 100.0, 100.0)
            } else {
                LogicalRect::from_xywh(20.0, 30.0, 40.0, 20.0)
            })
        });
        let mut manager = FocusManager::default();
        assert!(manager.focus(&focus, child));

        let indicator = manager
            .indicator(
                &frame,
                FocusIndicatorStyle {
                    color: [0.9, 0.5, 0.1, 0.8],
                    width: 3.0,
                    offset: 2.0,
                    radius: 8.0,
                },
            )
            .unwrap();

        assert_eq!(
            indicator.bounds,
            LogicalRect::from_xywh(15.0, 25.0, 50.0, 30.0)
        );
        assert_eq!(indicator.color, [0.9, 0.5, 0.1, 0.8]);
        assert_eq!(indicator.width, 3.0);
        assert_eq!(indicator.radius, 8.0);
    }

    #[test]
    fn focus_indicator_preserves_clip_and_normalizes_invalid_style() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let child = tree.append(root, FocusPolicy::focusable()).unwrap();
        let focus = FocusSnapshot::build(&tree, |_, policy| *policy);
        let frame = FrameSnapshot::build(&tree, |id, _| {
            if id == root {
                WidgetGeometry {
                    bounds: LogicalRect::from_xywh(0.0, 0.0, 30.0, 30.0),
                    overflow: Overflow::Clip,
                }
            } else {
                WidgetGeometry::new(LogicalRect::from_xywh(20.0, 20.0, 20.0, 20.0))
            }
        });
        let mut manager = FocusManager::default();
        assert!(manager.focus(&focus, child));
        let indicator = manager
            .indicator(
                &frame,
                FocusIndicatorStyle {
                    color: [2.0, -1.0, f32::NAN, 1.0],
                    width: f32::INFINITY,
                    offset: -2.0,
                    radius: 100.0,
                },
            )
            .unwrap();

        assert_eq!(
            indicator.clip,
            ClipRegion::Rect(LogicalRect::from_xywh(0.0, 0.0, 30.0, 30.0))
        );
        assert_eq!(indicator.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(indicator.width, 0.0);
        assert_eq!(indicator.radius, 10.0);
    }

    #[test]
    fn focus_indicator_requires_a_focused_widget_in_the_frame() {
        let tree = WidgetTree::new(FocusPolicy::focusable());
        let frame = FrameSnapshot::build(&tree, |_, _| {
            WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 10.0, 10.0))
        });
        let manager = FocusManager::default();

        assert_eq!(
            manager.indicator(&frame, FocusIndicatorStyle::default()),
            None
        );
    }

    #[test]
    fn rebuild_restores_focus_to_nearest_surviving_semantic_neighbor() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let first = tree.append(root, FocusPolicy::focusable()).unwrap();
        let second = tree.append(root, FocusPolicy::focusable()).unwrap();
        let third = tree.append(root, FocusPolicy::focusable()).unwrap();
        let initial = FocusSnapshot::build(&tree, |_, policy| *policy);
        let mut manager = FocusManager::default();
        assert!(manager.focus(&initial, second));

        tree.remove_subtree(second).unwrap();
        let without_middle = FocusSnapshot::build(&tree, |_, policy| *policy);
        assert_eq!(manager.restore_after_rebuild(&without_middle), Some(third));

        tree.remove_subtree(third).unwrap();
        let without_last = FocusSnapshot::build(&tree, |_, policy| *policy);
        assert_eq!(manager.restore_after_rebuild(&without_last), Some(first));
    }

    #[test]
    fn rebuild_clears_focus_when_no_eligible_widget_survives() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let child = tree.append(root, FocusPolicy::focusable()).unwrap();
        let initial = FocusSnapshot::build(&tree, |_, policy| *policy);
        let mut manager = FocusManager::default();
        assert!(manager.focus(&initial, child));

        tree.remove_subtree(child).unwrap();
        let empty = FocusSnapshot::build(&tree, |_, policy| *policy);
        assert_eq!(manager.restore_after_rebuild(&empty), None);
        assert_eq!(manager.focused(), None);
    }

    #[test]
    fn rebuild_preserves_focus_when_widget_remains_eligible() {
        let mut tree = WidgetTree::new(FocusPolicy::default());
        let root = tree.root();
        let focused = tree.append(root, FocusPolicy::focusable()).unwrap();
        let initial = FocusSnapshot::build(&tree, |_, policy| *policy);
        let mut manager = FocusManager::default();
        assert!(manager.focus(&initial, focused));

        tree.insert(root, 0, FocusPolicy::focusable()).unwrap();
        let rebuilt = FocusSnapshot::build(&tree, |_, policy| *policy);
        assert_eq!(manager.restore_after_rebuild(&rebuilt), Some(focused));
        assert_eq!(manager.focused(), Some(focused));
    }
}
