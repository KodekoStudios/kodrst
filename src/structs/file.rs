use napi::{bindgen_prelude::*, sys::*, *};
use kroos::Flake;

use crate::required_field;

pub struct File {
    pub data  : Buffer    ,
    pub header: Flake<str>,
}

impl FromNapiValue for File {
    unsafe fn from_napi_value(env: napi_env, value: napi_value) -> Result<Self> {
        let object = unsafe { JsObject::from_raw(env, value)? };
        let leaked = format!(
            "Content-Disposition: form-data; name=\"{field}\"; filename=\"{name}\"\r\n\
            Content-Type: {ctype}\r\n\r\n",
            ctype    = required_field!(object, "content_type", &str)?,
            field    = required_field!(object, "field"       , &str)?,
            name     = required_field!(object, "name"        , &str)?,
        ).leak();

        Ok(File {
            header: unsafe { Flake::<str>::from_raw(leaked as *const str) },
            data: required_field!(object, "data", Buffer)?
        })
    }
}