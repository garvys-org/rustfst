use std::fmt::Debug;

use crate::StateId;

/// Defines the different types of Queues usable.
#[derive(PartialOrd, PartialEq, Clone)]
pub enum QueueType {
    /// Single state queue.
    TrivialQueue,
    /// First-in, first-out queue.
    FifoQueue,
    /// Last-in, first-out queue.
    LifoQueue,
    /// Shortest-first queue.
    ShortestFirstQueue,
    /// Topologically-ordered queue.
    TopOrderQueue,
    /// State ID-ordered queue.
    StateOrderQueue,
    /// Component graph top-ordered meta-queue.
    SccQueue,
    /// Auto-selected queue.
    AutoQueue,
    OtherQueue,
}

// TODO: Test the queues with openfst
/// Unified interface to use different implementation of Queues.
pub trait Queue: Debug {
    fn head(&mut self) -> Option<StateId>;
    fn enqueue(&mut self, state: StateId);
    fn dequeue(&mut self) -> Option<StateId>;
    fn update(&mut self, state: StateId);
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
    fn queue_type(&self) -> QueueType;
}
