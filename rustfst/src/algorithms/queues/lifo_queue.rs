use crate::algorithms::{Queue, QueueType};
use crate::StateId;

/// Last-in, first-out (stack) queue discipline.
#[derive(Debug, Default, Clone)]
pub struct LifoQueue(Vec<StateId>);

impl Queue for LifoQueue {
    fn head(&mut self) -> Option<StateId> {
        self.0.last().cloned()
    }

    fn enqueue(&mut self, state: StateId) {
        self.0.push(state)
    }

    fn dequeue(&mut self) -> Option<StateId> {
        self.0.pop()
    }

    fn update(&mut self, _state: StateId) {}

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn clear(&mut self) {
        self.0.clear()
    }

    fn queue_type(&self) -> QueueType {
        QueueType::LifoQueue
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_lifo_queue() -> Result<()> {
        let mut queue = LifoQueue::default();

        assert_eq!(queue.head(), None);

        queue.enqueue(2);
        queue.enqueue(3);
        assert_eq!(queue.head(), Some(3));
        queue.dequeue();
        assert_eq!(queue.head(), Some(2));
        queue.dequeue();

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
