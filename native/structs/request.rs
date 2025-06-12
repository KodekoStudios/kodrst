use std::alloc::*;
use std::ptr::*;
use std::ffi::*;

use crate::*;

static DELETE: &'static str = "DELETE";
static PATCH : &'static str = "PATCH" ;
static POST  : &'static str = "POST"  ;
static PUT   : &'static str = "PUT"   ;
static GET   : &'static str = "GET"   ;

#[repr(C)]
pub struct Request {
    pub files : &'static [File],
    pub method: &'static str,
    pub reason: *const c_char,
    pub route : *const c_char,
    pub body  : *const c_char,
}

unsafe impl Send for Request {}
unsafe impl Sync for Request {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn constructor_request(
    method: *const c_char,
    route : *const c_char,
    files : *const File,
    len   : usize,
    body  : *const c_char,
    reason: *const c_char,
) -> *mut Request {
    if method.is_null() || route.is_null() {
        return null_mut();
    }

    let method = match cstr(method) {
        "DELETE" => DELETE,
        "PATCH"  => PATCH ,
        "POST"   => POST  ,
        "PUT"    => PUT   ,
        "GET"    => GET   ,
        _ => unreachable!()
    };

    let files = if files.is_null() || len == 0 { &[] } else {
        &*from_raw_parts::<[File]>(files, len)
    };

    let raw = alloc(Layout::new::<Request>()) as *mut Request;
    if raw.is_null() { return null_mut(); }

    write(raw, Request { files, method, reason, route, body });

    raw
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn destructor_request(raw: *mut Request) {
    if raw.is_null() { return; }

    drop_in_place(raw);
    dealloc(raw.cast(), Layout::new::<Request>());
}