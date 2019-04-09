use crate::StateId;
use crate::algorithms::{Queue, QueueType};

/// Trivial queue discipline; one may enqueue at most one state at a time. It
/// can be used for strongly connected components with only one state and no
/// self-loops.
pub struct TrivialQueue {
    state: Option<StateId>,
}

impl TrivialQueue {
    pub fn new() -> Self {
        Self{state: None}
    }
}

impl Queue for TrivialQueue {
    fn head(&self) -> Option<StateId> {
        self.state
    }

    fn enqueue(&mut self, state: usize) {
        self.state = Some(state);
    }

    fn dequeue(&mut self) {
        self.state = None;
    }

    fn update(&mut self, state: usize) {}

    fn is_empty(&self) -> bool {
        self.state.is_none()
    }

    fn clear(&mut self) {
        self.state = None;
    }

    fn queue_type() -> QueueType {
        QueueType::TrivialQueue
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use failure::Fallible;

    #[test]
    fn test_trivial_queue() -> Fallible<()> {
        let mut queue = TrivialQueue::new();

        assert_eq!(queue.head(), None);

        queue.enqueue(2);
        queue.enqueue(3);
        assert_eq!(queue.head(), Some(3));
        queue.dequeue();
        assert_eq!(queue.head(), None);

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