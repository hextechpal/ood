use std::fmt;

use crate::controller::request::Request;

#[derive(Debug, Clone)]
pub struct RequestError {
    pub request: Request,
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error processing request: {}", self.request.id)
    }
}

#[derive(Debug, Clone)]
pub struct StateError {
    pub elevator_id: usize,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error fetching state: {}", self.elevator_id)
    }
}
