use pyo3::{create_exception, prelude::*};

use pyo3::exceptions::PyException;
use pyo3::types::{PyDict, PyFunction};
use surtr::error::SurtrError;

pub mod py_handy_url;

create_exception!(py_surtr, SurtrException, PyException);
create_exception!(py_surtr, UrlParseError, SurtrException);
create_exception!(py_surtr, NoSchemeFoundError, SurtrException);
create_exception!(py_surtr, CanonicalizerError, SurtrException);

#[derive(FromPyObject)]
pub enum InputFunctions<'a> {
    #[pyo3(transparent, annotation = "callable")]
    Function(Bound<'a, PyFunction>),
    #[pyo3(transparent, annotation = "list[callable]")]
    FunctionList(Vec<Bound<'a, PyFunction>>),
}

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

fn build_options(dict: &Bound<'_, PyDict>) -> PyResult<surtr::options::SurtrOptions> {
    let mut opts = surtr::options::SurtrOptions::default();

    for item in dict.items() {
        let key = item.get_item(0)?.to_string();
        let value = matches!(item.get_item(1)?.to_string().as_str(), "True");
        opts.set(&key, value);
    }

    Ok(opts)
}

#[pyfunction]
#[pyo3(signature = (url=None, canonicalizer=None, **kwargs))]
pub fn surt<'a>(
    url: Option<UrlInput>,
    canonicalizer: Option<InputFunctions<'a>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<UrlOutput> {
    let opts: Option<surtr::options::SurtrOptions> = match kwargs {
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

    let func_li: Option<Vec<Bound<'a, PyFunction>>> = match canonicalizer {
        Some(InputFunctions::Function(f)) => Some(vec![f.clone()]),
        Some(InputFunctions::FunctionList(fl)) => Some(fl.clone()),
        None => None,
    };

    let canon: Option<surtr::Canonicalizer> = match func_li {
        Some(li) => Some(Box::new(
            move |x: surtr::handy_url::HandyUrl, y: &surtr::options::SurtrOptions| {
                let mut out: py_handy_url::PyHandyUrl = py_handy_url::PyHandyUrl::from(x);

                let success: Result<(), SurtrError> = Python::with_gil(|py| {
                    let py_opts = PyDict::new(py);
                    for (key, value) in y.as_items() {
                        py_opts.set_item(key, value).unwrap();
                    }

                    for func in li {
                        match func
                            .call::<(py_handy_url::PyHandyUrl,)>((out.clone(),), Some(&py_opts))
                        {
                            Ok(hurl) => match hurl.extract::<py_handy_url::PyHandyUrl>() {
                                Ok(o) => out = o,
                                Err(e) => {
                                    return Err(SurtrError::Error(e.value(py).to_string()));
                                }
                            },
                            Err(e) => {
                                return Err(SurtrError::Error(e.to_string()));
                            }
                        }
                    }

                    Ok(())
                });

                if let Err(e) = success {
                    Err(e)
                } else {
                    Ok(out.into())
                }
            },
        )),
        None => None,
    };

    match surtr::surt(Some(&in_url), canon, opts) {
        Ok(s) => match in_type {
            "string" => Ok(UrlOutput::String(s)),
            _ => Ok(UrlOutput::Bytes(s.as_bytes().to_vec())),
        },
        Err(e) => match e {
            SurtrError::CanonicalizerError(s) => Err(CanonicalizerError::new_err(s.to_string())),
            SurtrError::NoSchemeFoundError => Err(NoSchemeFoundError::new_err(e.to_string())),
            SurtrError::UrlParseError(s) => Err(UrlParseError::new_err(s.to_string())),
            SurtrError::Error(s) => Err(SurtrException::new_err(s.to_string())),
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
