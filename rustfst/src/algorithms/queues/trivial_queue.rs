use crate::algorithms::{Queue, QueueType};
use crate::StateId;

/// Trivial queue discipline; one may enqueue at most one state at a time. It
/// can be used for strongly connected components with only one state and no
/// self-loops.
#[derive(Debug, Default, Clone)]
pub struct TrivialQueue {
    state: Option<StateId>,
}

impl Queue for TrivialQueue {
    fn head(&mut self) -> Option<StateId> {
        self.state
    }

    fn enqueue(&mut self, state: StateId) {
        self.state = Some(state);
    }

    fn dequeue(&mut self) -> Option<StateId> {
        self.state.take()
    }

    fn update(&mut self, _state: StateId) {}

    fn is_empty(&self) -> bool {
        self.state.is_none()
    }

    fn clear(&mut self) {
        self.state = None;
    }

    fn queue_type(&self) -> QueueType {
        QueueType::TrivialQueue
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use anyhow::Result;

    #[test]
    fn test_trivial_queue() -> Result<()> {
        let mut queue = TrivialQueue::default();

        assert_eq!(queue.head(), None);

        queue.enqueue(2);
        queue.enqueue(3);
        assert_eq!(queue.head(), Some(3));
        queue.dequeue();
        assert_eq!(queue.head(), None);

        queue.enqueue(2);
        queue.enqueue(3);
        assert!(!queue.is_empty());
        assert_eq!(queue.head(), Some(3));
        queue.clear();
        assert_eq!(queue.head(), None);
        assert!(queue.is_empty());
        Ok(())
    }
}
