use std::error::Error;

use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender, error::SendError},
        oneshot::{self, Sender as OneshotSender},
    },
    task::JoinHandle,
};

use crate::controller::request::Request;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub struct ElevatorState {
    dir: Direction,
    pos: i16,
    is_idle: bool,
}

enum Command {
    AddRequest(Request),
    GetStatus { reply: OneshotSender<ElevatorState> },
    Destroy,
}

pub struct Elevator {
    pub id: i8,
    tx: Sender<Command>,
}

impl Elevator {
    pub fn new(id: i8) -> Elevator {
        let (tx, rx) = mpsc::channel(100);
        Self::start(rx);
        Elevator { id, tx }
    }

    fn start(mut rx: Receiver<Command>) -> JoinHandle<()> {
        let mut state = ElevatorState {
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
                            println!("Request Picked: {:?}, State: {:?}", request, state);
                            if start < dest {
                                state.is_idle = false;
                                state.dir = Direction::Up;
                                for floor in start + 1..=dest {
                                    state.pos = floor;
                                    println!(
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
                                    println!(
                                        "Request Processing : {:?}, State: {:?}",
                                        request, state
                                    );
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;
                                }
                            }
                            state.is_idle = true;
                            println!("Request Processed: {:?}, State: {:?}", request, state);
                        }
                        Command::GetStatus { reply } => {
                            reply
                                .send(state)
                                .expect("state communication to be successful");
                        }
                        Command::Destroy => {
                            println!("received destroy: stopping");
                            break;
                        }
                    },
                    None => {
                        println!("worker: all senders dropped, exiting");
                        break;
                    }
                }
            }
        })
    }

    pub async fn stop(self) -> Option<()> {
        self.tx.send(Command::Destroy).await.ok()
    }

    pub async fn add_request(&self, request: Request) -> Result<(), Box<dyn Error>> {
        match self.tx.send(Command::AddRequest(request)).await {
            Ok(_) => Ok(()),
            Err(_) => Err("error adding request".into()),
        }
    }

    pub async fn get_state(&self) -> Result<ElevatorState, Box<dyn Error>> {
        let (tx, rx) = oneshot::channel();
        match self.tx.send(Command::GetStatus { reply: tx }).await {
            Ok(_) => Ok(rx.await.unwrap()),
            Err(_) => Err("unable to get state".into()),
        }
    }
}
