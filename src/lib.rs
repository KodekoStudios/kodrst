#![feature(stmt_expr_attributes, core_intrinsics, ptr_metadata, trait_alias, str_as_str)]
#![allow(internal_features)]
#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

pub mod structs;
pub mod macros;
pub mod core;

use std::{sync::atomic::AtomicU16, alloc::*, ptr::*};
use kroos::{Counter, Flake, Rime};
use napi::{sys::*, *};

pub trait FlakeExtension {
    fn from_napi_object(env: napi_env, raw: napi_value, field: &str) -> Result<Flake<str>>;
    fn from_napi(env: napi_env, value: napi_value) -> Result<Flake<str>>;
    fn clone(&self) -> Flake<str> where Self: AsRef<str> { Flake::new(self.as_ref()) }
    #[inline(always)]
    fn empty() -> Flake<str> { 
        unsafe { 
            Flake::from_raw_parts(alloc(Layout::from_size_align_unchecked(0, 1)), 0)
        } 
    }
}

impl FlakeExtension for Flake<str> {
    fn from_napi_object(env: napi_env, raw: napi_value, field: &str) -> Result<Flake<str>> {
        let c_field = std::ffi::CString::new(field)?;
        
        unsafe {
            let mut value_ptr = null_mut();

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
                napi_get_value_string_utf8(env, value, null_mut(), 0, &mut needed),
                "Failed to get string length"
            )?;

            let layout = Layout::from_size_align_unchecked(needed, 1);
            let ptr = alloc(layout);

            if ptr.is_null() { 
                dealloc(ptr, layout);
                handle_alloc_error(layout);
            }
            
            let mut written = 0;
            check_status!(
                napi_get_value_string_utf8(env, value, ptr as *mut std::os::raw::c_char, needed + 1, &mut written),
                "Failed to read string into heap"
            )?;

            Ok(Flake::from_raw_parts(ptr, written))
        }
    }
}

pub trait RimeExtension {
    fn from_napi_object(env: napi_env, raw: napi_value, field: &str) -> Result<Rime<AtomicU16, str>>;
    fn from_napi(env: napi_env, value: napi_value) -> Result<Rime<AtomicU16, str>>;
    #[inline(always)]
    fn empty() -> Rime<AtomicU16, str> { Rime::new("") }
}

impl RimeExtension for Rime<AtomicU16, str> {
    fn from_napi_object(env: napi_env, raw: napi_value, field: &str) -> Result<Rime<AtomicU16, str>> {
        let c_field = std::ffi::CString::new(field)?;
        
        unsafe {
            let mut value_ptr = null_mut();

            check_status!(
                napi_get_named_property(env, raw, c_field.as_ptr(), &mut value_ptr),
                "Failed to get property with field `{field}`",
            )?;

            Rime::from_napi(env, value_ptr)
        }   
    }

    fn from_napi(env: napi_env, value: napi_value) -> Result<Rime<AtomicU16, str>>  {
        unsafe {
            let mut needed = 0;
            check_status!(
                napi_get_value_string_utf8(env, value, null_mut(), 0, &mut needed),
                "Failed to get string length"
            )?;

            let layout = Layout::from_size_align_unchecked(needed + 2, 2);
            let ptr = alloc(layout);

            if ptr.is_null() { 
                dealloc(ptr, layout);
                handle_alloc_error(layout);
            }

            let counter_ptr = ptr as *mut AtomicU16;
            write(counter_ptr, Counter::new());
            
            // rustc interpreted the 2 as i32 instead of isize, which causes
            // panic in the compiler when performing pointer arithmetic.
            // Thanks Rust! - Kaffee

            let inner_ptr = std::intrinsics::offset(ptr, 2isize);
            let mut written = 0;
            check_status!(
                napi_get_value_string_utf8(env, value, inner_ptr as *mut std::os::raw::c_char, needed + 1, &mut written),
                "Failed to read string into heap"
            )?;

            Ok(Rime::from_raw_parts(counter_ptr, inner_ptr, written))
        }
    }
    
    fn empty() -> Rime<AtomicU16, str> { Rime::new("") }
}