use std::intrinsics::*;
use std::alloc::*;
use std::ptr::*;
use std::ffi::*;

use crate::cstr;

#[repr(C)]
#[derive(Debug)]
pub struct File {
    pub buffer: &'static [u8],
    pub header: &'static str,
}

impl File {
    pub fn new(
        buffer: &'static [u8],
        r#type: &'static str,
        field : &'static str,
        name  : &'static str,
    ) -> File {
        let header = format!(
            "Content-Disposition: form-data; name=\"{field}\"; filename=\"{name}\"\r\n\
            Content-Type: {}\r\n\r\n",
            r#type
        ).leak();

        File { buffer, header }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn alloc_file_slice(len: usize) -> *mut File {
    if len == 0 { return null_mut(); }

    let layout = Layout::from_size_align_unchecked(unchecked_mul(size_of::<File>(), len),align_of::<File>());
    let ptr = alloc(layout) as *mut File;

    if ptr.is_null() {
        null_mut()
    } else {
        ptr
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc_file_slice(raw: *mut File, len: usize) {
    if raw.is_null() || len == 0 { return; }

    let layout = Layout::array::<File>(len).unwrap();
    let slice = &mut *from_raw_parts_mut::<[File]>(raw, len);

    for file in slice // Frees `Box<str>`
        { std::ptr::drop_in_place(file); } 

    dealloc(raw.cast(), layout);
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn alloc_file(
    raw   : *mut File,
    buffer: *const u8    , len: usize,
    r#type: *const c_char,
    field : *const c_char,
    name  : *const c_char,
) -> *mut File {
    let buffer = &*from_raw_parts::<[u8]>(buffer, len);

    write(raw, 
        File::new(
            Box::leak(buffer.to_vec().into_boxed_slice()), 
            cstr(r#type), 
            cstr(field ), 
            cstr(name  )
        )
    );

    raw
}
