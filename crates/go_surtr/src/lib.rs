//! Go Bindings for Surtr - A Rust based Sort-friendly URI Reordering Transform. ([SURT])
//! 
//! The crate intends to be as compatible as possible to the original [IA implementation]. 
//! 
//! Currently, custom Canonicalization functions are missing from the Public Interface.
//! They are being considered as part of a future release.
//! 
//! [SURT]: http://crawler.archive.org/articles/user_manual/glossary.html#surt
//! [IA implementation]: https://github.com/internetarchive/surt
//! 
//! # Safety
//! 
//! This crate is unsafe because it uses C bindings to call the Rust code.
//! The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the SurtrOptions struct is not null.
//! The caller is responsible for ensuring that the pointer to the c_char is valid and that the Results struct is not null.
//! The caller is responsible for ensuring that the pointer to the c_char is valid and that the Results struct is not null.


use std::ffi::{CStr, CString, c_char};
use surtr::SurtrError;
use surtr::SurtrOptions;

/// A CStyle Struct to pass errors back to Go.
#[repr(C)]
pub struct Results {
    /// A C Pointer to the successful SURT output string.
    output: *const c_char,
    /// A C Pointer to the error description.
    error: *const c_char,
}

impl Results {
    /// Create a Results struct from a successful SURT output string.
    /// 
    /// # Arguments
    /// 
    /// * `s` - The successful SURT output string.
    /// 
    /// # Returns
    /// 
    /// A Results struct in the successful Output configuration.
    pub fn from_string(s: String) -> Self {
        Self {
            output: CString::new(s).unwrap().into_raw(),
            error: ::std::ptr::null(),
        }
    }

    /// Create a Results struct from an error description.
    /// 
    /// # Arguments
    /// 
    /// * `e` - The error description.
    /// 
    /// # Returns
    /// 
    /// A Results struct in the Error configuration.
    pub fn from_error(e: String) -> Self {
        Self {
            output: ::std::ptr::null(),
            error: CString::new(e).unwrap().into_raw(),
        }
    }
}


// Internal function to call surtr and parse the error into an appropriate Results Object.
//
// # Arguments
// 
// * `url` - The URL to be transformed.
// * `options` - The options to be used for the transformation.
//
// # Returns
//
// A Results struct in the successful Output configuration, or an error configuration if the URL is invalid.
fn surt(url: &str, options: Option<SurtrOptions>) -> Results {
    match surtr::surt(url, options) {
        Ok(s) => Results::from_string(s),
        Err(SurtrError::UrlParseError(e)) => Results::from_error(e.to_string()),
        Err(SurtrError::NoSchemeFoundError) => Results::from_error("Excpected URL Scheme - None Found".to_string()),
        Err(SurtrError::CanonicalizerError(e)) => Results::from_error(e.to_string())
    }
}

/// Initialize the SurtrOptions Struct internally. This passes a Pointer back to the Caller.
/// 
/// # Returns
/// 
/// A Pointer to the SurtrOptions struct.
/// 
/// # Safety
/// 
/// This function is considered unsafe because is passes a pointer back to the caller. This pointer is intended to be used with
/// the destroy_options and set_option functions. Always use destroy_options after all uses of this pointer are used. This will help
/// to prevent memory leaks.
#[unsafe(no_mangle)]
pub extern "C" fn init_options() -> *mut SurtrOptions {
    let new_options = SurtrOptions::default();
    let boxed_options = Box::new(new_options);

    Box::into_raw(boxed_options)
}

/// Destroy the SurtrOptions Struct internally. This frees the memory allocated for the SurtrOptions struct.
/// 
/// # Arguments
/// 
/// * `inst_ref` - A Pointer to the SurtrOptions struct to be destroyed.
///
/// # Safety
/// 
/// This function is unsafe because it takes a pointer to a SurtrOptions struct and returns a pointer to a SurtrOptions struct.
/// The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the SurtrOptions struct is not null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn destroy_options(inst_ref: *mut SurtrOptions) {
    unsafe {
        let _ = Box::from_raw(inst_ref);
    };
}

/// Set an option within the SurtrOptions struct.
/// 
/// # Arguments
/// 
/// * `inst_ref` - A Pointer to the SurtrOptions struct to be modified.
/// * `name` - A Pointer to the c_char containing the name of the option to be set.
/// * `value` - A bool containing the value of the option to be set.
///
/// # Safety
/// 
/// This function is unsafe because it takes a pointer to a SurtrOptions struct and a pointer to a c_char and a bool.
/// The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the pointer to the c_char is valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_option(inst_ref: *mut SurtrOptions, name: *const c_char, value: bool) {
    let input_str = unsafe {CStr::from_ptr(name).to_str().unwrap() };

    let options_instance = unsafe { &mut *inst_ref };
    options_instance.set(input_str, value);
}

/// Generate a SURT from a URL.
/// 
/// # Arguments
/// 
/// * `url` - A Pointer to the c_char containing the URL to be transformed.
///
/// # Returns
/// 
/// A Results struct in the successful Output configuration, or an error configuration if the URL is invalid.
/// 
/// # Safety
/// 
/// This function is unsafe because it takes a pointer to a c_char and returns a pointer to a Results struct.
/// The caller is responsible for ensuring that the pointer to the c_char is valid and that the Results struct is not null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn generate_surt(url: *const c_char) -> Results {
    let input_cstr = unsafe { CStr::from_ptr(url) };
    let input = input_cstr.to_str().unwrap().to_string();

    surt(&input, None)
}

/// Generate a SURT from a URL with custom options.
/// 
/// # Arguments
/// 
/// * `url` - A Pointer to the c_char containing the URL to be transformed.
/// * `option_ref` - A Pointer to the SurtrOptions struct to be used for the transformation.
///
/// # Returns
/// 
/// A Results struct in the successful Output configuration, or an error configuration if the URL is invalid.
/// 
/// # Safety
/// 
/// This function is unsafe because it takes a pointer to a SurtrOptions struct and returns a pointer to a Results struct.
/// The caller is responsible for ensuring that the pointer to the SurtrOptions struct is valid and that the Results struct is not null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn generate_surt_with_options(url: *const c_char, option_ref: *mut SurtrOptions) -> Results {
    let input_cstr = unsafe { CStr::from_ptr(url) };
    let input = input_cstr.to_str().unwrap().to_string();

    let options = unsafe { (*option_ref).clone() };

    surt(&input, Some(options.clone()))
}
