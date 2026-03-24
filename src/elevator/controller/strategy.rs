use futures::{StreamExt, stream::FuturesUnordered};

use crate::controller::elevator::Elevator;

pub enum Strategy {
    First, // a really really dumb strategy
    Closest,
    #[allow(dead_code)]
    Quickest,
}

impl Strategy {
    pub async fn select_elevator(&self, elevators: &Vec<Elevator>, source: i16) -> Option<usize> {
        match self {
            Strategy::Closest => {
                if elevators.is_empty() {
                    return None;
                }

                let mut futures = elevators
                    .iter()
                    .map(|e| e.get_state())
                    .collect::<FuturesUnordered<_>>();

                let mut best: Option<(i16, usize)> = None;

                while let Some(state) = futures.next().await {
                    let s = state.unwrap();
                    let dist = (s.pos - source).abs();

                    match best {
                        Some((best_dist, _)) => {
                            if dist < best_dist {
                                best = Some((dist, s.id))
                            }
                        }
                        None => best = Some((dist, s.id)),
                    }
                }
                best.map(|(_, selected)| selected)
            }
            Strategy::First => elevators.first().map(|e| e.id),
            Strategy::Quickest => todo!(),
        }
    }
}
