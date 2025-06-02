use pyo3::{create_exception, prelude::*};
use pyo3::exceptions::PyException;
use pyo3::types::PyDict;
use surtr::SurtrError;

create_exception!(py_surtr, SurtrException, PyException);
create_exception!(py_surtr, UrlParseError, SurtrException);
create_exception!(py_surtr, NoSchemeFoundError, SurtrException);
create_exception!(py_surtr, CanonicalizerError, SurtrException);

/// UrlInput is the Rust Enums to allow a Python String
/// or a Python Bytes String to be passed into the surt function.
#[derive(FromPyObject, Debug)]
pub enum UrlInput {
    #[pyo3(transparent, annotation = "str")]
    String(String),
    #[pyo3(transparent, annotation = "bytes")]
    Bytes(Vec<u8>),
}


/// UrlInput is the Rust Enums to allow a Python String
/// or a Python Bytes String to be returned from the surt function.
#[derive(IntoPyObject, Debug)]
pub enum UrlOutput {
    #[pyo3(transparent)]
    String(String),
    #[pyo3(transparent)]
    Bytes(Vec<u8>),
}

// build_options is an internal function which converts a Python **kwargs dictionary
// into the SurtrOptions struct which is needed for Surtr.
fn build_options(dict: &Bound<'_, PyDict>) -> PyResult<surtr::SurtrOptions> {
    let mut opts = surtr::SurtrOptions::default();

    for item in dict.items() {
        let key = item.get_item(0)?.to_string();
        let value = matches!(item.get_item(1)?.to_string().as_str(), "True");
        opts.set(&key, value);
    }

    Ok(opts)
}

/// surt - Convert a URL into a Sort-friently URL Reordering Transform (SURT).
/// 
/// This function aims to be compatiable with the Internet Archive.
/// 
/// Currently missing the custom Canonicalization functions.
/// 
/// Args:
/// 
/// - url (str | bytes) - The URL to transform. String or Bytes format.
/// - **kwargs - A set of named boolean options. View the readme for a complete list.
/// 
/// Returns:
/// 
/// The SURT as a String.
/// 
/// Raises:
/// 
/// - UrlParseError - If the URL is invalid.
/// - NoSchemeFoundError - If the parsing expected a Scheme, but couldn't find one.
/// - CanonicalizerError - If there is an issue during canonicalization.
#[pyfunction]
#[pyo3(signature = (url=None, **kwargs))]
pub fn surt(
    url: Option<UrlInput>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<UrlOutput> {
    let opts: Option<surtr::SurtrOptions> = match kwargs {
        None => None,
        Some(d) => Some(build_options(d)?),
    };
    let mut in_type = "string";

    let in_url: String = match url {
        None => String::new(),
        Some(u) => match u {
            UrlInput::String(s) => s,
            UrlInput::Bytes(b) => {
                in_type = "bytes";
                String::from_utf8(b)?
            }
        },
    };

    if in_url == *"" {
        return Ok(UrlOutput::String("-".to_string()));
    }

    match surtr::surt(&in_url, opts) {
        Ok(s) => match in_type {
            "string" => Ok(UrlOutput::String(s)),
            _ => Ok(UrlOutput::Bytes(s.as_bytes().to_vec())),
        },
        Err(e) => match e {
            SurtrError::CanonicalizerError(s) => Err(CanonicalizerError::new_err(s.to_string())),
            SurtrError::NoSchemeFoundError => Err(NoSchemeFoundError::new_err(e.to_string())),
            SurtrError::UrlParseError(s) => Err(UrlParseError::new_err(s.to_string()))
        },
    }
}


/// py_surtr - Rust based Surt implementation aiming to be compatiable with the Internet Archives
/// SURT implementation.
#[pymodule]
pub fn py_surtr(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Custom Errors
    m.add("SurtrException", py.get_type::<SurtrException>())?;
    m.add("UrlParseError", py.get_type::<UrlParseError>())?;
    m.add("NoSchemeFoundError", py.get_type::<NoSchemeFoundError>())?;
    m.add("CanonicalizerError", py.get_type::<CanonicalizerError>())?;

    // Add Functions
    m.add_function(wrap_pyfunction!(surt, m)?)
}
