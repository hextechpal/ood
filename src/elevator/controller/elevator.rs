use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot::{self, Sender as OneshotSender},
    },
    task::JoinHandle,
};
use tracing::{debug, info, warn};

use crate::controller::{
    errors::{RequestError, StateError},
    request::Request,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub struct ElevatorState {
    pub id: usize,
    pub dir: Direction,
    pub pos: i16,
    pub is_idle: bool,
}

enum Command {
    AddRequest(Request),
    GetStatus {
        reply: OneshotSender<ElevatorState>,
    },
    #[allow(dead_code)]
    Destroy,
}

pub struct Elevator {
    pub id: usize,
    tx: Sender<Command>,
}

impl Elevator {
    pub fn new(id: usize) -> Elevator {
        let (tx, rx) = mpsc::channel(100);
        Self::start(id, rx);
        Elevator { id, tx }
    }

    fn start(id: usize, mut rx: Receiver<Command>) -> JoinHandle<()> {
        let mut state = ElevatorState {
            id,
            pos: 0,
            dir: Direction::Up,
            is_idle: true,
        };

        tokio::task::spawn(async move {
            loop {
                let msg = rx.recv().await;
                match msg {
                    Some(cmd) => match cmd {
                        Command::AddRequest(request) => {
                            let start = state.pos;
                            let dest = request.to;
                            debug!("Request Picked: {:?}, State: {:?}", request, state);
                            if start < dest {
                                state.is_idle = false;
                                state.dir = Direction::Up;
                                for floor in start + 1..=dest {
                                    state.pos = floor;
                                    debug!(
                                        "Request Processing : {:?}, State: {:?}",
                                        request, state
                                    );
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                            } else if start > dest {
                                state.is_idle = false;
                                state.dir = Direction::Down;
                                for floor in (dest..start).rev() {
                                    state.pos = floor;
                                    debug!(
                                        "Request Processing : {:?}, State: {:?}",
                                        request, state
                                    );
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                            }
                            state.is_idle = true;
                            info!("Request Processed: {:?}, State: {:?}", request, state);
                        }
                        Command::GetStatus { reply } => {
                            reply
                                .send(state)
                                .expect("state communication to be successful");
                        }
                        _ => {
                            warn!("received destroy: stopping");
                            break;
                        }
                    },
                    None => {
                        warn!("worker: all senders dropped, exiting");
                        break;
                    }
                }
            }
        })
    }

    pub async fn _stop(self) -> Option<()> {
        self.tx.send(Command::Destroy).await.ok()
    }

    pub async fn add_request(&self, request: Request) -> Result<(), RequestError> {
        match self.tx.send(Command::AddRequest(request.clone())).await {
            Ok(_) => Ok(()),
            Err(_) => Err(RequestError { request }),
        }
    }

    pub async fn get_state(&self) -> Result<ElevatorState, StateError> {
        let (tx, rx) = oneshot::channel();
        match self.tx.send(Command::GetStatus { reply: tx }).await {
            Ok(_) => Ok(rx.await.unwrap()),
            Err(_) => Err(StateError {
                elevator_id: self.id,
            }),
        }
    }
}
