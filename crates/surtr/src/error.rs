use std::fmt;

pub enum SurtrError {
    Error(String),
    UrlParseError(String),
    NoSchemeFoundError,
    CanonicalizerError(String),
}

impl fmt::Display for SurtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_str: String = match self {
            Self::Error(s) => format!("{}", s),
            Self::UrlParseError(s) => format!("{}", s),
            Self::NoSchemeFoundError => "no scheme found in given URL".to_string(),
            Self::CanonicalizerError(s) => format!("{}", s),
        };

        write!(f, "{}", err_str)
    }
}

impl fmt::Debug for SurtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_str: String = match self {
            SurtrError::Error(s) => format!("SutrError::Error {{ {} }}", s),
            SurtrError::UrlParseError(s) => format!("SurtrError::UrlParseError {{ {} }}", s),
            Self::NoSchemeFoundError => "SurtrError::NoSchemeFound".to_string(),
            SurtrError::CanonicalizerError(s) => format!("SurtrError::EncodingError {{ {} }}", s),
        };

        write!(f, "{}", err_str)
    }
}
