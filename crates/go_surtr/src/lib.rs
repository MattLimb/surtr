use std::ffi::{CStr, CString, c_char};
use surtr;
use surtr::error::SurtrError;
use surtr::options::SurtrOptions;


#[repr(C)]
pub struct Results {
    output: *const c_char,
    error: *const c_char,
}

impl Results {
    pub fn from_string(s: String) -> Self {
        Self {
            output: CString::new(s).unwrap().into_raw(),
            error: ::std::ptr::null(),
        }
    }

    pub fn from_error(e: String) -> Self {
        Self {
            output: ::std::ptr::null(),
            error: CString::new(e).unwrap().into_raw(),
        }
    }
}


fn surt(url: &str, options: Option<SurtrOptions>) -> Results {
    match surtr::surt(Some(url), None, options) {
        Ok(s) => Results::from_string(s),
        Err(SurtrError::Error(e)) => Results::from_error(e.to_string()),
        Err(SurtrError::UrlParseError(e)) => Results::from_error(e.to_string()),
        Err(SurtrError::NoSchemeFoundError) => Results::from_error("Excpected URL Scheme - None Found".to_string()),
        Err(SurtrError::CanonicalizerError(e)) => Results::from_error(e.to_string())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn init_options() -> *mut SurtrOptions {
    let new_options = SurtrOptions::default();
    let boxed_options = Box::new(new_options);

    Box::into_raw(boxed_options)
}

#[unsafe(no_mangle)]
pub extern "C" fn destroy_options(inst_ref: *mut SurtrOptions) {
    let _ = unsafe { Box::from_raw(inst_ref) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_option(inst_ref: *mut SurtrOptions, name: *const c_char, value: bool) {
    let input_str = CStr::from_ptr(name).to_str().unwrap();

    let options_instance = &mut *inst_ref;
    options_instance.set(input_str, value);
}

#[unsafe(no_mangle)]
pub extern "C" fn generate_surt(url: *const c_char) -> Results {
    let input_cstr = unsafe { CStr::from_ptr(url) };
    let input = input_cstr.to_str().unwrap().to_string();

    surt(&input, None)
}

#[unsafe(no_mangle)]
pub extern "C" fn generate_surt_with_options(url: *const c_char, option_ref: *mut SurtrOptions) -> Results {
    let input_cstr = unsafe { CStr::from_ptr(url) };
    let input = input_cstr.to_str().unwrap().to_string();

    let options = unsafe { &mut *option_ref }.clone();

    surt(&input, Some(options))
}

#[unsafe(no_mangle)]
pub extern "C" fn generate_surt_error(_url: *const c_char) -> Results {
    Results::from_error("Some Error".to_string())
}
