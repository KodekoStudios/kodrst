use std::sync::atomic::AtomicU16;
use kroos::Rime;
use napi::{bindgen_prelude::*, sys::*, *};

pub struct Bucket {
    pub near_limit: bool      ,
    pub reset_at  : u64       ,
    pub id        : Rime<AtomicU16, str>,
}

impl Bucket {
    #[inline(always)]
    pub fn new(id: Rime<AtomicU16, str>) -> Self {
        Self {
            near_limit: false,
            reset_at  : 0,
            id,
        }
    }

    #[inline(always)]
    pub fn maybe_delay(&self, now: u64) -> Option<u64> {
        if self.near_limit && now < self.reset_at {
            Some(self.reset_at - now)
        } else {
            None
        }
    }
}

impl ToNapiValue for Bucket {
    #[rustfmt::skip]
    unsafe fn to_napi_value(env: napi_env, bucket: Self) -> Result<napi_value> {
        let mut object = unsafe { JsObject::from_raw(env, std::ptr::null_mut())? };
        
        object.set_named_property("near_limit",   bucket.near_limit)?;
        object.set_named_property("reset_at"  ,   bucket.reset_at  )?;
        object.set_named_property("id"        , &*bucket.id        )?;

        unsafe { Ok(object.raw()) }
    }
}
