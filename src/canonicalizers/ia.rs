use lazy_static::lazy_static;
use regex::Regex;

use crate::handy_url::HandyUrl;
use crate::handy_url::DEFAULT_PORT;
use crate::options::SurtrOptions;
use crate::regex_transformer::{strip_path_session_id, strip_query_session_id};

lazy_static! {
    static ref RE_WWWDIGITS: Regex = Regex::new(r#"www\d*\."#).unwrap();
}

pub fn canonicalize(url_input: HandyUrl, options: &SurtrOptions) -> Result<HandyUrl, String> {
    let mut url = url_input;

    if options.host_lowercase && url.host.is_some() {
        url.host = Some(url.host.unwrap().to_lowercase());
    }

    let scheme = &url.scheme.clone().unwrap_or(String::new());

    if options.host_massage && url.host.is_some() && scheme != "dns" {
        url.host = Some(massage_host(url.host.unwrap()));
    }

    if options.auth_strip_user {
        url.auth_user = None;
        url.auth_pass = None;
    } else if options.auth_strip_pass {
        url.auth_pass = None;
    }

    if options.port_strip_default && url.scheme.is_some() {
        let default_port = get_default_port(url.scheme.clone().unwrap());

        if let Some(port) = &url.port {
            if port == &default_port {
                url.port = DEFAULT_PORT;
            }
        }
    }

    if let Some(mut path) = url.path {
        let mut should_be_none = false;

        if options.path_strip_empty && &path == "/" {
            url.path = None;
        } else {
            if options.path_lowercase {
                path = path.to_lowercase()
            }
            if options.path_strip_session_id {
                path = strip_path_session_id(path);
            }
            if options.path_strip_empty && &path == "/" {
                should_be_none = true;
            }
            if options.path_strip_trailing_slash_unless_empty {
                if path.ends_with('/') && path.len() > 1 {
                    path = path[0..(path.len() - 1)].to_string();
                }
            }
        }

        if should_be_none {
            url.path = None;
        } else {
            url.path = Some(path)
        }
    }

    if let Some(mut query) = url.query {
        if query.len() > 0 {
            if options.query_strip_session_id {
                query = strip_query_session_id(query);
            }
            if options.query_lowercase {
                query = query.to_lowercase();
            }
            if options.query_alpha_reorder {
                query = alpha_reorder_query(query);
            }
        }
        if &query == "" && options.query_strip_empty {
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

fn massage_host(host: String) -> String {
    if let Some(captues) = RE_WWWDIGITS.captures(&host) {
        if let Some(cap) = captues.get(0) {
            return host[cap.len()..].to_string();
        }
    }

    host
}

fn get_default_port(scheme: String) -> String {
    match scheme.to_lowercase().as_str() {
        "http" => "80",
        "https" => "443",
        _ => "0",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
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
        assert_eq!(massage_host("foo.com".to_string()), "foo.com");
        assert_eq!(massage_host("www.foo.com".to_string()), "foo.com");
        assert_eq!(massage_host("www12.foo.com".to_string()), "foo.com");

        assert_eq!(massage_host("www2foo.com".to_string()), "www2foo.com");
        assert_eq!(massage_host("www2.www2foo.com".to_string()), "www2foo.com");
    }

    #[test]
    fn test_get_default_port() {
        // These tests are from IAURLCanonicalizerTest.java
        assert_eq!(get_default_port("foo".to_string()), "0");
        assert_eq!(get_default_port("http".to_string()), "80");
        assert_eq!(get_default_port("https".to_string()), "443");
    }
}
