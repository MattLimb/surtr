use std::fmt;

pub enum SaturError {
    UrlParseError(String),
    NoSchemeFoundError,
    CanonicalizerError(String),
}

impl fmt::Display for SaturError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_str: String = match self {
            Self::UrlParseError(s) => format!("{}", s),
            Self::NoSchemeFoundError => "no scheme found in given URL".to_string(),
            Self::CanonicalizerError(s) => format!("{}", s),
        };

        write!(f, "{}", err_str)
    }
}

impl fmt::Debug for SaturError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_str: String = match self {
            SaturError::UrlParseError(s) => format!("UrlParseError {{ {} }}", s),
            Self::NoSchemeFoundError => "NoSchemeFound".to_string(),
            SaturError::CanonicalizerError(s) => format!("EncodingError {{ {} }}", s),
        };

        write!(f, "{}", err_str)
    }
}
