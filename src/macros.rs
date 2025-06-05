#[macro_export]
macro_rules! required_field {
    ($obj:expr, $key:literal, $ty:ty) => {
        $obj.get::<&str, $ty>($key)?
            .ok_or_else(|| napi::Error::from_reason(concat!("Missing `", $key, "` field")))
    };
}

#[macro_export]
macro_rules! optional_field {
    ($obj:expr, $key:literal, $ty:ty) => {
        $obj.get::<&str, $ty>($key)
    };
}

#[macro_export]
macro_rules! napi_map_err {
    ($expr:expr) => {
        $expr.map_err(|e| napi::Error::from_reason(e.to_string()))
    };
}

#[inline(always)]
pub fn usize_to_str_bytes<'a>(n: usize) -> &'a [u8] {
    use std::{io::{Cursor, Write}, ptr::*};

    let mut buffer = [0u8; 20];
    let mut cursor = Cursor::new(&mut buffer[..]);

    write!(cursor, "{n}").unwrap();
    let written = cursor.position() as usize;
    drop(cursor);

    unsafe { &*slice_from_raw_parts(buffer.as_ptr(), written) }
}
