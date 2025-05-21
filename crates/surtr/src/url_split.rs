use lazy_static::lazy_static;
use regex::Regex;

use crate::error::SurtrError;

lazy_static! {
    // These Regexes expect here, because they should always compile. The system doesn't work without them compiling
    // so we should panic if they cannot compile.
    static ref REGEX: Regex = Regex::new(
        r#"^(([a-zA-Z][a-zA-Z0-9+.-]*):)?((//([^/?#]*))?([^?#]*)(\?([^#]*))?)?(#(.*))?$"#
    ).expect("Failed to compile RFC2396 Regex");
}

#[derive(Debug, PartialEq, Eq)]
pub struct SplitResult {
    pub scheme: Option<String>,
    pub netloc: Option<String>,
    pub path: Option<String>,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl SplitResult {
    pub fn parse(url: String) -> Result<Self, SurtrError> {
        let captures = match REGEX.captures(&url) {
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
            netloc,
            path,
            query,
            fragment,
        })
    }
}

pub fn split_netloc(netloc: String) -> (Option<String>, Option<String>) {
    let split: Vec<&str> = netloc.split(":").collect();

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
        let result =
            SplitResult::parse(String::from("http://www.ics.uci.edu/pub/ietf/uri/#Related"));

        assert!(result.is_ok());
        let comp_result = result.unwrap();

        assert_eq!(
            comp_result,
            SplitResult {
                scheme: Some(String::from("http")),
                netloc: Some(String::from("www.ics.uci.edu")),
                path: Some(String::from("/pub/ietf/uri/")),
                query: None,
                fragment: Some(String::from("Related"))
            }
        );
        assert_eq!(
            split_netloc(comp_result.netloc.unwrap()),
            (Some(String::from("www.ics.uci.edu")), None)
        )
    }
}
