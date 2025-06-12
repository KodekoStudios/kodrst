use std::sync::atomic::{Ordering, AtomicBool, AtomicU32};
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::{Header, now_seconds_u32};

#[repr(C)]
pub struct Bucket {
    reset_at  : AtomicU32    ,
    near_limit: AtomicBool   ,
}

impl Bucket {
    #[inline(always)]
    pub fn new() -> Arc<RwLock<Bucket>>  {
        Arc::new(
            RwLock::new(
                Bucket {
                    reset_at  : AtomicU32::new(0),
                    near_limit: AtomicBool::new(false)
                }
            )
        )
    }

    #[inline(always)]
    pub unsafe fn update(&mut self, status: u16, headers: &[Header]) {
         if status == 429 {
            self.near_limit.store(true, Ordering::Release);
            self.reset_at.swap(
                headers.iter()
                    .find_map(|hdr|
                        match hdr.name() {
                            "Retry-After" | "retry-after" => hdr.value().parse::<u32>().map(|ms| (ms + 999) / 1000).ok(),
                            _ => None
                        }
                    ).unwrap_or(1)
                    + now_seconds_u32(), 
                Ordering::Release
            );

            return;
        }

        let mut retry_after = None;
        let mut remaining   = None;

        for hdr in headers {
            match hdr.name() {
                "X-RateLimit-Remaining" | "x-ratelimit-remaining"
                    => remaining   = hdr.value().parse::<u16>().ok(),

                "Retry-After"           | "retry-after"
                    => retry_after = hdr.value().parse::<u32>().ok().map(|ms| (ms + 999) / 1000),

                _ => {}
            }

            if remaining.is_some() && retry_after.is_some() {
                break;
            }
        }

        let near = remaining.expect("Missing X-RateLimit-Remaining header") == 0;
        
        if near { 
            self.reset_at.swap(now_seconds_u32() + retry_after.unwrap_or(1), Ordering::Release);
        }

        if near != self.near_limit.load(Ordering::Acquire) {
            self.near_limit.fetch_not(Ordering::Release);
        }
    }

    #[inline(always)]
    pub fn next_available_time(&self) -> u32 {
        let reset_at = self.reset_at.load(Ordering::Acquire);
        let now_secs = now_seconds_u32();

        if self.near_limit.load(Ordering::Acquire) && reset_at.gt(&now_secs) {
            return reset_at - now_secs;
        }

        now_secs
    }
}