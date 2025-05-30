use lazy_static::lazy_static;
use regex::Regex;

use crate::{error::SurtrError, SurtrOptions};

lazy_static! {
    // These Regexes expect here, because they should always compile. The system doesn't work without them compiling
    // so we should panic if they cannot compile.
    static ref REGEX: Regex = Regex::new(
        r#"^(([a-zA-Z][a-zA-Z0-9+.-]*):)?((//([^/?#]*))?([^?#]*)(\?([^#]*))?)?(#(.*))?$"#
    ).expect("Failed to compile RFC2396 Regex");
}

// Full URL Deconstruction. This follows the Python Standard Library Result implementation.
#[derive(Debug, PartialEq, Eq)]
pub struct SplitResult {
    // The Protocol used (http, dns, ftp, etc.)
    pub scheme: Option<String>,
    // The network location. Typically known as the Host part of the URL.
    pub netloc: SplitNetloc,
    // The path of the resource specified in the URL.
    pub path: Option<String>,
    // The query parameters of the URL.
    pub query: Option<String>,
    // The Hash query parameters of the URL. Client only URL query parameters.
    pub fragment: Option<String>,
}

impl SplitResult {
    // Parse the URL into the components. 
    // 
    // # Arguments
    //
    // * `url` - The URL to parse.
    // * `options` - The SurtrOptions struct.
    //
    // # Returns
    //
    // A Result containing the parsed URL as a SplitResult struct, or an error if the URL is invalid.
    pub fn parse(url: String, options: &SurtrOptions) -> Result<Self, SurtrError> {
        let captures: regex::Captures<'_> = match REGEX.captures(&url) {
            Some(t) => t,
            None => {
                return Err(SurtrError::UrlParseError(
                    "url regex match failed".to_string(),
                ))
            }
        };

        let scheme = captures
            .get(2)
            .map(|x| String::from(x.as_str()));
        let query = captures.get(7).and_then(|x| {
                let q = x.as_str().replacen("?", "", 1);

                if q.is_empty() {
                    return None;
                }
                Some(q)
            });
        let fragment = captures
            .get(10)
            .map(|x| String::from(x.as_str()));

        let mut netloc = captures.get(5).and_then(|x| {
            if x.is_empty() {
                return None;
            }
            Some(String::from(x.as_str()))
        });
        let mut path = captures
            .get(6)
            .map(|x| String::from(x.as_str()));

        (netloc, path) = match &scheme {
            None => (netloc, path),
            Some(sch) => {
                let mut tmpnl = netloc;
                let mut tmpp = path;

                if sch.starts_with("http") && tmpnl.is_none() && tmpp.is_some() {
                    let mut pth: String = tmpp.clone().unwrap();
                    pth = pth.trim_start_matches('/').to_string();

                    if let Some((shst, spth)) = pth.split_once('/') {
                        tmpnl = Some(String::from(shst));
                        tmpp = Some(format!("/{}", spth));
                    } else {
                        tmpnl = Some(pth);
                        tmpp = Some(String::from("/"));
                    }
                };

                (tmpnl, tmpp)
            }
        };

        if let Some(pth) = &path {
            if pth.is_empty() {
                path = None;
            }
        }

        Ok(Self {
            scheme,
            netloc: SplitNetloc::parse_opt(netloc, options.get_or("auth_exclude", true)),
            path,
            query,
            fragment,
        })
    }
}


// A struct to correctly handle the splitting of the host section of a URL
// Supports BasicAuth username and password, in addition to splitting URLs and ports.
#[derive(Debug, PartialEq, Eq)]
pub struct SplitNetloc {
    pub auth_user: Option<String>,
    pub auth_pass: Option<String>,
    pub domain: Option<String>,
    pub port: Option<String>,
}


impl SplitNetloc {
    // Parse the URL into the components. 
    // 
    // # Arguments
    //
    // * `netloc` - The Host portion of a URL.
    // * `options` - The SurtrOptions struct.
    //
    // # Returns
    //
    // A Result containing the parsed URL as a SplitNetloc struct, or an error if the URL is invalid.
    //
    pub fn parse(netloc: String, auth_exclude: bool) -> Self {
        let mut auth_user: Option<String> = None;
        let mut auth_pass:Option<String>  = None;
        let domain: Option<String>;
        let port: Option<String>;

        let s: Vec<&str> = netloc.splitn(2, '@').collect();
        if s.len() == 2 {
            if !auth_exclude {
                (auth_user, auth_pass) = split_on_char(s[0].to_string(), ':');
            }
            (domain, port) = split_on_char(s[1].to_string(), ':');
        } else {
            (domain, port) = split_on_char(s[0].to_string(), ':');
        }

        Self {
            auth_user,
            auth_pass,
            domain,
            port,
        }
    }

    // Parse the URL into the components. Takes in an Option<String> instead of a bare String.
    // 
    // # Arguments
    //
    // * `netloc` - The Host portion of a URL.
    // * `options` - The SurtrOptions struct.
    //
    // # Returns
    //
    // A Result containing the parsed URL as a SplitNetloc struct, or a blank Netloc struct.
    //
    pub fn parse_opt(netloc: Option<String>, auth_exclude: bool) -> Self {
        match netloc {
            None => Self::default(),
            Some(nl) => Self::parse(nl, auth_exclude)
        }
    }
}

impl Default for SplitNetloc {
    fn default() -> Self {
        Self {
            auth_user: None,
            auth_pass: None,
            domain: None,
            port: None
        }
    }
}


// Splits the Network Location into domain/ip and port number.
pub fn split_on_char(item: String, char: char) -> (Option<String>, Option<String>) {
    let split: Vec<&str> = item.split(char).collect();

    (
        split.first().map(|x| x.to_string()),
        split.get(1).and_then(|x| {
            if x.is_empty() {
                return None;
            }
            Some(x.to_string())
        }),
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_result() {
        let options = SurtrOptions::default();
        let result =
            SplitResult::parse(String::from("http://www.ics.uci.edu/pub/ietf/uri/#Related"), &options);

        assert!(result.is_ok());
        let comp_result = result.unwrap();


        let exp_netloc = SplitNetloc {
            auth_user: None,
            auth_pass: None,
            domain: Some(String::from("www.ics.uci.edu")),
            port: None
        };
        assert_eq!(
            comp_result,
            SplitResult {
                scheme: Some(String::from("http")),
                netloc: exp_netloc,
                path: Some(String::from("/pub/ietf/uri/")),
                query: None,
                fragment: Some(String::from("Related"))
            }
        );
    }

    #[test]
    fn split_result_with_username() {
        let mut options = SurtrOptions::default();
        options.set("auth_exclude", false);

        let result =
            SplitResult::parse(String::from("http://user@www.ics.uci.edu/pub/ietf/uri/#Related"), &options);

        assert!(result.is_ok());
        let comp_result = result.unwrap();

        let exp_netloc = SplitNetloc {
            auth_user: Some(String::from("user")),
            auth_pass: None,
            domain: Some(String::from("www.ics.uci.edu")),
            port: None
        };
        assert_eq!(
            comp_result,
            SplitResult {
                scheme: Some(String::from("http")),
                netloc: exp_netloc,
                path: Some(String::from("/pub/ietf/uri/")),
                query: None,
                fragment: Some(String::from("Related"))
            }
        );
    }

    #[test]
    fn split_result_with_auth() {
        let mut options = SurtrOptions::default();
        options.set("auth_exclude", false);

        let result =
            SplitResult::parse(String::from("http://user:pass@www.ics.uci.edu/pub/ietf/uri/#Related"), &options);

        assert!(result.is_ok());
        let comp_result = result.unwrap();

        let exp_netloc = SplitNetloc {
            auth_user: Some(String::from("user")),
            auth_pass: Some(String::from("pass")),
            domain: Some(String::from("www.ics.uci.edu")),
            port: None
        };
        assert_eq!(
            comp_result,
            SplitResult {
                scheme: Some(String::from("http")),
                netloc: exp_netloc,
                path: Some(String::from("/pub/ietf/uri/")),
                query: None,
                fragment: Some(String::from("Related"))
            }
        );
    }

    #[test]
    fn split_result_with_auth_port() {
        let mut options = SurtrOptions::default();
        options.set("auth_exclude", false);

        let result =
            SplitResult::parse(String::from("http://user:pass@www.ics.uci.edu:8080/pub/ietf/uri/#Related"), &options);

        assert!(result.is_ok());
        let comp_result = result.unwrap();

        let exp_netloc = SplitNetloc {
            auth_user: Some(String::from("user")),
            auth_pass: Some(String::from("pass")),
            domain: Some(String::from("www.ics.uci.edu")),
            port: Some(String::from("8080"))
        };
        assert_eq!(
            comp_result,
            SplitResult {
                scheme: Some(String::from("http")),
                netloc: exp_netloc,
                path: Some(String::from("/pub/ietf/uri/")),
                query: None,
                fragment: Some(String::from("Related"))
            }
        );
    }

    #[test]
    fn split_result_with_port() {
        let mut options = SurtrOptions::default();
        options.set("auth_exclude", false);

        let result =
            SplitResult::parse(String::from("http://www.ics.uci.edu:8080/pub/ietf/uri/#Related"), &options);

        assert!(result.is_ok());
        let comp_result = result.unwrap();

        let exp_netloc = SplitNetloc {
            auth_user: None,
            auth_pass: None,
            domain: Some(String::from("www.ics.uci.edu")),
            port: Some(String::from("8080"))
        };
        assert_eq!(
            comp_result,
            SplitResult {
                scheme: Some(String::from("http")),
                netloc: exp_netloc,
                path: Some(String::from("/pub/ietf/uri/")),
                query: None,
                fragment: Some(String::from("Related"))
            }
        );
    }
}
