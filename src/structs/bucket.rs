use std::sync::atomic::AtomicU16;
use kroos::Rime;
use napi::{bindgen_prelude::*, sys::*, *};

pub struct Bucket {
    pub id        : Rime<AtomicU16, str>,
    pub reset_at  : u32 ,
    pub near_limit: bool,
}

impl Bucket {
    #[inline(always)]
    pub fn new(id: Rime<AtomicU16, str>) -> Self {
        Self { near_limit: false, reset_at: 0, id }
    }

    #[inline(always)]
    pub fn expire_delay(&self, when: u32) -> Option<u32> {
        (self.near_limit & (when < self.reset_at))
            .then_some(self.reset_at.wrapping_sub(when))
    }
}

impl ToNapiValue for Bucket {
    unsafe fn to_napi_value(env: napi_env, bucket: Self) -> Result<napi_value> {
        let mut object = unsafe { JsObject::from_raw(env, std::ptr::null_mut())? };
        
        object.set_named_property("near_limit", bucket.near_limit )?;
        object.set_named_property("reset_at"  , bucket.reset_at   )?;
        object.set_named_property("id"        , bucket.id.as_ref())?;

        unsafe { Ok(object.raw()) }
    }
}
