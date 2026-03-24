mod elevator;
pub mod errors;
pub mod request;
mod strategy;

use std::collections::HashMap;

use elevator::Elevator;
use futures::future::join_all;
use tracing::info;

use crate::controller::{
    elevator::ElevatorState, errors::RequestError, request::Request, strategy::Strategy,
};

pub struct Controller {
    elevators: Vec<Elevator>,
    strategy: Strategy,
    // mutex: Mutex<()>,
}

#[allow(dead_code)]
impl Controller {
    pub fn new(capacity: usize) -> Controller {
        let elevators = (0..capacity).map(|id| Elevator::new(id)).collect();
        Controller {
            elevators,
            strategy: Strategy::Closest,
            // mutex: Mutex::new(()),
        }
    }

    pub async fn request(&self, from: i16, to: i16) -> Result<bool, RequestError> {
        let request = Request::new(from, to);
        // let _lock = self.mutex.lock().expect("should acquire lock");
        match self.strategy.select_elevator(&self.elevators, from).await {
            Some(id) => {
                info!("elevator selected: {}", id);
                let el = self.elevators.iter().find(|&e| e.id == id).unwrap();
                el.add_request(request).await?;
                Ok(true)
            }

            None => Err(RequestError { request }),
        }
    }

    pub async fn state(&self) -> HashMap<usize, ElevatorState> {
        let results: Vec<_> = join_all(self.elevators.iter().map(|el| el.get_state())).await;

        results
            .into_iter()
            .filter_map(|res| res.ok().map(|state| (state.id, state)))
            .collect()
    }
}
