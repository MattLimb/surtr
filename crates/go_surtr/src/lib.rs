use std::ffi::{CStr, CString, c_char};
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
    match surtr::surt(url, options) {
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

/// # Safety
/// 
/// This function is unsafe because it takes a pointer to a SurtrOptions struct and returns a pointer to a SurtrOptions struct.
/// The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the SurtrOptions struct is not null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn destroy_options(inst_ref: *mut SurtrOptions) {
    let _ = Box::from_raw(inst_ref);
}

/// # Safety
/// 
/// This function is unsafe because it takes a pointer to a SurtrOptions struct and a pointer to a c_char and a bool.
/// The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the pointer to the c_char is valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_option(inst_ref: *mut SurtrOptions, name: *const c_char, value: bool) {
    let input_str = CStr::from_ptr(name).to_str().unwrap();

    let options_instance = &mut *inst_ref;
    options_instance.set(input_str, value);
}

/// # Safety
/// 
/// This function is unsafe because it takes a pointer to a c_char and returns a pointer to a Results struct.
/// The caller is responsible for ensuring that the pointer to the c_char is valid and that the Results struct is not null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn generate_surt(url: *const c_char) -> Results {
    let input_cstr = CStr::from_ptr(url);
    let input = input_cstr.to_str().unwrap().to_string();

    surt(&input, None)
}

/// # Safety
/// 
/// This function is unsafe because it takes a pointer to a SurtrOptions struct and returns a pointer to a Results struct.
/// The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the Results struct is not null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn generate_surt_with_options(url: *const c_char, option_ref: *mut SurtrOptions) -> Results {
    let input_cstr = CStr::from_ptr(url);
    let input = input_cstr.to_str().unwrap().to_string();

    let options = (*option_ref).clone();

    surt(&input, Some(options.clone()))
}

#[unsafe(no_mangle)]
pub extern "C" fn generate_surt_error(_url: *const c_char) -> Results {
    Results::from_error("Some Error".to_string())
}
