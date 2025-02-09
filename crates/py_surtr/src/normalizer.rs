use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFunction};

use surtr;

use crate::py_handy_url::PyHandyUrl;

#[derive(FromPyObject)]
pub enum InputFunctions<'a> {
    #[pyo3(transparent, annotation = "callable")]
    Function(Bound<'a, PyFunction>),
    #[pyo3(transparent, annotation = "list[callable]")]
    FunctionList(Vec<Bound<'a, PyFunction>>),
}

pub struct Normalizer<'a> {
    funcs: Vec<Bound<'a, PyFunction>>,
}

impl<'a> Normalizer<'a> {
    pub fn new_from(funcs: InputFunctions<'a>) -> Self {
        match funcs {
            InputFunctions::Function(f) => Self { funcs: vec![f] },
            InputFunctions::FunctionList(fl) => Self { funcs: fl },
        }
    }
}

pub fn canonicalize(funcs: InputFunctions) -> Box<surtr::Canonicalizer> {
    let func_li: Vec<Bound<'_, PyFunction>> = match funcs {
        InputFunctions::Function(f) => vec![f.clone()],
        InputFunctions::FunctionList(fl) => fl.clone(),
    };

    Box::new(
        |url_input: surtr::handy_url::HandyUrl,
         options: &surtr::options::SurtrOptions|
         -> Result<surtr::handy_url::HandyUrl, String> {
            Ok(url_input)

            // let mut out: PyHandyUrl = PyHandyUrl::from(url_input);

            // Python::with_gil(|py| {
            //     let py_opts = PyDict::new(py);
            //     for (key, value) in options.as_items() {
            //         py_opts.set_item(key, value).unwrap();
            //     }

            //     for func in func_li {
            //         out = func
            //             .call::<(PyHandyUrl,)>((out.clone(),), Some(&py_opts))
            //             .unwrap()
            //             .extract::<PyHandyUrl>()
            //             .unwrap();
            //     }
            // });

            // Ok(out.into())
        },
    )
}
