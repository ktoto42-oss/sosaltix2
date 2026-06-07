use core::{future::Future, pin::Pin};
use alloc::boxed::Box;

pub struct Task {
    pub(crate) id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

use core::task::{Context, Poll};

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

pub mod executor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);
use core::sync::atomic::{AtomicU64, Ordering};

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct SimpleQueue<T, const N: usize> {
    buffer: [Option<T>; N],
    head: usize,
    tail: usize,
}

impl<T: Copy, const N: usize> SimpleQueue<T, N> {
    pub const fn new() -> Self {
        Self {
            buffer: [None; N],
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, item: T) -> Result<(), ()> {
        let next_tail = (self.tail + 1) % N;
        if next_tail == self.head {
            return Err(());
        }
        self.buffer[self.tail] = Some(item);
        self.tail = next_tail;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.head == self.tail {
            return None;
        }
        let item = self.buffer[self.head].take();
        self.head = (self.head + 1) % N;
        item
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }
}