use pyo3::prelude::*;
use surtr::handy_url::HandyUrl;

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyHandyUrl {
    #[pyo3(get, set)]
    scheme: Option<String>,
    #[pyo3(get, set)]
    auth_user: Option<String>,
    #[pyo3(get, set)]
    auth_pass: Option<String>,
    #[pyo3(get, set)]
    host: Option<String>,
    #[pyo3(get, set)]
    port: Option<String>,
    #[pyo3(get, set)]
    path: Option<String>,
    #[pyo3(get, set)]
    query: Option<String>,
    #[pyo3(get, set)]
    hash: Option<String>,
    #[pyo3(get, set)]
    last_delimiter: Option<String>,
}

impl From<HandyUrl> for PyHandyUrl {
    fn from(value: HandyUrl) -> Self {
        Self {
            scheme: value.scheme,
            auth_user: value.auth_user,
            auth_pass: value.auth_pass,
            host: value.host,
            port: value.port,
            path: value.path,
            query: value.query,
            hash: value.hash,
            last_delimiter: value.last_delimiter,
        }
    }
}

impl Into<HandyUrl> for PyHandyUrl {
    fn into(self) -> HandyUrl {
        HandyUrl {
            scheme: self.scheme,
            auth_user: self.auth_user,
            auth_pass: self.auth_pass,
            host: self.host,
            port: self.port,
            path: self.path,
            query: self.query,
            hash: self.hash,
            last_delimiter: self.last_delimiter,
        }
    }
}
