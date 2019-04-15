use crate::algorithms::{Queue, QueueType};
use crate::StateId;

pub struct StateOrderQueue {
    front: StateId,
    back: Option<StateId>,
    enqueued: Vec<bool>,
}

impl StateOrderQueue {
    pub fn new() -> Self {
        Self {
            front: 0,
            back: None,
            enqueued: vec![],
        }
    }
}

impl Queue for StateOrderQueue {
    fn head(&mut self) -> Option<usize> {
        Some(self.front)
    }

    fn enqueue(&mut self, state: usize) {
        if self.back.is_none() || self.front > self.back.unwrap() {
            self.front = state;
            self.back = Some(state)
        } else if state > self.back.unwrap() {
            self.back = Some(state);
        } else if state < self.front {
            self.front = state;
        }

        while self.enqueued.len() <= state {
            self.enqueued.push(false);
        }
        self.enqueued[state] = true;
    }

    fn dequeue(&mut self) {
        self.enqueued[self.front] = false;
        if let Some(back_) = self.back {
            while self.front <= back_ && !self.enqueued[self.front] {
                self.front += 1;
            }
        }
    }

    fn update(&mut self, _state: usize) {}

    fn is_empty(&self) -> bool {
        if let Some(back_) = self.back {
            self.front > back_
        } else {
            true
        }
    }

    fn clear(&mut self) {
        if let Some(back_) = self.back {
            for i in self.front..=back_ {
                self.enqueued[i] = false;
            }
        }
        self.front = 0;
        self.back = None;
    }

    fn queue_type(&self) -> QueueType {
        QueueType::StateOrderQueue
    }
}
