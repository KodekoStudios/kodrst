use crate::{structs::*, napi_map_err};
use super::client::Client;

use std::{cmp::*, collections::*, sync::{atomic::AtomicU16, Arc}};
use tokio::{sync::mpsc::*, time::*, select};
use rustc_hash::FxHashMap;
use kroos::Rime;

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

#[repr(C)]
pub(crate) struct Scheduler {
    routes : FxHashMap<Rime<AtomicU16, str>, Rime<AtomicU16, str>>,
    buckets: FxHashMap<Rime<AtomicU16, str>, Bucket>              ,
    queue  : BinaryHeap<Command>                                  ,
    tx     : Arc<UnboundedSender<Command>>                        ,
    rx     : UnboundedReceiver<Command>                           ,
}

impl Scheduler {
    #[inline(always)]
    pub fn new(tx: Arc<UnboundedSender<Command>>, rx: UnboundedReceiver<Command>) -> Self {
        Self { 
            buckets: Default::default(),
            routes : Default::default(),
            queue  : Default::default(),
            tx, rx
        }
    }

    #[inline(never)]
    pub async fn run(&mut self, client: Client) {
        let unknown_bucket_id: Rime<AtomicU16, str> = Rime::new("unknown");
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

            
            let now_secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u32;
            let now = Instant::now();

            while let Some(Command::Schedule { when, .. }) = self.queue.peek() {
                if when >= &now { break; }

                if let Command::Schedule { mut request, .. } = self.queue.pop().unwrap() {
                    let suggested_bucket_id = self.routes.get(&request.route).unwrap_or(&unknown_bucket_id);
                    let mut bucket = self.buckets.entry(suggested_bucket_id.clone())
                                                 .or_insert_with(|| Bucket::new(suggested_bucket_id.clone()));

                    if let Some(delay) = bucket.expire_delay(now_secs) {
                        self.tx.send(
                            Command::Schedule { 
                                when: now + Duration::from_secs(delay as u64),
                                request, 
                            }).unwrap();
                        continue;
                    }

                    if let Ok(response) = napi_map_err!(client.send(&request).await) {
                        if let Some(deferred) = request.deferred.take() {
                            if response.status == 429 /* Too Many Request */ {
                                let reset_at = response.headers.iter()
                                    .find_map(|(key, value)| 
                                        key.as_ref().eq("Retry-After")
                                           .then(|| value.parse::<u32>().map(|ms| (ms + 999) / 1000).ok())
                                           .flatten()
                                    )
                                    .unwrap_or(1);

                                bucket.near_limit = true;
                                bucket.reset_at   = now_secs + reset_at;

                                request.deferred = Some(deferred);
                                self.tx.send(
                                    Command::Schedule {
                                        when: now + std::time::Duration::from_secs(reset_at as u64),
                                        request,
                                    }
                                ).unwrap();
    
                                continue;
                            }
                            
                            let mut remaining = None;
                            let mut bucket_id = None;
                            let mut reset_at  = None;

                            for (key, value) in &response.headers {
                                match key.as_ref() {
                                    "X-RateLimit-Remaining" | "x-ratelimit-remaining"
                                        => remaining = value.parse::<u16>().ok(),

                                    "X-RateLimit-Bucket"    | "x-ratelimit-bucket"
                                        => bucket_id = Some(value.as_ref()),

                                    "Retry-After"           | "retry-after"
                                        => reset_at  = value.parse::<u32>().ok().map(|ms| (ms + 999) / 1000),

                                    _ => {}
                                }

                                if remaining.is_some() && bucket_id.is_some() && reset_at.is_some() {
                                    break;
                                }
                            }

                            let remaining = remaining.expect("Missing X-RateLimit-Remaining");
                            let bucket_id = bucket_id.expect("Missing X-RateLimit-Bucket");
                            let reset_at  = reset_at .unwrap_or(1);

                            if bucket_id != suggested_bucket_id.as_ref() {
                                let bucket_id = Rime::<AtomicU16, str>::new(bucket_id);
                                self.routes.insert(request.route, bucket_id.clone());
                                bucket = self.buckets.entry(bucket_id.clone())
                                                    .or_insert_with(|| Bucket::new(bucket_id));
                            }
                            
                            let near = remaining == 0;
                            if bucket.near_limit != near { bucket.near_limit = near; }
                            if near { bucket.reset_at = now_secs + reset_at; }
    
                            deferred.resolve(Box::new(move |_| Ok(response)));
                        }
                    } else {
                        self.tx.send(Command::Schedule { request, when: Instant::now() }).unwrap();
                        continue;
                    }
                    
                } else { break; }
            }
        }
    }
}