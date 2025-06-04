#![feature(stmt_expr_attributes, ptr_metadata, trait_alias, str_as_str)]
#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

pub mod structs;
pub mod macros;
pub mod core;

use kroos::Flake;
use napi::{sys::*, *};

pub trait FlakeExtension {
    fn from_napi_object(env: napi_env, raw: napi_value, field: &str) -> Result<Flake<str>>;
    fn from_napi(env: napi_env, value: napi_value) -> Result<Flake<str>>;
    fn clone(&self) -> Flake<str> where Self: AsRef<str> { Flake::new(self.as_ref()) }
    fn empty() -> Flake<str> { 
        unsafe { 
            use std::{alloc::*, ptr::*};
        
            Flake::from_raw(from_raw_parts(alloc(Layout::from_size_align_unchecked(0, 1)), 0))
        } 
    }
}

impl FlakeExtension for Flake<str> {
    fn from_napi_object(env: napi_env, raw: napi_value, field: &str) -> Result<Flake<str>> {
        let c_field = std::ffi::CString::new(field)?;
        
        unsafe {
            let mut value_ptr = std::ptr::null_mut();

            check_status!(
                napi_get_named_property(env, raw, c_field.as_ptr(), &mut value_ptr),
                "Failed to get property with field `{field}`",
            )?;

            Flake::from_napi(env, value_ptr)
        }    
    }

    fn from_napi(env: napi_env, value: napi_value) -> Result<Flake<str>> {
        unsafe {
            let mut needed = 0;
            check_status!(
                napi_get_value_string_utf8(env, value, std::ptr::null_mut(), 0, &mut needed),
                "Failed to get string length"
            )?;

            let layout = std::alloc::Layout::from_size_align_unchecked(needed, 1);
            let ptr = std::alloc::alloc(layout);

            if ptr.is_null() { 
                std::alloc::dealloc(ptr, layout);
                std::alloc::handle_alloc_error(layout);
            }
            
            let mut written = 0;
            check_status!(
                napi_get_value_string_utf8(env, value, ptr as *mut std::os::raw::c_char, needed + 1, &mut written),
                "Failed to read string into heap"
            )?;

            Ok(Flake::from_raw(std::ptr::from_raw_parts::<str>(ptr, written)))
        }
    }
}
