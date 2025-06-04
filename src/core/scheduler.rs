use crate::{napi_map, structs::{bucket::Bucket, request::Request}};
use super::client::Client;

use std::{cmp::*, collections::*, sync::{atomic::AtomicU16, Arc}};
use tokio::{sync::mpsc::*, time::*, select};
use rustc_hash::FxHashMap;
use kroos::{Flake, Rime};

pub enum Command {
    Schedule { request: Request, when: Instant },
    Shutdown,
}

impl Command {
    fn when(&self) -> Option<&tokio::time::Instant> {
        match self {
            Command::Schedule { when, .. } => Some(when),
            Command::Shutdown => None
        }
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool { self.when() == other.when() }
}

impl Eq for Command {}

impl PartialOrd for Command {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(other.when().cmp(&self.when())) }
}

impl Ord for Command {
    fn cmp(&self, other: &Self) -> Ordering { other.when().cmp(&self.when()) }
}

#[allow(dead_code)]
pub(crate) struct Scheduler {
    routes : FxHashMap<Flake<str>, Rime<AtomicU16, str>>,
    buckets: FxHashMap<Rime<AtomicU16, str>, Bucket>    ,
    queue  : BinaryHeap<Command>                        ,
    rx     : UnboundedReceiver<Command>                 ,
}

impl Scheduler {
    #[inline(always)]
    pub fn new(rx: UnboundedReceiver<Command>) -> Self {
        Self { 
            buckets: Default::default(),
            routes : Default::default(),
            queue  : Default::default(), 
            rx
        }
    }

    pub async fn run(&mut self, client: Client) {
        let client = Arc::new(client);
        loop {
            if let Some(next) = self.queue.peek() {
                if matches!(next, Command::Shutdown) { break };
                select! {
                    maybe = self.rx.recv() => {
                        match maybe {
                            Some(command) => {
                                if matches!(command, Command::Shutdown) { break; } 
                                self.queue.push(command)
                            },
                            _ => break,
                        }
                    },
                    _ = sleep_until(*next.when().expect("Cannot be Kill")) => { }
                };
            } else {
                match self.rx.recv().await {
                    Some(command) => {
                        if matches!(command, Command::Shutdown) { break; } 
                        self.queue.push(command)
                    },
                    _ => break,
                }
            }

            let now = Instant::now();
            while let Some(Command::Schedule { when, .. }) = self.queue.peek() {
                if when >= &now { break; }

                if let Command::Schedule { mut request, .. } = self.queue.pop().unwrap() {
                    // if self.routes.contains_key(&request.route) {
                    //     let bucket_id = self.routes.get(&request.route).unwrap();
                    //     let bucket = self.buckets.entry(bucket_id.clone())
                    //                              .or_insert_with(|| Bucket::new(bucket_id.clone()));

                    //     let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
                    //     match bucket.maybe_delay(now_ms) {
                    //         Some(_delay) => {
                    //             // self.rx
                    //         }
                    //         None => {}
                    //     }
                    // }

                    let deferred = request.deferred.take();
                    let response = napi_map!(client.send(request).await);

                    
                    match deferred {
                        Some(deferred) => {
                            match response {
                                Ok(_) => {
                                    deferred.resolve(Box::new(move |_| response));
                                },
                                Err(error) => deferred.reject(error),
                            };
                        },
                        None => {}
                    }
                } else { break; }
            }
        }
    }
}