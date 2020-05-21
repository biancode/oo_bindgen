use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub struct StringClass {
    value: CString,
}

impl StringClass {
    fn new() -> Self {
        Self{ value: CString::new("").unwrap() }
    }
}

#[no_mangle]
pub unsafe extern "C" fn string_new() -> *mut StringClass {
    let string_class = Box::new(StringClass::new());
    Box::into_raw(string_class)
}

#[no_mangle]
pub unsafe extern "C" fn string_destroy(string_class: *mut StringClass) {
    if !string_class.is_null() {
        Box::from_raw(string_class);
    }
}

#[no_mangle]
pub unsafe extern "C" fn string_echo(string_class: *mut StringClass, value: *const c_char) -> *const c_char {
    let mut string_class = string_class.as_mut().unwrap();
    string_class.value = CStr::from_ptr(value).to_owned();
    string_class.value.as_ptr()
}