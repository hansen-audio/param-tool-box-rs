// Copyright(c) 2023 Hansen Audio.

use std::{ffi::CStr, os::raw::c_char};

use crate::convert::{
    converter::Converter,
    transform::{Display, Transform},
};

//-----------------------------------------------------------------------------
// https://firefox-source-docs.mozilla.org/writing-rust-code/ffi.html
#[no_mangle]
pub unsafe extern "C" fn new_linear(min: f32, max: f32, is_int: bool) -> *mut Converter {
    let c = Converter::new(min, max, None, is_int);
    Box::into_raw(Box::new(c))
}

#[no_mangle]
pub unsafe extern "C" fn new_log(min: f32, max: f32, mid: f32) -> *mut Converter {
    let c = Converter::new(min, max, Some(mid), false);
    Box::into_raw(Box::new(c))
}

#[no_mangle]
pub unsafe extern "C" fn new_list(num_items: i32) -> *mut Converter {
    let max = (num_items - 1) as f32;
    let c = Converter::new(0., max, None, true);
    Box::into_raw(Box::new(c))
}

#[no_mangle]
pub unsafe extern "C" fn delete_converter(converter: *mut Converter) {
    drop(Box::from_raw(converter));
}

#[no_mangle]
pub unsafe extern "C" fn to_physical(converter: &Converter, normalized: f32) -> f32 {
    converter.to_physical(normalized)
}

#[no_mangle]
pub unsafe extern "C" fn to_normalized(converter: &Converter, physical: f32) -> f32 {
    converter.to_normalized(physical)
}

pub type FnStringCallback = extern "C" fn(s: *const u8, len: i32);

#[no_mangle]
pub extern "C" fn to_string(
    converter: &Converter,
    physical: f32,
    precision: usize,
    fn_string: FnStringCallback,
) {
    let display_string = converter.to_display(physical, Some(precision));
    let c_display_string = display_string.as_ptr();
    fn_string(c_display_string, display_string.len() as i32);
}

/// String 's' MUST BE null terminated!
#[no_mangle]
pub unsafe extern "C" fn from_string(converter: &Converter, s: *const c_char) -> f32 {
    // Manually check for NULL case (easy to forget)
    if s.is_null() {
        return 0.;
    }
    let s = CStr::from_ptr(s);
    let tmp = s.to_str();

    converter.from_display(tmp.unwrap_or("0").to_string())
}

#[no_mangle]
pub extern "C" fn num_steps(converter: &Converter) -> i32 {
    if converter.is_int() {
        return (converter.max() - converter.min()) as i32;
    }

    0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_delete() {
        let c_lin = unsafe { new_linear(0., 100., false) };
        unsafe { delete_converter(c_lin) };

        let c_log = unsafe { new_log(0., 100., 25.) };
        unsafe { delete_converter(c_log) };
    }

    #[test]
    fn test() {
        let transformer = Converter::new(0., 100., None, false);
        let val = 0.5;
        unsafe { to_physical(&transformer, val) };
    }
}
