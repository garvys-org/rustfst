use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::Formatter;

use anyhow::Result;
use binary_heap_plus::{BinaryHeap, FnComparator};

use crate::algorithms::{Queue, QueueType};
use crate::semirings::Semiring;
use crate::StateId;

#[derive(Clone)]
pub struct StateWeightCompare<W: Semiring, C: Clone + Fn(&W, &W) -> Result<bool>> {
    less: C,
    weights: Vec<W>,
}

impl<W: Semiring, C: Clone + Fn(&W, &W) -> Result<bool>> StateWeightCompare<W, C> {
    pub fn new(weights: Vec<W>, less: C) -> Self {
        Self { less, weights }
    }

    pub fn compare(&self, s1: StateId, s2: StateId) -> Result<bool> {
        (self.less)(&self.weights[s1 as usize], &self.weights[s2 as usize])
    }
}

pub fn natural_less<W: Semiring>(w1: &W, w2: &W) -> Result<bool> {
    Ok((&w1.plus(w2)? == w1) && (w1 != w2))
}

#[derive(Clone)]
pub struct ShortestFirstQueue<C: Clone + FnMut(&StateId, &StateId) -> Ordering> {
    heap: BinaryHeap<StateId, FnComparator<C>>,
}

impl<C: Clone + FnMut(&StateId, &StateId) -> Ordering> Debug for ShortestFirstQueue<C> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(format!("ShortestFirstQueue {{ heap: {:?} }}", self.heap).as_str())
    }
}

impl<C: Clone + FnMut(&StateId, &StateId) -> Ordering> ShortestFirstQueue<C> {
    pub fn new(c: C) -> Self {
        Self {
            heap: BinaryHeap::new_by(c),
        }
    }
}

impl<C: Clone + FnMut(&StateId, &StateId) -> Ordering> Queue for ShortestFirstQueue<C> {
    fn head(&mut self) -> Option<StateId> {
        self.heap.peek().cloned()
    }

    fn enqueue(&mut self, state: StateId) {
        self.heap.push(state);
    }

    fn dequeue(&mut self) -> Option<StateId> {
        self.heap.pop()
    }

    fn update(&mut self, _state: StateId) {
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

#[derive(Debug)]
pub struct NaturalShortestFirstQueue {
    queue: Box<dyn Queue>,
}

impl NaturalShortestFirstQueue {
    pub fn new<W: 'static + Semiring>(weights: Vec<W>) -> Self {
        let a = StateWeightCompare::new(weights, natural_less);
        let heap = ShortestFirstQueue::new(move |v1, v2| {
            if a.compare(*v1, *v2).unwrap() {
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
    fn head(&mut self) -> Option<StateId> {
        self.queue.head()
    }

    fn enqueue(&mut self, state: StateId) {
        self.queue.enqueue(state)
    }

    fn dequeue(&mut self) -> Option<StateId> {
        self.queue.dequeue()
    }

    fn update(&mut self, state: StateId) {
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
