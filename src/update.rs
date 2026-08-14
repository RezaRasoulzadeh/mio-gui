// update.rs

use std::collections::VecDeque;

use crate::{WidgetId, WidgetTree};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Invalidation {
    #[default]
    None,
    Paint,
    Layout,
}

impl Invalidation {
    pub const fn merge(self, other: Self) -> Self {
        if self as u8 >= other as u8 {
            self
        } else {
            other
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WidgetMessage<Message> {
    pub target: WidgetId,
    pub message: Message,
}

#[derive(Clone, Debug)]
pub struct UpdateQueue<Message> {
    messages: VecDeque<WidgetMessage<Message>>,
    invalidation: Invalidation,
}

impl<Message> Default for UpdateQueue<Message> {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
            invalidation: Invalidation::None,
        }
    }
}

impl<Message> UpdateQueue<Message> {
    pub fn emit(&mut self, target: WidgetId, message: Message) {
        self.messages.push_back(WidgetMessage { target, message });
    }

    pub fn request_paint(&mut self) {
        self.invalidation = self.invalidation.merge(Invalidation::Paint);
    }

    pub fn request_layout(&mut self) {
        self.invalidation = Invalidation::Layout;
    }

    pub fn pending(&self) -> usize {
        self.messages.len()
    }

    pub fn invalidation(&self) -> Invalidation {
        self.invalidation
    }

    pub fn take_invalidation(&mut self) -> Invalidation {
        std::mem::take(&mut self.invalidation)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DispatchReport {
    pub handled: usize,
    pub dropped: usize,
    pub remaining: usize,
    pub invalidation: Invalidation,
}

pub struct UpdateRuntime<State, Message> {
    tree: WidgetTree<State>,
    queue: UpdateQueue<Message>,
}

impl<State, Message> UpdateRuntime<State, Message> {
    pub fn new(root_state: State) -> Self {
        Self {
            tree: WidgetTree::new(root_state),
            queue: UpdateQueue::default(),
        }
    }

    pub fn tree(&self) -> &WidgetTree<State> {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut WidgetTree<State> {
        &mut self.tree
    }

    pub fn queue(&self) -> &UpdateQueue<Message> {
        &self.queue
    }

    pub fn queue_mut(&mut self) -> &mut UpdateQueue<Message> {
        &mut self.queue
    }

    pub fn dispatch(
        &mut self,
        budget: usize,
        mut update: impl FnMut(&mut State, Message, &mut UpdateQueue<Message>),
    ) -> DispatchReport {
        let mut handled = 0;
        let mut dropped = 0;
        while handled + dropped < budget {
            let Some(queued) = self.queue.messages.pop_front() else {
                break;
            };
            if let Some(node) = self.tree.get_mut(queued.target) {
                update(&mut node.state, queued.message, &mut self.queue);
                handled += 1;
            } else {
                dropped += 1;
            }
        }
        DispatchReport {
            handled,
            dropped,
            remaining: self.queue.messages.len(),
            invalidation: self.queue.take_invalidation(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Invalidation, UpdateQueue, UpdateRuntime};

    #[derive(Clone, Copy)]
    enum Message {
        Add(i32),
        Cascade(i32),
    }

    #[test]
    fn messages_are_processed_fifo_and_emission_is_non_reentrant() {
        let mut runtime = UpdateRuntime::new(Vec::<i32>::new());
        let root = runtime.tree().root();
        runtime.queue_mut().emit(root, Message::Cascade(1));
        runtime.queue_mut().emit(root, Message::Add(3));

        let report = runtime.dispatch(10, |state, message, queue| match message {
            Message::Add(value) => state.push(value),
            Message::Cascade(value) => {
                state.push(value);
                queue.emit(root, Message::Add(2));
            }
        });

        assert_eq!(runtime.tree().get(root).unwrap().state, [1, 3, 2]);
        assert_eq!(report.handled, 3);
        assert_eq!(report.remaining, 0);
    }

    #[test]
    fn budget_preserves_remaining_work_for_next_event_loop_turn() {
        let mut runtime = UpdateRuntime::new(0_u32);
        let root = runtime.tree().root();
        runtime.queue_mut().emit(root, ());

        let first = runtime.dispatch(3, |state, (), queue| {
            *state += 1;
            queue.emit(root, ());
        });
        assert_eq!(first.handled, 3);
        assert_eq!(first.remaining, 1);
        assert_eq!(runtime.tree().get(root).unwrap().state, 3);

        let second = runtime.dispatch(2, |state, (), queue| {
            *state += 1;
            queue.emit(root, ());
        });
        assert_eq!(second.handled, 2);
        assert_eq!(second.remaining, 1);
        assert_eq!(runtime.tree().get(root).unwrap().state, 5);
    }

    #[test]
    fn messages_for_removed_widgets_are_dropped_safely() {
        let mut runtime = UpdateRuntime::new(0_i32);
        let root = runtime.tree().root();
        let child = runtime.tree_mut().append(root, 10).unwrap();
        runtime.queue_mut().emit(child, Message::Add(5));
        runtime.tree_mut().remove_subtree(child).unwrap();

        let report = runtime.dispatch(10, |state, message, _| match message {
            Message::Add(value) | Message::Cascade(value) => *state += value,
        });

        assert_eq!(report.dropped, 1);
        assert_eq!(report.handled, 0);
        assert_eq!(runtime.tree().get(root).unwrap().state, 0);
    }

    #[test]
    fn layout_invalidation_dominates_paint_until_consumed() {
        let mut queue = UpdateQueue::<()>::default();
        queue.request_paint();
        assert_eq!(queue.invalidation(), Invalidation::Paint);
        queue.request_layout();
        queue.request_paint();
        assert_eq!(queue.invalidation(), Invalidation::Layout);
        assert_eq!(queue.take_invalidation(), Invalidation::Layout);
        assert_eq!(queue.invalidation(), Invalidation::None);
    }
}
