#![feature(core_intrinsics, ptr_as_ref_unchecked, ptr_metadata)]
#![allow(unsafe_op_in_unsafe_fn, internal_features)]

mod structs;
mod core;

pub use structs::*;
pub use core::*;

#[macro_export]
macro_rules! map_err {
    ($expr:expr) => {
        $expr.map_err(|e| e.to_string())
    };
}

use tokio::runtime::Runtime;
use once_cell::sync::Lazy;

use std::ffi::c_char;

pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Failed to create Tokio Runtime"));

pub struct Ptr<T>(pub *const T);

impl<T> Ptr<T> {
    #[inline(always)]
    pub fn as_ref(&self) -> &T { unsafe { &*self.0} }
}

unsafe impl<T> Send for Ptr<T> {}
unsafe impl<T> Sync for Ptr<T> {}

#[inline(always)]
pub(crate) unsafe fn leak_as_cstr(s: &str) -> &'static str {
    use std::alloc::*;
    use std::ptr::*;

    let len = s.len();

    let raw_cstr = alloc(Layout::from_size_align_unchecked(len + 1, 1));
    copy_nonoverlapping(s.as_ptr(), raw_cstr, len);
    *raw_cstr.add(len) = 0;

    &*from_raw_parts(raw_cstr, len + 1)
}

#[inline(always)]
pub(crate) unsafe fn cstr(ptr: *const c_char) -> &'static str {
    use std::ptr::from_raw_parts;

    &*from_raw_parts(ptr as *const u8, cstr_len(ptr))
}

#[inline(always)]
pub(crate) fn cstr_len(ptr: *const c_char) -> usize {
    let mut len = 0;

    while unsafe { *ptr.add(len) } != 0
        { len += 1; } // don't complain, this is readable.

    len
}

#[inline(always)]
pub fn usize_to_bytes<'a>(n: usize) -> &'a [u8] {
    use std::{io::{Cursor, Write}, ptr::*};

    let mut buffer = [0u8; 20];
    let mut cursor = Cursor::new(&mut buffer[..]);

    write!(cursor, "{n}").unwrap();
    let written = cursor.position() as usize;
    drop(cursor);

    unsafe { &*slice_from_raw_parts(buffer.as_ptr(), written) }
}

#[inline(always)]
pub fn seconds_to_instant(secs: u32) -> tokio::time::Instant {
    use tokio::time::*;

    let now_secs = now_seconds_u32();

    if secs >= now_secs {
        return Instant::now() + Duration::from_secs((secs - now_secs) as u64);
    }
    
    Instant::now()
}

#[inline(always)]
pub fn now_seconds_u32() -> u32 {
    use std::time::*;

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
         as u32
}