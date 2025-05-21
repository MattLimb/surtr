use lazy_static::lazy_static;
use regex::Regex;

use crate::error::SurtrError;
use crate::handy_url::HandyUrl;
use crate::options::SurtrOptions;
use crate::regex_transformer::{strip_path_session_id, strip_query_session_id};

lazy_static! {
    static ref RE_WWWDIGITS: Regex = Regex::new(r#"www\d*\."#).expect("Failed to compile www.digits regex");
}

pub fn canonicalize(url_input: HandyUrl, options: &SurtrOptions) -> Result<HandyUrl, SurtrError> {
    let mut url = url_input;

    if options.get_or("host_lowercase", true) && url.host.is_some() {
        url.host = Some(url.host.unwrap().to_lowercase());
    }

    let scheme = &url.scheme.clone().unwrap_or_default();

    if options.get_or("host_massage", true) && url.host.is_some() && scheme != "dns" {
        url.host = massage_host(url.host);
    }

    if options.get_or("auth_strip_user", true) {
        url.auth_user = None;
        url.auth_pass = None;
    } else if options.get_or("auth_strip_pass", true) {
        url.auth_pass = None;
    }

    if options.get_or("port_strip_default", true) && url.scheme.is_some() {
        let default_port = get_default_port(&url.scheme);

        if let Some(port) = &url.port {
            if port == &default_port {
                url.port = None;
            }
        }
    }

    if let Some(mut path) = url.path {
        let mut should_be_none = false;

        if options.get_or("path_strip_empty", false) && &path == "/" {
            url.path = None;
        } else {
            if options.get_or("path_lowercase", true) {
                path = path.to_lowercase()
            }
            if options.get_or("path_strip_session_id", true) {
                path = strip_path_session_id(path);
            }
            if options.get_or("path_strip_empty", false) && &path == "/" {
                should_be_none = true;
            }
            if options.get_or("path_strip_trailing_slash_unless_empty", true) && path.ends_with('/') && path.len() > 1 {
                path = path[0..(path.len() - 1)].to_string();
            }
        }

        if should_be_none {
            url.path = None;
        } else {
            url.path = Some(path)
        }
    }

    if let Some(mut query) = url.query {
        if !query.is_empty() {
            if options.get_or("query_strip_session_id", true) {
                query = strip_query_session_id(query);
            }
            if options.get_or("query_lowercase", true) {
                query = query.to_lowercase();
            }
            if options.get_or("query_alpha_reorder", true) {
                query = alpha_reorder_query(query);
            }
        }
        if query.is_empty() && options.get_or("query_strip_empty", true) {
            url.query = None
        } else {
            url.query = Some(query)
        }
    } else {
        url.last_delimiter = None
    }

    Ok(url)
}

fn alpha_reorder_query(input: String) -> String {
    if input.len() <= 1 {
        return input;
    }

    let args: Vec<&str> = input.split('&').collect();
    let mut qas: Vec<Vec<&str>> = args
        .iter()
        .map(|arg| arg.splitn(2, '=').collect())
        .collect();

    qas.sort();
    let mut output = String::new();

    for item in qas.iter() {
        match &item.len() {
            0 => {}
            1 => output = format!("{}{}&", output, item[0]),
            _ => output = format!("{}{}={}&", output, item[0], item[1]),
        }
    }

    output[0..(output.len() - 1)].to_string()
}

fn massage_host(host: Option<String>) -> Option<String> {
    host.as_ref()?;

    // Unwrap is ok to use here. We know that host has a value at this point.
    let host = host.unwrap();

    if let Some(captues) = RE_WWWDIGITS.captures(&host) {
        if let Some(cap) = captues.get(0) {
            return Some(host[cap.len()..].to_string());
        }
    }

    Some(host)
}

fn get_default_port(scheme: &Option<String>) -> String {
    if let Some(sch) = scheme {
        return match sch.to_lowercase().as_str() {
            "http" => "80",
            "https" => "443",
            _ => "0",
        }.to_string();
    }

    String::from("0")
}

#[cfg(test)]
mod tests {
    use crate::options::SurtrOptions;

    use super::*;

    #[test]
    fn test_ia_canonicalizer() {
        let def_options = SurtrOptions::default();

        // These tests are from IAURLCanonicalizerTest.java
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://ARCHIVE.ORG/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://archive.org/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.archive.org:80/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://archive.org/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("https://www.archive.org:80/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "https://archive.org:80/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.archive.org:443/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://archive.org:443/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("https://www.archive.org:443/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "https://archive.org/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.archive.org/big/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://archive.org/big"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("dns:www.archive.org").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "dns:www.archive.org"
        );
    }

    #[test]
    fn test_alpha_reorder_query() {
        // These tests are from IAURLCanonicalizerTest.java
        assert_eq!(alpha_reorder_query("".to_string()), "");
        assert_eq!(alpha_reorder_query("".to_string()), "");
        assert_eq!(alpha_reorder_query("a".to_string()), "a");
        assert_eq!(alpha_reorder_query("a".to_string()), "a");
        assert_eq!(alpha_reorder_query("a=1".to_string()), "a=1");
        assert_eq!(alpha_reorder_query("ab=1".to_string()), "ab=1");
        assert_eq!(alpha_reorder_query("a=1&".to_string()), "&a=1");
        assert_eq!(alpha_reorder_query("a=1&b=1".to_string()), "a=1&b=1");
        assert_eq!(alpha_reorder_query("b=1&a=1".to_string()), "a=1&b=1");
        assert_eq!(alpha_reorder_query("a=a&a=a".to_string()), "a=a&a=a");
        assert_eq!(alpha_reorder_query("a=b&a=a".to_string()), "a=a&a=b");
        assert_eq!(
            alpha_reorder_query("b=b&a=b&b=a&a=a".to_string()),
            "a=a&a=b&b=a&b=b"
        );
    }

    #[test]
    fn test_massage_host() {
        // These tests are from IAURLCanonicalizerTest.java
        assert_eq!(massage_host(Some("foo.com".to_string())), Some("foo.com".to_string()));
        assert_eq!(massage_host(Some("www.foo.com".to_string())), Some("foo.com".to_string()));
        assert_eq!(massage_host(Some("www12.foo.com".to_string())), Some("foo.com".to_string()));

        assert_eq!(massage_host(Some("www2foo.com".to_string())), Some("www2foo.com".to_string()));
        assert_eq!(massage_host(Some("www2.www2foo.com".to_string())), Some("www2foo.com".to_string()));
    }

    #[test]
    fn test_get_default_port() {
        // These tests are from IAURLCanonicalizerTest.java
        assert_eq!(get_default_port(&Some("foo".to_string())), "0");
        assert_eq!(get_default_port(&Some("http".to_string())), "80");
        assert_eq!(get_default_port(&Some("https".to_string())), "443");
    }
}
