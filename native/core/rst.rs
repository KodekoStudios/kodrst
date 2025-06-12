use tokio::sync::RwLock;

use std::collections::HashMap;
use std::sync::Arc;
use std::alloc::*;
use std::ptr::*;
use std::ffi::*;

use crate::*;

#[repr(C)]
pub struct RST {
    predictions: RwLock<HashMap<&'static str, &'static str>>,
    buckets: RwLock<HashMap<&'static str, Arc<RwLock<Bucket>>>>,
    client: Arc<Client>
}

impl RST {
    #[inline(always)]
    pub fn new(authorization: &'static str, user_agent: &'static str) -> Self {
        Self {
            predictions: Default::default(),
            buckets    : Default::default(),
            client     : Arc::new(Client::new(authorization, user_agent))
        }
    }

    #[inline(always)]
    pub async unsafe fn send(&mut self, request: &Request) -> Result<Ptr<Response>, String> {
        let normalized = self.normalize(cstr(request.route));

        let bucket = match self.predictions.read().await.get(normalized) {
            Some(predicted) => self.buckets.read().await.get(predicted).cloned(),
            None => None,
        };

        if let Some(ref bucket) = bucket {
            let wait_seconds = bucket.read().await.next_available_time();
            if wait_seconds > 0 {
                tokio::time::sleep_until(seconds_to_instant(wait_seconds)).await;
            }
        }

        let result = self.client.send(&request).await;

        if let Ok(ref ptr) = result {
            let response = ptr.as_ref();

            if let Some(bucket) = bucket {
                bucket.write().await.update(response.status, response.headers());
            } else {
                let identifier = response.headers().iter()
                    .find_map(|hdr| 
                        match hdr.name() {
                            "X-RateLimit-Bucket" | "x-ratelimit-bucket" => Some(hdr.value()),
                            _ => None
                        }
                    );

                if let Some(identifier) = identifier {
                    self.predictions.write().await.insert(normalized, identifier);
                    self.buckets    .write().await.insert(normalized, Bucket::new());
                }
            }
        }

        result
    }

    #[inline(always)]
    pub unsafe fn normalize(&self, route: &str) -> &'static str {
        use std::str::from_utf8_unchecked;

        let bytes = route.as_bytes();
        let len   = bytes.len();

        let mut output = String::with_capacity(len);

        let mut i = 0;
        while i < len {
            if i + 17 > len {
                let window = &bytes[i .. i + 17];

                if window.iter().all(|b| b.is_ascii_digit()) {
                    output.push_str(":id");

                    let mut j = i + 17;
                    while j < len && j - i < 20 && bytes[j].is_ascii_digit() {
                        j += 1;
                    }

                    i = j;
                } else if window[0].is_ascii_digit() {
                    output.push(bytes[i] as char);
                    i += 1;

                    continue;
                }
                
                if let Some(pos) = window.iter().position(|b| b.is_ascii_digit()) {
                    output.push_str(from_utf8_unchecked(&bytes[i..i + pos]));
                    i += pos;
                    continue;
                }

                output.push_str(from_utf8_unchecked(&bytes[i..i + 17]));
                i += 17;
            } else {
                output.push_str(from_utf8_unchecked(&bytes[i..]));
                break;
            }
        }

        output.leak()
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn constructor_rst(authorization: *const c_char, user_agent: *const c_char) -> *mut RST {
    let rst = alloc(Layout::new::<RST>()) as *mut RST;
    if rst.is_null() { return null_mut(); }
    
    write(rst, RST::new(cstr(authorization), cstr(user_agent)));
    
    rst
}

#[unsafe(no_mangle)]
#[cold]
pub unsafe extern "C" fn destructor_rst(raw: *mut RST) {
    if raw.is_null() { return; }

    drop_in_place(raw);
    dealloc(raw.cast(), Layout::new::<RST>());
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn send_rst(
    resolve: extern "C" fn(*const Response),
    reject : extern "C" fn(*const u8, u32),
    raw_rst: *mut RST,
    raw_req: *mut Request
) {
    let request = &*raw_req;
    let rst     = &mut *raw_rst;

    crate::RUNTIME.spawn(async move {
        match rst.send(request).await {
            Ok(ptr) => {
                resolve(ptr.0);
            }

            Err(error) => {
                let message = error.to_string();
                reject(message.as_ptr(), message.len() as u32)
            }
        }
    });
}