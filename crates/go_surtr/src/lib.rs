use std::ffi::{CStr, CString, c_char};
use surtr;
use surtr::options::SurtrOptions;

#[unsafe(no_mangle)]
pub extern "C" fn options_init() -> *mut SurtrOptions {
    let mut new_options = SurtrOptions::default();
    let boxed_options = Box::new(new_options);

    Box::into_raw(boxed_options)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn options_set(inst_ref: *mut SurtrOptions, name: *const c_char, value: bool) {
    let input_str = CStr::from_ptr(name).to_str().unwrap();

    let options_instance = &mut *inst_ref;
    options_instance.set(input_str, value);
}

#[unsafe(no_mangle)]
pub extern "C" fn options_destroy(inst_ref: *mut SurtrOptions) {
    let _ = unsafe { Box::from_raw(inst_ref) };
}

#[unsafe(no_mangle)]
pub extern "C" fn GenerateSurtFromURL(url: *const c_char) -> *const c_char {
    let input_cstr = unsafe { CStr::from_ptr(url) };
    let input = input_cstr.to_str().unwrap().to_string();

    match surtr::surt(Some(&input), None, None) {
        Ok(s) => match CString::new(s) {
            Ok(cstring) => cstring.into_raw(),
            Err(e) => {
                println!("({})", e);
                ::std::ptr::null()
            }
        },
        Err(_) => ::std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn GenerateSurtFromURLWithOptions(url: *const c_char, option_ref: *mut SurtrOptions) -> *const c_char {
    let input_cstr = unsafe { CStr::from_ptr(url) };
    let input = input_cstr.to_str().unwrap().to_string();

    let options = unsafe { &mut *option_ref }.clone();

    match surtr::surt(Some(&input), None, Some(options)) {
        Ok(s) => match CString::new(s) {
            Ok(cstring) => cstring.into_raw(),
            Err(e) => {
                println!("({})", e);
                ::std::ptr::null()
            }
        },
        Err(_) => ::std::ptr::null(),
    }
}