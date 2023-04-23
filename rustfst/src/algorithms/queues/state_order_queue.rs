use crate::algorithms::{Queue, QueueType};
use crate::StateId;

#[derive(Debug, Default, Clone)]
pub struct StateOrderQueue {
    front: usize,
    back: Option<usize>,
    enqueued: Vec<bool>,
}

impl Queue for StateOrderQueue {
    fn head(&mut self) -> Option<StateId> {
        Some(self.front as StateId)
    }

    fn enqueue(&mut self, state: StateId) {
        let state = state as usize;
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

    fn dequeue(&mut self) -> Option<StateId> {
        if self.is_empty() {
            return None;
        }
        let old_head = self.head();
        self.enqueued[self.front] = false;
        if let Some(back_) = self.back {
            while self.front <= back_ && !self.enqueued[self.front] {
                self.front += 1;
            }
        }
        old_head
    }

    fn update(&mut self, _state: StateId) {}

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
