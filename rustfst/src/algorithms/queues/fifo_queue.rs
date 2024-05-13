use std::collections::VecDeque;

use crate::algorithms::{Queue, QueueType};
use crate::StateId;

/// First-in, first-out (queue) queue discipline.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FifoQueue(VecDeque<StateId>);

impl Queue for FifoQueue {
    fn head(&mut self) -> Option<StateId> {
        self.0.front().cloned()
    }

    fn enqueue(&mut self, state: StateId) {
        self.0.push_back(state)
    }

    fn dequeue(&mut self) -> Option<StateId> {
        self.0.pop_front()
    }

    fn update(&mut self, _state: StateId) {}

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn clear(&mut self) {
        self.0.clear()
    }

    fn queue_type(&self) -> QueueType {
        QueueType::FifoQueue
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_fifo_queue() -> Result<()> {
        let mut queue = FifoQueue::default();

        assert_eq!(queue.head(), None);

        queue.enqueue(2);
        queue.enqueue(3);
        assert_eq!(queue.head(), Some(2));
        queue.dequeue();
        assert_eq!(queue.head(), Some(3));
        queue.dequeue();

        queue.enqueue(2);
        queue.enqueue(3);
        assert!(!queue.is_empty());
        assert_eq!(queue.head(), Some(2));
        queue.clear();
        assert_eq!(queue.head(), None);
        assert!(queue.is_empty());

        Ok(())
    }
}
