use pyo3::{create_exception, prelude::*};

use pyo3::exceptions::PyException;
use pyo3::types::PyDict;
use surtr::SurtrError;

pub mod py_handy_url;

create_exception!(py_surtr, SurtrException, PyException);
create_exception!(py_surtr, UrlParseError, SurtrException);
create_exception!(py_surtr, NoSchemeFoundError, SurtrException);
create_exception!(py_surtr, CanonicalizerError, SurtrException);

#[derive(FromPyObject, Debug)]
pub enum UrlInput {
    #[pyo3(transparent, annotation = "str")]
    String(String),
    #[pyo3(transparent, annotation = "bytes")]
    Bytes(Vec<u8>),
}

#[derive(IntoPyObject, Debug)]
pub enum UrlOutput {
    #[pyo3(transparent)]
    String(String),
    #[pyo3(transparent)]
    Bytes(Vec<u8>),
}

fn build_options(dict: &Bound<'_, PyDict>) -> PyResult<surtr::SurtrOptions> {
    let mut opts = surtr::SurtrOptions::default();

    for item in dict.items() {
        let key = item.get_item(0)?.to_string();
        let value = matches!(item.get_item(1)?.to_string().as_str(), "True");
        opts.set(&key, value);
    }

    Ok(opts)
}

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
