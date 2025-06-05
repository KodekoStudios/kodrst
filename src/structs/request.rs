use napi::{bindgen_prelude::*, sys::*, *};
use std::sync::atomic::AtomicU16;
use kroos::{Flake, Rime};

use crate::{FlakeExtension, RimeExtension, optional_field, required_field};
use super::{file::File, response::Response};

#[napi(string_enum)]
pub enum Method { DELETE, PATCH, POST, GET, PUT }
impl<'a> From<Method> for &'a str {
    #[inline(always)]
    fn from(value: Method) -> &'a str {
        match value {
            Method::DELETE => "DELETE",
            Method::PATCH  => "PATCH" ,
            Method::POST   => "POST"  ,
            Method::PUT    => "PUT"   ,
            Method::GET    => "GET"   ,
        }
    }
}

pub type Deferred = JsDeferred<Response, Box<dyn FnOnce(Env) -> Result<Response> + Send + 'static>>;

#[repr(C)]
pub struct Request {
    pub route      : Rime<AtomicU16, str>, // 24 | 0x18
    pub files      : Vec<File>           , // 24 | 0x18
    pub deferred   : Option<Deferred>    , // 16 | 0x10
    pub reason     : Flake<str>          , // 16 | 0x10
    pub body       : Flake<str>          , // 16 | 0x10
    pub method     : Method              , // 1
}

impl FromNapiValue for Request {
    unsafe fn from_napi_value(env: napi_env, raw_value: napi_value) -> Result<Self> {
        let object = unsafe { JsObject::from_raw(env, raw_value)? };
        let raw = unsafe { object.raw() };
        
        Ok(Self { 
            reason  : Flake::from_napi_object(env, raw, "reason") .unwrap_or(Flake::empty()), 
            body    : Flake::from_napi_object(env, raw, "body"  ) .unwrap_or(Flake::empty()),
            route   : Rime::from_napi_object (env, raw, "route" )?                          ,
            files   : optional_field!(object, "files" , Vec<File>)         ?.unwrap_or_default()      , 
            method  : required_field!(object, "method", Method   )         ?                          , 
            deferred: None, 
        })
    }
}

unsafe impl Send for Request {}
unsafe impl Sync for Request {}