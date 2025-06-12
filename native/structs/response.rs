use std::alloc::*;
use std::ptr::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Header {              // size | aling | offset [ 32 (0x20), 0x8 ]
    pub name : (u8 , *const u8), //   16 |   0x8 |   0x 0
    pub value: (u32, *const u8), //   16 |   0x8 |   0x10
}

unsafe impl Send for Header {}
unsafe impl Sync for Header {}

impl Header {
    #[inline(always)]
    pub unsafe fn name(&self) -> &'static str {
        &*from_raw_parts::<str>(self.name.1, self.name.0 as usize)
    }

    #[inline(always)]
    pub unsafe fn value(&self) -> &'static str {
        &*from_raw_parts::<str>(self.value.1, self.value.0 as usize)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Response {                  // size | aling | offset [ 40 (0x28), 0x8 ]
    pub headers: (u8 , *const Header), //   16 |   0x8 |   0x 0
    pub body   : (u32, *const u8    ), //   16 |   0x8 |   0x10
    pub status :  u16                , //    1 |   0x2 |   0x20
    pub _pad   : [u8; 6]             , //    6 |   0x1 |   0x22
}

unsafe impl Send for Response {}
unsafe impl Sync for Response {}

impl Response {
    #[inline(always)]
    pub unsafe fn headers(&self) -> &'static [Header] {
        &*from_raw_parts::<[Header]>(self.headers.1, self.headers.0 as usize)
    }

    #[inline(always)]
    pub unsafe fn body(&self) -> &'static str {
        &*from_raw_parts::<str>(self.body.1, self.body.0 as usize)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn headers_len(ptr: *const Response) -> u8 {
    (*ptr).headers.0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn headers(ptr: *const Response) -> *const Header {
    (*ptr).headers.1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn destructor_response(ptr: *mut Response) {
    if ptr.is_null() { return; }

    let res = &*ptr;

    if !res.body.1.is_null() && res.body.0 > 0 {
        dealloc(res.body.1.cast_mut(), Layout::from_size_align_unchecked(res.body.0 as usize + 1, 1));
    }

    if !res.headers.1.is_null() && res.headers.0 > 0 {
        dealloc(res.headers.1 as *mut u8, Layout::array::<Header>(res.headers.0 as usize).unwrap());
    }

    drop_in_place(ptr);
    dealloc(ptr.cast(), Layout::new::<Response>());
}