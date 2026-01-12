use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// A simulation clock that manages virtual time and async task scheduling.
///
/// This clock allows tasks to sleep until specific times, and advances
/// time in discrete steps based on scheduled wake events.
pub struct SimClock {
    inner: Arc<Mutex<ClockInner>>,
}

struct ClockInner {
    pub now: f64,
    pub wakers: BinaryHeap<Reverse<WakeEvent>>,
}

#[derive(Debug)]
struct WakeEvent {
    time: f64,
    waker: Waker,
}

impl PartialEq for WakeEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl Eq for WakeEvent {}
impl PartialOrd for WakeEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for WakeEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.total_cmp(&other.time)
    }
}

impl SimClock {
    /// Creates a new simulation clock starting at time 0.0
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ClockInner {
                now: 0.0,
                wakers: BinaryHeap::new(),
            })),
        }
    }

    /// Returns the current simulation time
    pub fn now(&self) -> f64 {
        self.inner.lock().unwrap().now
    }

    /// Returns the current simulation time (same as now())
    /// This is the maximum time the clock has advanced to
    #[allow(dead_code)]
    pub fn elapsed(&self) -> f64 {
        self.now()
    }

    /// Sleeps for the specified duration in simulation time
    pub async fn sleep(&self, duration: f64) {
        if duration <= 0.0 {
            tokio::task::yield_now().await;
            return;
        }
        let wake_time = self.now() + duration;
        self.sleep_until(wake_time).await;
    }

    /// Sleeps until the specified absolute time in simulation time
    pub async fn sleep_until(&self, wake_time: f64) {
        let now = self.now();
        if wake_time <= now {
            tokio::task::yield_now().await;
            return;
        }

        SleepFuture {
            clock: self.inner.clone(),
            wake_time,
            registered: false,
        }
        .await;
    }

    /// Advances the simulation clock to the next scheduled event
    ///
    /// Returns true if time was advanced, false if no events remain
    pub fn advance(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if let Some(Reverse(event)) = inner.wakers.pop() {
            inner.now = event.time;
            event.waker.wake();
            while let Some(Reverse(peek)) = inner.wakers.peek() {
                if peek.time <= inner.now {
                    let Reverse(e) = inner.wakers.pop().unwrap();
                    e.waker.wake();
                } else {
                    break;
                }
            }
            true
        } else {
            false
        }
    }
}

impl Default for SimClock {
    fn default() -> Self {
        Self::new()
    }
}

struct SleepFuture {
    clock: Arc<Mutex<ClockInner>>,
    wake_time: f64,
    registered: bool,
}

impl Future for SleepFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let mut inner = this.clock.lock().unwrap();
        if inner.now >= this.wake_time {
            Poll::Ready(())
        } else {
            if !this.registered {
                inner.wakers.push(Reverse(WakeEvent {
                    time: this.wake_time,
                    waker: cx.waker().clone(),
                }));
                this.registered = true;
            }
            Poll::Pending
        }
    }
}
