use crate::algorithms::{Queue, QueueType};
use crate::StateId;

/// Last-in, first-out (stack) queue discipline.
#[derive(Debug)]
pub struct LifoQueue(Vec<StateId>);

impl LifoQueue {
    pub fn new() -> Self {
        Self(vec![])
    }
}

impl Queue for LifoQueue {
    fn head(&mut self) -> Option<usize> {
        self.0.last().cloned()
    }

    fn enqueue(&mut self, state: usize) {
        self.0.push(state)
    }

    fn dequeue(&mut self) {
        self.0.pop();
    }

    fn update(&mut self, _state: usize) {}

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
    use failure::Fallible;

    #[test]
    fn test_lifo_queue() -> Fallible<()> {
        let mut queue = LifoQueue::new();

        assert_eq!(queue.head(), None);

        queue.enqueue(2);
        queue.enqueue(3);
        assert_eq!(queue.head(), Some(3));
        queue.dequeue();
        assert_eq!(queue.head(), Some(2));
        queue.dequeue();

        queue.enqueue(2);
        queue.enqueue(3);
        assert_eq!(queue.is_empty(), false);
        assert_eq!(queue.head(), Some(3));
        queue.clear();
        assert_eq!(queue.head(), None);
        assert_eq!(queue.is_empty(), true);

        Ok(())
    }
}
