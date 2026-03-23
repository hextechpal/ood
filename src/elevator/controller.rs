mod elevator;
pub mod errors;
pub mod request;

use std::{collections::HashMap, error::Error, sync::Mutex};

use elevator::Elevator;
use futures::future::join_all;

use crate::controller::{elevator::ElevatorState, errors::RequestError, request::Request};

#[allow(dead_code)]
enum Strategy {
    Closest,
}

#[allow(dead_code)]
pub struct Controller {
    elevators: Vec<Elevator>,
    strategy: Strategy,
    mutex: Mutex<()>,
}

#[allow(dead_code)]
impl Controller {
    pub fn new(capacity: usize) -> Controller {
        let elevators = (0..capacity).map(|id| Elevator::new(id)).collect();
        Controller {
            elevators,
            strategy: Strategy::Closest,
            mutex: Mutex::new(()),
        }
    }

    pub async fn request(&self, from: i16, to: i16) -> Result<bool, RequestError> {
        let el = self.select_elevator(from, to).await;
        el.add_request(Request::new(from, to)).await?;
        Ok(true)
    }

    // Lets explore streams here
    pub async fn state(&self) -> HashMap<usize, ElevatorState> {
        let results: Vec<_> = join_all(
            self.elevators
                .iter()
                .map(|el| async move { (el.id, el.get_state().await) }),
        )
        .await;

        results
            .into_iter()
            .filter_map(|(id, res)| res.ok().map(|state| (id, state)))
            .collect()
    }

    async fn select_elevator(&self, _from: i16, _to: i16) -> &Elevator {
        &self.elevators.first().unwrap()
    }
}
