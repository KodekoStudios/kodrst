use napi::{bindgen_prelude::*, sys::*, *};
use kroos::Flake;

use super::{file::File, response::Response};
use crate::{optf, reqf, FlakeExtension};

#[napi(string_enum)]
pub enum Method { DELETE, PATCH, POST, GET, PUT }
impl<'a> From<Method> for &'a str {
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

pub struct Request {
    pub deferred   : Option<Deferred>,
    pub method     : Method          ,
    pub route      : Flake<str>      ,
    pub reason     : Flake<str>      ,
    pub files      : Vec<File>       ,
    pub body       : Flake<str>      ,
}

impl FromNapiValue for Request {
    unsafe fn from_napi_value(env: napi_env, raw_value: napi_value) -> Result<Self> {
        let object = unsafe { JsObject::from_raw(env, raw_value)? };
        let raw = unsafe { object.raw() };

        Ok(Self { 
            deferred: None, 
            method  : reqf!(object, "method", Method   )?                                   , 
            route   : Flake::from_napi_object(env, raw, "route" )?                          ,
            reason  : Flake::from_napi_object(env, raw, "reason") .unwrap_or(Flake::empty()), 
            files   : optf!(object, "files" , Vec<File>)         ?.unwrap_or_default()      , 
            body    : Flake::from_napi_object(env, raw, "body"  ) .unwrap_or(Flake::empty()),
        })
    }
}