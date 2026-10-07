use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueuePush {
    Added,
    DroppedOldest,
    Full,
    Closed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum QueuePop<T> {
    Item(T),
    TimedOut,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueReadiness {
    Ready,
    TimedOut,
    Closed,
}

#[derive(Debug)]
struct State<T> {
    items: VecDeque<T>,
    closed: bool,
}

#[derive(Debug)]
pub struct BoundedQueue<T> {
    capacity: usize,
    state: Mutex<State<T>>,
    ready: Condvar,
}

impl<T> BoundedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "queue capacity must be non-zero");
        Self {
            capacity,
            state: Mutex::new(State {
                items: VecDeque::with_capacity(capacity),
                closed: false,
            }),
            ready: Condvar::new(),
        }
    }

    pub fn push_latest(&self, item: T) -> QueuePush {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if state.closed {
            return QueuePush::Closed;
        }
        let outcome = if state.items.len() == self.capacity {
            state.items.pop_front();
            QueuePush::DroppedOldest
        } else {
            QueuePush::Added
        };
        state.items.push_back(item);
        self.ready.notify_one();
        outcome
    }

    pub fn push(&self, item: T) -> QueuePush {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if state.closed {
            return QueuePush::Closed;
        }
        if state.items.len() == self.capacity {
            return QueuePush::Full;
        }
        state.items.push_back(item);
        self.ready.notify_one();
        QueuePush::Added
    }

    pub fn replace(&self, item: T) -> QueuePush {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if state.closed {
            return QueuePush::Closed;
        }
        let outcome = if state.items.is_empty() {
            QueuePush::Added
        } else {
            QueuePush::DroppedOldest
        };
        state.items.clear();
        state.items.push_back(item);
        self.ready.notify_one();
        outcome
    }

    pub fn replace_where(&self, item: T, mut should_remove: impl FnMut(&T) -> bool) -> QueuePush {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if state.closed {
            return QueuePush::Closed;
        }
        let original_len = state.items.len();
        state.items.retain(|queued| !should_remove(queued));
        if state.items.len() == self.capacity {
            return QueuePush::Full;
        }
        let outcome = if state.items.len() == original_len {
            QueuePush::Added
        } else {
            QueuePush::DroppedOldest
        };
        state.items.push_back(item);
        self.ready.notify_one();
        outcome
    }

    pub fn try_pop(&self) -> Option<T> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .items
            .pop_front()
    }

    pub fn try_pop_latest(&self) -> Option<(T, usize)> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let latest = state.items.pop_back()?;
        let skipped = state.items.len();
        state.items.clear();
        Some((latest, skipped))
    }

    pub fn wait_pop(&self, timeout: Duration) -> QueuePop<T> {
        let start = Instant::now();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        loop {
            if let Some(item) = state.items.pop_front() {
                return QueuePop::Item(item);
            }
            if state.closed {
                return QueuePop::Closed;
            }
            let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
                return QueuePop::TimedOut;
            };
            let (next, result) = self
                .ready
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poison| poison.into_inner());
            state = next;
            if result.timed_out() {
                return state
                    .items
                    .pop_front()
                    .map_or(QueuePop::TimedOut, QueuePop::Item);
            }
        }
    }

    pub fn wait_readable(&self, timeout: Duration) -> QueueReadiness {
        let start = Instant::now();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        loop {
            if state.closed {
                return QueueReadiness::Closed;
            }
            if !state.items.is_empty() {
                return QueueReadiness::Ready;
            }
            let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
                return QueueReadiness::TimedOut;
            };
            let (next, _) = self
                .ready
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poison| poison.into_inner());
            state = next;
        }
    }

    pub fn pop_timeout(&self, timeout: Duration) -> Option<T> {
        match self.wait_pop(timeout) {
            QueuePop::Item(item) => Some(item),
            QueuePop::TimedOut | QueuePop::Closed => None,
        }
    }

    pub fn clear(&self) {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .items
            .clear();
    }

    pub fn is_closed(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .closed
    }

    pub fn close(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        state.closed = true;
        state.items.clear();
        self.ready.notify_all();
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .items
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_keeps_latest_and_skipped_items_for_the_consumer() {
        let queue = BoundedQueue::new(3);
        for frame in 1..=3 {
            queue.push_latest(frame);
        }
        assert_eq!(queue.wait_readable(Duration::ZERO), QueueReadiness::Ready);
        assert_eq!(queue.len(), 3);
        assert_eq!(queue.try_pop_latest(), Some((3, 2)));
        assert_eq!(queue.wait_readable(Duration::ZERO), QueueReadiness::TimedOut);
    }

    #[test]
    fn readiness_wakes_on_push_and_close() {
        use std::sync::{Arc, mpsc};
        for close in [false, true] {
            let queue = Arc::new(BoundedQueue::new(1));
            let waiting = Arc::clone(&queue);
            let (started, start) = mpsc::sync_channel(1);
            let (finished, finish) = mpsc::sync_channel(1);
            let worker = std::thread::spawn(move || {
                started.send(()).unwrap();
                finished
                    .send(waiting.wait_readable(Duration::from_secs(5)))
                    .unwrap();
            });
            start.recv().unwrap();
            if close {
                queue.close();
            } else {
                queue.push_latest(7);
            }
            assert_eq!(
                finish.recv_timeout(Duration::from_secs(1)).unwrap(),
                if close {
                    QueueReadiness::Closed
                } else {
                    QueueReadiness::Ready
                }
            );
            worker.join().unwrap();
            assert_eq!(
                queue.try_pop_latest(),
                if close { None } else { Some((7, 0)) }
            );
        }
    }

    #[test]
    fn spurious_readiness_wakes_do_not_restart_the_timeout() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        let queue = Arc::new(BoundedQueue::<u8>::new(1));
        let notified = Arc::clone(&queue);
        let done = Arc::new(AtomicBool::new(false));
        let notifier_done = Arc::clone(&done);
        let worker = std::thread::spawn(move || {
            let start = Instant::now();
            while !notifier_done.load(Ordering::Acquire)
                && start.elapsed() < Duration::from_millis(500)
            {
                notified.ready.notify_all();
                std::thread::sleep(Duration::from_millis(1));
            }
        });
        let start = Instant::now();
        assert_eq!(
            queue.wait_readable(Duration::from_millis(25)),
            QueueReadiness::TimedOut
        );
        let elapsed = start.elapsed();
        done.store(true, Ordering::Release);
        worker.join().unwrap();
        assert!(elapsed >= Duration::from_millis(25));
        assert!(elapsed < Duration::from_millis(250));
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn latest_pop_discards_stale_frames_and_counts_them() {
        let queue = BoundedQueue::new(3);
        assert_eq!(queue.try_pop_latest(), None);
        for frame in 1..=3 {
            assert_eq!(queue.push(frame), QueuePush::Added);
        }
        assert_eq!(queue.try_pop_latest(), Some((3, 2)));
        assert_eq!(queue.try_pop_latest(), None);
        assert_eq!(queue.push(4), QueuePush::Added);
        assert_eq!(queue.try_pop_latest(), Some((4, 0)));
    }

    #[test]
    fn keeps_newest_items_at_capacity() {
        let queue = BoundedQueue::new(2);
        assert_eq!(queue.push_latest(1), QueuePush::Added);
        assert_eq!(queue.push_latest(2), QueuePush::Added);
        assert_eq!(queue.push_latest(3), QueuePush::DroppedOldest);
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.try_pop(), Some(2));
        assert_eq!(queue.try_pop(), Some(3));
    }

    #[test]
    fn close_wakes_waiters_and_rejects_pushes() {
        let queue = BoundedQueue::new(1);
        queue.close();
        assert_eq!(queue.wait_pop(Duration::from_secs(1)), QueuePop::Closed);
        assert_eq!(queue.push_latest(1), QueuePush::Closed);
        assert_eq!(queue.push(1), QueuePush::Closed);
    }

    #[test]
    fn timeout_does_not_close_an_idle_queue() {
        let queue = BoundedQueue::new(1);
        assert_eq!(queue.wait_pop(Duration::from_millis(1)), QueuePop::TimedOut);
        assert_eq!(queue.push_latest(7), QueuePush::Added);
        assert_eq!(queue.wait_pop(Duration::from_secs(1)), QueuePop::Item(7));
    }

    #[test]
    fn non_evicting_push_and_atomic_replace_preserve_control() {
        let queue = BoundedQueue::new(1);
        assert_eq!(queue.push(1), QueuePush::Added);
        assert_eq!(queue.push(2), QueuePush::Full);
        assert_eq!(queue.replace(3), QueuePush::DroppedOldest);
        assert_eq!(queue.try_pop(), Some(3));
    }

    #[test]
    fn conditional_replace_never_removes_control() {
        let queue = BoundedQueue::new(2);
        assert_eq!(queue.push((false, 1)), QueuePush::Added);
        assert_eq!(queue.push((true, 2)), QueuePush::Added);
        assert_eq!(
            queue.replace_where((true, 3), |(media, _)| *media),
            QueuePush::DroppedOldest
        );
        assert_eq!(queue.try_pop(), Some((false, 1)));
        assert_eq!(queue.try_pop(), Some((true, 3)));
    }
}
