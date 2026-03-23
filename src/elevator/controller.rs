mod elevator;
pub mod request;

use std::{collections::HashMap, error::Error, sync::Mutex};

use elevator::Elevator;

use crate::controller::{
    elevator::{Direction, ElevatorState},
    request::Request,
};

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
    pub fn new(capacity: i8) -> Controller {
        let elevators = (0..capacity).map(|id| Elevator::new(id)).collect();
        Controller {
            elevators,
            strategy: Strategy::Closest,
            mutex: Mutex::new(()),
        }
    }

    pub async fn request(&self, from: i16, to: i16) -> Result<bool, Box<dyn Error>> {
        let el = self.select_elevator(from, to).await;
        match el.add_request(Request::new(from, to)).await {
            Ok(_) => Ok(true),
            Err(_) => Err("some error".into()),
        }
    }

    // Lets explore streams here
    pub async fn state(&self) -> HashMap<i8, Option<ElevatorState>> {
        todo!()
    }

    async fn select_elevator(&self, _from: i16, _to: i16) -> &Elevator {
        &self.elevators.first().unwrap()
    }
}
