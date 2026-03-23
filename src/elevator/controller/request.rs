#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Request {
    pub from: i16,
    pub to: i16,
}

impl Request {
    pub fn new(from: i16, to: i16) -> Request {
        Request { from, to }
    }
}
