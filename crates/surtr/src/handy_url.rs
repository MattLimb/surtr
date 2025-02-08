use std::fmt::Display;

use lazy_static::lazy_static;
use regex::Regex;
use tld_extract;

use crate::{
    options::SurtrOptions,
    regex_transformer::host_to_surt,
    url_split::{self, SplitResult},
};

const TLD_SOURCE: tld_extract::Source = tld_extract::Source::Snapshot;
pub const DEFAULT_PORT: Option<String> = None;

lazy_static! {
    static ref RE_MULTIPLE_PROTOCOLS: Regex = Regex::new(r#"^(https?://)+"#).unwrap();
    static ref RE_HAS_PROTOCOL: Regex = Regex::new(r#"^([a-zA-Z][a-zA-Z0-9\+\-\.]*):"#).unwrap();
    static ref RE_SPACES: Regex = Regex::new(r#"[\n\r\t]"#).unwrap();
}

#[derive(Debug, Clone)]
pub struct HandyUrl {
    pub scheme: Option<String>,
    pub auth_user: Option<String>,
    pub auth_pass: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
    pub path: Option<String>,
    pub query: Option<String>,
    pub hash: Option<String>,
    pub last_delimiter: Option<String>,
}

impl HandyUrl {
    fn add_default_scheme_if_needed(url: &str) -> String {
        if RE_HAS_PROTOCOL.is_match(url) {
            return String::from(url);
        }

        format!("http://{}", url)
    }

    pub fn parse(raw_url: &str) -> Result<Self, String> {
        let mut url: String = raw_url.trim().to_string();
        url = RE_SPACES.replace_all(&url, "").to_string();

        url = HandyUrl::add_default_scheme_if_needed(&url);

        url = RE_MULTIPLE_PROTOCOLS
            .replace(&url, |caps: &regex::Captures| format!("{}", &caps[1]))
            .to_string();

        let split_url = match SplitResult::parse(String::from(&url)) {
            Ok(su) => su,
            Err(e) => return Err(e),
        };

        let (host, port) = match split_url.netloc {
            Some(nl) => url_split::split_netloc(nl),
            None => (None, None),
        };
        let last_delimiter =
            match &split_url.query.is_none() == &true && &url.clone().ends_with("?") == &true {
                true => Some(String::from("?")),
                false => None,
            };

        Ok(Self {
            scheme: split_url.scheme,
            auth_user: None,
            auth_pass: None,
            host,
            port,
            path: split_url.path,
            query: split_url.query,
            hash: split_url.fragment,
            last_delimiter,
        })
    }

    pub fn get_public_suffix(&self) -> Option<String> {
        let suffix = tld_extract::SuffixList::new(TLD_SOURCE, false, None);
        let mut extract = tld_extract::TLDExtract::new(suffix, true).unwrap();

        if let Some(host) = &self.host {
            return match extract.extract(host) {
                Ok(t) => t.registered_domain,
                Err(_) => None,
            };
        }

        None
    }

    pub fn get_public_prefix(&self) -> Option<String> {
        let suffix = tld_extract::SuffixList::new(TLD_SOURCE, false, None);
        let mut extract = tld_extract::TLDExtract::new(suffix, true).unwrap();

        if let Some(host) = &self.host {
            return match extract.extract(host) {
                Ok(t) => t.subdomain,
                Err(_) => None,
            };
        }

        None
    }

    pub fn get_url(&self, options: &SurtrOptions) -> Result<String, String> {
        let mut host_src = self.host.clone();

        // Host
        if self.host.is_some() {
            if options.get_or("public_suffix", false) {
                host_src = self.get_public_suffix();
            }
            if options.get_or("surt", false) {
                host_src = Some(host_to_surt(
                    host_src.unwrap(),
                    options.get_or("reverse_ipaddr", true),
                ))
            }
        }

        // Scheme
        let mut scheme_parts: Vec<&str> = vec![];
        if options.get_or("with_scheme", true) {
            match &self.scheme {
                Some(sch) => scheme_parts.push(&sch),
                None => return Err("no parsed scheme".to_string()),
            };

            scheme_parts.push(":");

            if host_src.is_some() {
                if scheme_parts[0] != "dns" {
                    scheme_parts.push("//");
                }
                if options.get_or("surt", false) {
                    scheme_parts.push("(");
                }
            }
        } else if host_src.is_none() {
            match &self.scheme {
                Some(sch) => scheme_parts.push(&sch),
                None => return Err("no parsed scheme".to_string()),
            };
            scheme_parts.push(":");
        } else {
            scheme_parts.push("");
        }

        let mut output_string: String = scheme_parts.join("");

        if let Some(host) = host_src {
            // Auth
            if let Some(user) = &self.auth_user {
                output_string = format!("{}{}", output_string, user);

                if let Some(pass) = &self.auth_pass {
                    output_string = format!("{}{}", output_string, pass);
                }

                output_string = format!("{}@", output_string);
            }

            output_string = format!("{}{}", output_string, host);

            if let Some(port) = &self.port {
                output_string = format!("{}:{}", output_string, port);
            }

            if options.get_or("surt", false) {
                if options.get_or("trailing_comma", false) {
                    output_string = format!("{},", output_string);
                }
                output_string = format!("{})", output_string);
            }
        }

        if let Some(path) = &self.path {
            output_string = format!("{}{}", output_string, path);
        } else if self.query.is_some() || self.hash.is_some() {
            output_string = format!("{}/", output_string);
        }

        if let Some(query) = &self.query {
            output_string = format!("{}?{}", output_string, query);
        }
        if let Some(hash) = &self.hash {
            output_string = format!("{}#{}", output_string, hash);
        }

        if let Some(ld) = &self.last_delimiter {
            output_string = format!("{}{}", output_string, ld);
        }

        Ok(output_string)
    }
}

impl Display for HandyUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut middle = format!(
            "    scheme: \"{}\"",
            self.scheme.clone().unwrap_or("Unconfigured".to_string())
        );
        middle = format!(
            "{}\n    auth_user: \"{}\"",
            middle,
            self.auth_user.clone().unwrap_or("Unconfigured".to_string())
        );
        middle = format!(
            "{}\n    auth_pass: \"{}\"",
            middle,
            self.auth_pass.clone().unwrap_or("Unconfigured".to_string())
        );
        middle = format!(
            "{}\n    host: \"{}\"",
            middle,
            self.host.clone().unwrap_or("Unconfigured".to_string())
        );
        middle = format!(
            "{}\n    port: \"{}\"",
            middle,
            self.port.clone().unwrap_or("Unconfigured".to_string())
        );
        middle = format!(
            "{}\n    path: \"{}\"",
            middle,
            self.path.clone().unwrap_or("Unconfigured".to_string())
        );
        middle = format!(
            "{}\n    query: \"{}\"",
            middle,
            self.query.clone().unwrap_or("Unconfigured".to_string())
        );
        middle = format!(
            "{}\n    hash: \"{}\"",
            middle,
            self.hash.clone().unwrap_or("Unconfigured".to_string())
        );
        middle = format!(
            "{}\n    last_delimiter: \"{}\"",
            middle,
            self.last_delimiter
                .clone()
                .unwrap_or("Unconfigured".to_string())
        );

        write!(f, "HandyUrl {{\n{}\n}}", middle)
    }
}

#[cfg(test)]
mod tests {
    use crate::options::SurtrOptions;

    use super::*;

    #[test]
    fn test_handyurl_parse() {
        let opts = SurtrOptions::default();

        // These tests come from URLParserTest.java
        assert_eq!(
            HandyUrl::parse("http://www.archive.org/index.html#foo")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org/index.html#foo"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org/")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org/"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org?")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org?"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org:8080/index.html?query#foo")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org:8080/index.html?query#foo"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org:8080/index.html?#foo")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org:8080/index.html#foo"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org:8080?#foo")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org:8080/#foo"
        );
        assert_eq!(
            HandyUrl::parse("http://bücher.ch:8080?#foo")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://bücher.ch:8080/#foo"
        );
        assert_eq!(
            HandyUrl::parse("dns:bücher.ch")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "dns:bücher.ch"
        );
        // assert_eq!(HandyUrl::parse("http://bücher.ch:8080?#foo").unwrap().get_url(&opt).unwrap(), "http://b\xfccher.ch:8080/#foo");
        // assert_eq!(HandyUrl::parse("dns:bücher.ch").unwrap().get_url(&opt).unwrap(), "dns:b\xfccher.ch");

        // From Tymm:
        assert_eq!(
            HandyUrl::parse("http:////////////////www.vikings.com")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.vikings.com/"
        );
        assert_eq!(
            HandyUrl::parse("http://https://order.1and1.com")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "https://order.1and1.com"
        );

        // From Common Crawl, host ends with ":" without a port number
        assert_eq!(
            HandyUrl::parse(
                "http://mineral.galleries.com:/minerals/silicate/chabazit/chabazit.htm"
            )
            .unwrap()
            .get_url(&opts)
            .unwrap(),
            "http://mineral.galleries.com/minerals/silicate/chabazit/chabazit.htm"
        );

        assert_eq!(
            HandyUrl::parse("mailto:bot@archive.org")
                .unwrap()
                .scheme
                .unwrap(),
            "mailto".to_string()
        );
        assert_eq!(
            HandyUrl::parse("mailto:bot@archive.org")
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "mailto:bot@archive.org"
        );
    }

    #[test]
    fn test_get_public_suffix() {
        // These tests are based off the ones found in HandyURLTest.java
        let mut url = HandyUrl {
            scheme: None,
            auth_user: None,
            auth_pass: None,
            host: None,
            port: None,
            path: None,
            query: None,
            hash: None,
            last_delimiter: None,
        };

        url.host = Some("www.fool.com".to_string());
        assert_eq!(&url.get_public_suffix().unwrap(), "fool.com");

        url.host = Some("www.amazon.co.uk".to_string());
        assert_eq!(&url.get_public_suffix().unwrap(), "amazon.co.uk");

        url.host = Some("www.images.amazon.co.uk".to_string());
        assert_eq!(&url.get_public_suffix().unwrap(), "amazon.co.uk");

        url.host = Some("funky-images.fancy.co.jp".to_string());
        assert_eq!(&url.get_public_suffix().unwrap(), "fancy.co.jp");
    }

    #[test]
    fn test_get_public_prefix() {
        // These tests are based off the ones found in HandyURLTest.java
        let mut url = HandyUrl {
            scheme: None,
            auth_user: None,
            auth_pass: None,
            host: None,
            port: None,
            path: None,
            query: None,
            hash: None,
            last_delimiter: None,
        };

        url.host = Some("www.fool.com".to_string());
        assert_eq!(&url.get_public_prefix().unwrap(), "www");

        url.host = Some("www.amazon.co.uk".to_string());
        assert_eq!(&url.get_public_prefix().unwrap(), "www");

        url.host = Some("www.images.amazon.co.uk".to_string());
        assert_eq!(&url.get_public_prefix().unwrap(), "www.images");

        url.host = Some("funky-images.fancy.co.jp".to_string());
        assert_eq!(&url.get_public_prefix().unwrap(), "funky-images");
    }
}
