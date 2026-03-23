use std::sync::atomic::{AtomicUsize, Ordering};
static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub id: usize,
    pub from: i16,
    pub to: i16,
}

impl Request {
    pub fn new(from: i16, to: i16) -> Request {
        Request {
            id: get_id(),
            from,
            to,
        }
    }
}
