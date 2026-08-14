// frame.rs

use std::collections::HashMap;

use crate::{ClipRegion, LogicalPoint, LogicalRect, Overflow, WidgetId, WidgetTree};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidgetGeometry {
    pub bounds: LogicalRect,
    pub overflow: Overflow,
}

impl WidgetGeometry {
    pub fn new(bounds: LogicalRect) -> Self {
        Self {
            bounds,
            overflow: Overflow::Visible,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameNode {
    pub id: WidgetId,
    pub bounds: LogicalRect,
    pub clip: ClipRegion<crate::Logical>,
    pub paint_index: usize,
}

#[derive(Clone, Debug)]
pub struct FrameSnapshot {
    root: WidgetId,
    nodes: HashMap<WidgetId, FrameNode>,
    paint_order: Vec<WidgetId>,
}

impl FrameSnapshot {
    pub fn build<State>(
        tree: &WidgetTree<State>,
        mut layout: impl FnMut(WidgetId, Option<FrameNode>) -> WidgetGeometry,
    ) -> Self {
        let root = tree.root();
        let mut nodes = HashMap::with_capacity(tree.len());
        let mut paint_order = Vec::with_capacity(tree.len());
        let mut stack = vec![(root, None)];

        while let Some((id, parent)) = stack.pop() {
            let geometry = layout(id, parent);
            let own_clip = ClipRegion::from_overflow(geometry.bounds, geometry.overflow);
            let clip = parent
                .map(|parent| parent.clip.intersect(own_clip))
                .unwrap_or(own_clip);
            let frame_node = FrameNode {
                id,
                bounds: geometry.bounds,
                clip,
                paint_index: paint_order.len(),
            };
            nodes.insert(id, frame_node);
            paint_order.push(id);
            stack.extend(
                tree.get(id)
                    .unwrap()
                    .children()
                    .iter()
                    .rev()
                    .map(|child| (*child, Some(frame_node))),
            );
        }

        Self {
            root,
            nodes,
            paint_order,
        }
    }

    pub fn root(&self) -> WidgetId {
        self.root
    }

    pub fn get(&self, id: WidgetId) -> Option<&FrameNode> {
        self.nodes.get(&id)
    }

    pub fn paint_order(&self) -> &[WidgetId] {
        &self.paint_order
    }

    pub fn hit_test(&self, point: LogicalPoint) -> Option<WidgetId> {
        self.paint_order.iter().rev().copied().find(|id| {
            let node = self.nodes[id];
            node.bounds.contains(point) && node.clip.contains(point)
        })
    }

    pub fn paint(&self, mut paint: impl FnMut(FrameNode)) {
        for id in &self.paint_order {
            paint(self.nodes[id]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameSnapshot, WidgetGeometry};
    use crate::{ClipRegion, LogicalPoint, LogicalRect, Overflow, WidgetTree};

    #[test]
    fn paint_and_hit_test_use_opposite_deterministic_stacking_order() {
        let mut tree = WidgetTree::new("root");
        let root = tree.root();
        let back = tree.append(root, "back").unwrap();
        let front = tree.append(root, "front").unwrap();
        let snapshot = FrameSnapshot::build(&tree, |id, _| {
            if id == root {
                WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 100.0, 100.0))
            } else {
                WidgetGeometry::new(LogicalRect::from_xywh(10.0, 10.0, 40.0, 40.0))
            }
        });

        assert_eq!(snapshot.paint_order(), &[root, back, front]);
        assert_eq!(
            snapshot.hit_test(LogicalPoint::new(20.0, 20.0)),
            Some(front)
        );
        assert_eq!(snapshot.get(back).unwrap().paint_index, 1);
        assert_eq!(snapshot.get(front).unwrap().paint_index, 2);
    }

    #[test]
    fn nested_clips_reject_visible_geometry_outside_ancestor() {
        let mut tree = WidgetTree::new("root");
        let root = tree.root();
        let parent = tree.append(root, "parent").unwrap();
        let child = tree.append(parent, "child").unwrap();
        let snapshot = FrameSnapshot::build(&tree, |id, _| match id {
            id if id == root => WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 100.0, 100.0)),
            id if id == parent => WidgetGeometry {
                bounds: LogicalRect::from_xywh(20.0, 20.0, 30.0, 30.0),
                overflow: Overflow::Clip,
            },
            _ => WidgetGeometry::new(LogicalRect::from_xywh(40.0, 30.0, 40.0, 20.0)),
        });

        assert_eq!(
            snapshot.get(child).unwrap().clip,
            ClipRegion::Rect(LogicalRect::from_xywh(20.0, 20.0, 30.0, 30.0))
        );
        assert_eq!(
            snapshot.hit_test(LogicalPoint::new(45.0, 35.0)),
            Some(child)
        );
        assert_ne!(
            snapshot.hit_test(LogicalPoint::new(60.0, 35.0)),
            Some(child)
        );
    }

    #[test]
    fn empty_parent_clip_makes_descendants_unhittable() {
        let mut tree = WidgetTree::new(());
        let root = tree.root();
        let child = tree.append(root, ()).unwrap();
        let snapshot = FrameSnapshot::build(&tree, |id, _| {
            if id == root {
                WidgetGeometry {
                    bounds: LogicalRect::from_xywh(0.0, 0.0, 0.0, 100.0),
                    overflow: Overflow::Clip,
                }
            } else {
                WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 50.0, 50.0))
            }
        });

        assert_eq!(snapshot.get(child).unwrap().clip, ClipRegion::Empty);
        assert_eq!(snapshot.hit_test(LogicalPoint::new(10.0, 10.0)), None);
    }

    #[test]
    fn snapshot_is_unchanged_when_tree_mutates_after_layout() {
        let mut tree = WidgetTree::new(0);
        let root = tree.root();
        let original = tree.append(root, 1).unwrap();
        let snapshot = FrameSnapshot::build(&tree, |id, _| {
            WidgetGeometry::new(if id == root {
                LogicalRect::from_xywh(0.0, 0.0, 100.0, 100.0)
            } else {
                LogicalRect::from_xywh(10.0, 10.0, 20.0, 20.0)
            })
        });
        let added = tree.append(root, 2).unwrap();
        tree.remove_subtree(original).unwrap();

        assert!(snapshot.get(original).is_some());
        assert!(snapshot.get(added).is_none());
        assert_eq!(snapshot.paint_order(), &[root, original]);
    }

    #[test]
    fn paint_callback_receives_the_frozen_snapshot_order() {
        let mut tree = WidgetTree::new(());
        let root = tree.root();
        let child = tree.append(root, ()).unwrap();
        let snapshot = FrameSnapshot::build(&tree, |_, _| {
            WidgetGeometry::new(LogicalRect::from_xywh(0.0, 0.0, 10.0, 10.0))
        });
        let mut painted = Vec::new();

        snapshot.paint(|node| painted.push(node.id));

        assert_eq!(painted, [root, child]);
    }
}
