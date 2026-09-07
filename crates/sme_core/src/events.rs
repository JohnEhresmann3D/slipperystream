//! Typed, bounded, single-owner event queues. No globals, threads, networking or callbacks.
//!
//! Producers enqueue owned values. The host dispatches FIFO at an explicit boundary
//! (e.g. after fixed update, before audio/render). Drain once and explicitly fan out to
//! multiple consumers; competing drains are NOT broadcast subscriptions. Consumers that
//! produce more events should use a separate next-boundary queue to avoid feedback loops.
//! Overflow returns the unsent event so the caller must choose retry/coalescing/rejection.
//! Bounds limit event count, not the byte size of arbitrary payloads.
use std::collections::VecDeque;

#[derive(Debug)]
pub struct EventQueue<E> {
    pending: VecDeque<E>,
    capacity: usize,
}

impl<E> EventQueue<E> {
    /// A zero-capacity queue is valid and rejects every event.
    pub fn new(capacity: usize) -> Self {
        Self {
            pending: VecDeque::new(),
            capacity,
        }
    }

    pub fn send(&mut self, event: E) -> Result<(), E> {
        if self.pending.len() >= self.capacity {
            return Err(event);
        }
        self.pending.push_back(event);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<E> {
        self.pending.pop_front()
    }
    pub fn len(&self) -> usize {
        self.pending.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Removes the current batch. Dropping the iterator also drops any unread events.
    /// The exclusive borrow prevents producers from enqueueing during this dispatch.
    pub fn drain(&mut self) -> impl Iterator<Item = E> + '_ {
        self.pending.drain(..)
    }

    /// Explicit lifecycle cancellation (e.g. unloading a scene).
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_batches_are_exactly_once_and_overflow_returns_ownership() {
        let mut queue = EventQueue::new(2);
        queue.send(String::from("exploration")).unwrap();
        queue.send(String::from("boss")).unwrap();
        assert_eq!(
            queue.send(String::from("victory")),
            Err(String::from("victory"))
        );
        assert_eq!(queue.drain().collect::<Vec<_>>(), ["exploration", "boss"]);
        assert_eq!(queue.pop(), None);
        queue.send(String::from("next boundary")).unwrap();
        assert_eq!(queue.len(), 1);
        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn zero_capacity_and_abandoned_batch_have_explicit_semantics() {
        assert_eq!(EventQueue::new(0).send(7), Err(7));
        let mut queue = EventQueue::new(3);
        for i in 0..3 {
            queue.send(i).unwrap();
        }
        assert_eq!(queue.drain().next(), Some(0));
        assert!(queue.is_empty());
        assert_eq!(queue.capacity(), 3);
    }

    #[test]
    fn dispatcher_can_broadcast_in_order_without_cloning_payloads() {
        let mut queue = EventQueue::new(2);
        queue.send(10).unwrap();
        queue.send(20).unwrap();
        let (mut audio, mut observer) = (Vec::new(), Vec::new());
        for event in queue.drain() {
            audio.push(event);
            observer.push(event);
        }
        assert_eq!(audio, observer);
        assert_eq!(audio, [10, 20]);
    }
}
