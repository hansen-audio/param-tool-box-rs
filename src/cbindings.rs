// Copyright(c) 2023 Hansen Audio.

use std::ffi::CString;
// use std::slice;
use std::{ffi::CStr, os::raw::c_char};

use crate::param::conversion::Conversion as Converter;
use crate::param::{conversion::Kind, display_handling::DisplayHandling};

//-----------------------------------------------------------------------------
// https://firefox-source-docs.mozilla.org/writing-rust-code/ffi.html
#[no_mangle]
pub unsafe extern "C" fn new_linear(min: f32, max: f32, kind: Kind) -> *mut Converter {
    let c = Converter::new(min, max, None, kind);
    Box::into_raw(Box::new(c))
}

#[no_mangle]
pub unsafe extern "C" fn new_log(min: f32, max: f32, mid: f32) -> *mut Converter {
    let c = Converter::new(min, max, Some(mid), Kind::Float);
    Box::into_raw(Box::new(c))
}

#[no_mangle]
pub unsafe extern "C" fn new_list(num_items: i32) -> *mut Converter {
    let max = (num_items - 1) as f32;
    let c = Converter::new(0., max, None, Kind::Int);
    Box::into_raw(Box::new(c))
}

/*
TODO!!!
#[no_mangle]
pub unsafe extern "C" fn new_list2(s: *const *const c_char, num_items: i32) -> *mut Converter {
    let max = (num_items - 1) as f32;
    let c = Converter::new(0., max, None, true);

    let slice = slice::from_raw_parts(s, num_items as usize);
    for v in slice {
        let s = CStr::from_ptr(*v);
        let tmp = s.to_str();
        println!("{:?}", tmp.unwrap());
    }

    Box::into_raw(Box::new(c))
}
*/

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

/// String 's' guaranteed to be null terminated
pub type FnStringCallback = extern "C" fn(s: *const c_char);

#[no_mangle]
#[allow(temporary_cstring_as_ptr)] // this warning should be ok here
pub extern "C" fn to_string(
    converter: &Converter,
    physical: f32,
    precision: usize,
    fn_string: FnStringCallback,
) {
    let display_string = converter.to_display(physical, Some(precision));
    let c_display_string = CString::new(display_string);
    fn_string(c_display_string.unwrap().as_ptr());
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
    converter.num_steps() as i32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_delete() {
        let c_lin = unsafe { new_linear(0., 100., Kind::Float) };
        unsafe { delete_converter(c_lin) };

        let c_log = unsafe { new_log(0., 100., 25.) };
        unsafe { delete_converter(c_log) };
    }

    #[test]
    fn test() {
        let transformer = Converter::new(0., 100., None, Kind::Float);
        let val = 0.5;
        unsafe { to_physical(&transformer, val) };
    }
}
