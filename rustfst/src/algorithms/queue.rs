use crate::StateId;

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

pub trait Queue {
    fn head(&mut self) -> Option<StateId>;
    fn enqueue(&mut self, state: StateId);
    fn dequeue(&mut self);
    fn update(&mut self, state: StateId);
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
    fn queue_type(&self) -> QueueType;
}
