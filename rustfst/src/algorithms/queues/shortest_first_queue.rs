use std::cmp::Ordering;

use binary_heap_plus::{BinaryHeap, FnComparator};

use failure::Fallible;

use crate::algorithms::{Queue, QueueType};
use crate::semirings::Semiring;
use crate::StateId;

#[derive(Clone)]
pub struct StateWeightCompare<W: Semiring, C: Clone + Fn(&W, &W) -> Fallible<bool>> {
    less: C,
    weights: Vec<W>,
}

impl<W: Semiring, C: Clone + Fn(&W, &W) -> Fallible<bool>> StateWeightCompare<W, C> {
    pub fn new(weights: Vec<W>, less: C) -> Self {
        Self { less, weights }
    }

    pub fn compare(&self, s1: &StateId, s2: &StateId) -> Fallible<bool> {
        (self.less)(&self.weights[*s1], &self.weights[*s2])
    }
}

pub fn natural_less<W: Semiring>(w1: &W, w2: &W) -> Fallible<bool> {
    Ok((&w1.plus(w2)? == w1) && (w1 != w2))
}

pub struct ShortestFirstQueue<C: Clone + FnMut(&StateId, &StateId) -> Ordering> {
    heap: BinaryHeap<StateId, FnComparator<C>>,
}

impl<C: Clone + FnMut(&StateId, &StateId) -> Ordering> ShortestFirstQueue<C> {
    pub fn new(c: C) -> Self {
        Self {
            heap: BinaryHeap::new_by(c),
        }
    }
}

impl<C: Clone + FnMut(&StateId, &StateId) -> Ordering> Queue for ShortestFirstQueue<C> {
    fn head(&mut self) -> Option<usize> {
        self.heap.peek().cloned()
    }

    fn enqueue(&mut self, state: usize) {
        self.heap.push(state);
    }

    fn dequeue(&mut self) {
        self.heap.pop();
    }

    fn update(&mut self, state: usize) {
        unimplemented!()
    }

    fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    fn clear(&mut self) {
        self.heap.clear()
    }

    fn queue_type(&self) -> QueueType {
        QueueType::ShortestFirstQueue
    }
}

pub struct NaturalShortestFirstQueue {
    queue: Box<Queue>,
}

impl NaturalShortestFirstQueue {
    pub fn new<W: 'static + Semiring>(weights: Vec<W>) -> Self {
        let a = StateWeightCompare::new(weights, natural_less);
        let heap = ShortestFirstQueue::new(move |v1, v2| {
            if a.compare(v1, v2).unwrap() {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        NaturalShortestFirstQueue {
            queue: Box::new(heap),
        }
    }
}

impl Queue for NaturalShortestFirstQueue {
    fn head(&mut self) -> Option<usize> {
        self.queue.head()
    }

    fn enqueue(&mut self, state: usize) {
        self.queue.enqueue(state)
    }

    fn dequeue(&mut self) {
        self.queue.dequeue()
    }

    fn update(&mut self, state: usize) {
        self.queue.update(state)
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn clear(&mut self) {
        self.queue.clear()
    }

    fn queue_type(&self) -> QueueType {
        self.queue.queue_type()
    }
}
