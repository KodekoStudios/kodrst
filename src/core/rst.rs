use crate::{napi_map, structs::{file::File, request::{Method, Request}}, FlakeExtension};
use super::{client::Client, scheduler::*};

use napi::{bindgen_prelude::*, sys::*, *};
use tokio::{sync::{mpsc::*}, time::*};
use kroos::Flake;

pub struct Settings {
    pub authorization: Flake<str>,
    pub user_agent   : Flake<str>,
}

impl FromNapiValue for Settings {
    unsafe fn from_napi_value(env: napi_env, value: napi_value) -> Result<Self> {
        Ok(Settings {
            authorization: Flake::from_napi_object(env, value, "authorization")?,
            user_agent   : Flake::from_napi_object(env, value, "user_agent"   )?,
        })
    }
}

impl TypeName for Settings {
    fn type_name() -> &'static str { "object" }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}

#[napi(js_name = "RST", custom_finalize)]
pub struct RST { tx: UnboundedSender<Command> }

#[napi]
impl RST {
    #[napi(constructor)]
    pub unsafe fn new(settings: Settings) -> Self {
        let (tx, rx) = unbounded_channel();

        tokio::spawn(async {
            let mut scheduler = Scheduler::new(rx);
            scheduler.run(Client::new(settings.authorization, settings.user_agent).unwrap()).await;
        });
        
        Self { tx }
    }

    // #[napi(getter)]
    // pub fn settings(&self, env: Env) -> Result<napi_value> {
    //     let mut object = env.create_object()?;

    //     object.set_named_property("authorization", &*self.client.authorization)?;
    //     object.set_named_property("user_agent"   , &*self.client.user_agent   )?;

    //     unsafe { Ok(object.raw()) }
    // }

    #[napi]
    pub fn dispatch(&self, request: Request) -> Result<()> {
        napi_map!(self.tx.send(Command::Schedule { request, when: Instant::now()} ))?;
        Ok(())
    }

    #[napi]
    pub fn send(&self, env: Env, mut request: Request) -> Result<JsObject> {
        let (deferred, promise) = env.create_deferred()?;
        request.deferred = Some(deferred);
        
        napi_map!(self.tx.send(Command::Schedule { request, when: Instant::now()} ))?;

        Ok(promise)
    }

    #[napi]
    pub fn delete(&self, env: Env, route: JsString, body: Option<JsString>, files: Option<Vec<File>>, reason: Option<JsString>) -> Result<JsObject> {
        let (deferred, promise) = env.create_deferred()?;

        let env = env.raw();
        let request = unsafe {
            Request {
                deferred: Some(deferred),
                method: Method::DELETE, 
                route : Flake::from_napi(env, route.raw())?,
                reason: reason.map(|reason| Flake::from_napi(env, reason.raw())).unwrap_or(Ok(Flake::empty()))?, 
                body  : body.map(|body| Flake::from_napi(env, body.raw())).unwrap_or(Ok(Flake::empty()))?, 
                files : files.unwrap_or_default()
            }
        };

        napi_map!(self.tx.send(Command::Schedule { request, when: Instant::now() }))?;

        Ok(promise)
    }

    #[napi]
    pub fn patch(&self, env: Env, route: JsString, body: Option<JsString>, files: Option<Vec<File>>, reason: Option<JsString>) -> Result<JsObject> {
        let (deferred, promise) = env.create_deferred()?;

        let env = env.raw();
        let request = unsafe {
            Request {
                method: Method::PATCH, 
                route : Flake::from_napi(env, route.raw())?,
                reason: reason.map(|reason| Flake::from_napi(env, reason.raw())).unwrap_or(Ok(Flake::empty()))?, 
                body  : body.map(|body| Flake::from_napi(env, body.raw())).unwrap_or(Ok(Flake::empty()))?, 
                files: files.unwrap_or_default(), 
                deferred: Some(deferred)
            }
        };

        napi_map!(self.tx.send(Command::Schedule { request, when: Instant::now() }))?;

        Ok(promise)
    }

    #[napi]
    pub fn post(&self, env: Env, route: JsString, body: Option<JsString>, files: Option<Vec<File>>, reason: Option<JsString>) -> Result<JsObject> {
        let (deferred, promise) = env.create_deferred()?;
        
        let env = env.raw();
        let request = unsafe {
            Request {
                method: Method::POST, 
                route : Flake::from_napi(env, route.raw())?,
                reason: reason.map(|reason| Flake::from_napi(env, reason.raw())).unwrap_or(Ok(Flake::empty()))?, 
                body  : body.map(|body| Flake::from_napi(env, body.raw())).unwrap_or(Ok(Flake::empty()))?, 
                files: files.unwrap_or_default(), 
                deferred: Some(deferred)
            }
        };

        napi_map!(self.tx.send(Command::Schedule { request, when: Instant::now() }))?;

        Ok(promise)
    }

    #[napi]
    pub fn put(&self, env: Env, route: JsString, body: Option<JsString>, files: Option<Vec<File>>, reason: Option<JsString>) -> Result<JsObject> {
        let (deferred, promise) = env.create_deferred()?;

        let env = env.raw();
        let request = unsafe {
            Request {
                method: Method::PUT, 
                route : Flake::from_napi(env, route.raw())?,
                reason: reason.map(|reason| Flake::from_napi(env, reason.raw())).unwrap_or(Ok(Flake::empty()))?, 
                body  : body.map(|body| Flake::from_napi(env, body.raw())).unwrap_or(Ok(Flake::empty()))?, 
                files: files.unwrap_or_default(), 
                deferred: Some(deferred)
            }
        };

        napi_map!(self.tx.send(Command::Schedule { request, when: Instant::now() }))?;

        Ok(promise)
    }

    #[napi]
    pub fn get(&self, env: Env, route: JsString, body: Option<JsString>, files: Option<Vec<File>>, reason: Option<JsString>) -> Result<JsObject> {
        let (deferred, promise) = env.create_deferred()?;
        
        let env = env.raw();
        let request = unsafe {
            Request {
                method: Method::GET, 
                route : Flake::from_napi(env, route.raw())?,
                reason: reason.map(|reason| Flake::from_napi(env, reason.raw())).unwrap_or(Ok(Flake::empty()))?, 
                body  : body.map(|body| Flake::from_napi(env, body.raw())).unwrap_or(Ok(Flake::empty()))?, 
                files: files.unwrap_or_default(), 
                deferred: Some(deferred)
            }
        };

        napi_map!(self.tx.send(Command::Schedule { request, when: Instant::now() }))?;

        Ok(promise)
    }

    // #[napi]
    // pub async unsafe fn run(&mut self) -> Result<()> {
    //     self.scheduler.run(self.client.clone()).await;
    //     Ok(())
    // }

    #[napi]
    pub fn shutdown(&self) -> Result<()> {
        napi_map!(self.tx.send(Command::Shutdown))
    }
}

impl ObjectFinalize for RST {
    fn finalize(self, _env: Env) -> Result<()> {
        let _ = self.tx.send(Command::Shutdown);
        drop(self);
        Ok(())
    }
}