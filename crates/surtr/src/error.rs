use std::fmt;

/// All possible errors Surtr can emit.
#[derive(Clone, PartialEq)]
pub enum SurtrError {
    /// A Parse Error where a URL is malformed.
    UrlParseError(String),
    /// An error occurs during canonicalization.
    /// This error is currently only present when a string is not UTF-8 encoded.
    CanonicalizerError(String),
    /// An error which expects the URL to contain a Scheme, but doesn't.
    NoSchemeFoundError,
}

impl fmt::Display for SurtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_str: String = match self {
            Self::UrlParseError(s) => format!("UrlParseError: {}", s),
            Self::NoSchemeFoundError => "NoSchemeFoundError: Expected scheme to be present in URL".to_string(),
            Self::CanonicalizerError(s) => format!("CanonicalizerError: {}", s),
        };

        write!(f, "{}", err_str)
    }
}

impl fmt::Debug for SurtrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_str: String = match self {
            SurtrError::UrlParseError(s) => format!("SurtrError::UrlParseError {{ {} }}", s),
            Self::NoSchemeFoundError => "SurtrError::NoSchemeFound".to_string(),
            SurtrError::CanonicalizerError(s) => format!("SurtrError::CanonicalizerError {{ {} }}", s),
        };

        write!(f, "{}", err_str)
    }
}
