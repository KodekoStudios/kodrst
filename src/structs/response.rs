use std::collections::HashMap;
use kroos::Flake;
use napi::*;

use crate::napi_map_err;

#[napi]
#[repr(C)]
pub struct Response {
    pub(crate) headers: HashMap<Flake<str>, Flake<str>>,
    pub(crate) body   : Flake<str>                     ,
    #[napi(writable = false)]
    pub status: u16,
}

#[napi]
impl Response {
    #[inline(always)]
    pub fn new(status: u16, headers: HashMap<Flake<str>, Flake<str>>, body: Flake<str>) -> Self {
        Self { status, headers, body }
    }

    #[napi(getter)]
    #[cold]
    pub fn headers(&self, env: Env) -> Result<JsObject> {
        let mut object = env.create_object()?;

        for (key, value) in self.headers.iter() {
            object.set_named_property(&**key, &**value)?;
        }

        Ok(object)
    } 

    #[napi(js_name = "body_as_json")]
    #[inline(always)]
    pub fn body_as_json(&self) -> Result<serde_json::Value> {
        napi_map_err!(serde_json::from_str(&*self.body))
    }

    #[napi(js_name = "body_as_str")]
    #[inline(always)]
    pub fn body_as_str(&self) -> String {
        self.body.to_string()
    }
}