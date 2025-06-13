use std::alloc::*;
use std::ffi::c_char;
use std::ptr::*;

use crate::{cstr, cstr_len};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Header {           // size | aling | offset [ 16 (0x8), 0x8 ]
    pub name : *const c_char, //    8 |   0x8 |    0x0
    pub value: *const c_char, //    8 |   0x8 |    0x8
}

unsafe impl Send for Header {}
unsafe impl Sync for Header {}

impl Header {
    #[inline(always)]
    pub unsafe fn name(&self) -> &'static str { cstr(self.name) }

    #[inline(always)]
    pub unsafe fn value(&self) -> &'static str { cstr(self.value) }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Response {                  // size | aling | offset [ 32 (0x20), 0x8 ]
    pub headers: (u8 , *const Header), //   16 |   0x8 |   0x 0
    pub body   : *const c_char       , //    8 |   0x8 |   0x10
    pub status :  u16                , //    1 |   0x2 |   0x20
}

unsafe impl Send for Response {}
unsafe impl Sync for Response {}

impl Response {
    #[inline(always)]
    pub unsafe fn headers(&self) -> &'static [Header] {
        &*from_raw_parts::<[Header]>(self.headers.1, self.headers.0 as usize)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn destructor_response(ptr: *mut Response) {
    if ptr.is_null() { return; }

    let res = &*ptr;

    if !res.body.is_null() {
        dealloc(res.body as *mut u8, Layout::from_size_align_unchecked(cstr_len(res.body) + 1, 1));
    }

    if !res.headers.1.is_null() && res.headers.0 > 0 {
        dealloc(res.headers.1 as *mut u8, Layout::array::<Header>(res.headers.0 as usize).unwrap());
    }

    drop_in_place(ptr);
    dealloc(ptr.cast(), Layout::new::<Response>());
}