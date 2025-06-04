use napi::{bindgen_prelude::*, sys::*, *};
use kroos::Flake;

use crate::reqf;

pub struct File {
    pub header: Flake<str>,
    pub data  : Buffer    ,
}

impl FromNapiValue for File {
    unsafe fn from_napi_value(env: napi_env, value: napi_value) -> Result<Self> {
        let object = unsafe { JsObject::from_raw(env, value)? };
        let leaked = format!(
            "Content-Disposition: form-data; name=\"{field}\"; filename=\"{name}\"\r\n\
            Content-Type: {ctype}\r\n\r\n",
            ctype    = reqf!(object, "content_type", &str)?,
            field    = reqf!(object, "field"       , &str)?,
            name     = reqf!(object, "name"        , &str)?,
        ).leak();

        Ok(File {
            header: unsafe { Flake::<str>::from_raw(leaked as *const str) },
            data: reqf!(object, "data", Buffer)?
        })
    }
}