use std::cmp::Ordering;

use binary_heap_plus::{BinaryHeap,FnComparator};

use crate::StateId;
use crate::algorithms::{Queue, QueueType};

/// Shortest-first queue discipline, templated on the StateId and as well as a
/// comparison functor used to compare two StateIds. If a (single) state's order
/// changes, it can be reordered in the queue with a call to Update(). If update
/// is false, call to Update() does not reorder the queue.
pub struct ShortestFirstQueue<F> where F: Clone + FnMut(&StateId, &StateId) -> Ordering {
    heap: BinaryHeap<StateId, FnComparator<F>>,
}

impl<F> ShortestFirstQueue<F>
    where F: Clone + FnMut(&StateId, &StateId) -> Ordering
{
    pub fn new(f: F) -> Self {
        Self{heap: BinaryHeap::<StateId, FnComparator<F>>::new_by(f)}
    }
}

impl<F> Queue for ShortestFirstQueue<F>
    where F: Clone + FnMut(&StateId, &StateId) -> Ordering
{
    fn head(&self) -> Option<usize> {
        self.heap.peek().cloned()
    }

    fn enqueue(&mut self, state: usize) {
        self.heap.push(state);
    }

    fn dequeue(&mut self) {
        self.heap.pop();
    }

    fn update(&mut self, state: usize) {}

    fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    fn clear(&mut self) {
        self.heap.clear()
    }

    fn queue_type() -> QueueType {
        QueueType::ShortestFirstQueue
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use failure::Fallible;

    #[test]
    fn test_shortest_first_queue() -> Fallible<()> {
        let mut queue = ShortestFirstQueue::new(|a: &StateId, b: &StateId| a.cmp(b));

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