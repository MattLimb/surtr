use std::fmt::Display;

use lazy_static::lazy_static;
use regex::Regex;
use tld_extract::{SuffixList, TLDExtract};

use crate::{
    error::SurtrError,
    options::SurtrOptions,
    regex_transformer::host_to_surt,
    url_split::SplitResult,
};

const TLD_SOURCE: tld_extract::Source = tld_extract::Source::Snapshot;

lazy_static! {
    // These Regexes expect here, because they should always compile. The system doesn't work without them compiling
    // so we should panic if they cannot compile.
    static ref RE_MULTIPLE_PROTOCOLS: Regex = Regex::new(r#"^(https?://)+"#).expect("Failed to compile Multiple Protocols Regex");
    static ref RE_HAS_PROTOCOL: Regex = Regex::new(r#"^([a-zA-Z][a-zA-Z0-9\+\-\.]*):"#).expect("Failed to compile Has Protocol Regex");
    static ref RE_SPACES: Regex = Regex::new(r#"[\n\r\t]"#).expect("Failed to compile Spaces Regex");
}

/// A struct which decomiles and stores all parts of the URL, ready for processing.
/// 
/// # Examples
/// 
/// ```rust
/// use surtr::{HandyUrl, SurtrOptions};
/// 
/// let url = "http://www.example.com/";
/// let mut options = SurtrOptions::default();
/// 
/// // These options are set by the `surtr::surt()` function if not set by the user.
/// options.set("surt", true);
/// options.set("with_scheme", false);
/// 
/// let handy_url = HandyUrl::parse(url, &options).unwrap();
/// 
/// println!("{handy_url}");
/// // HandyURL {
/// //     scheme: "http",
/// //     auth_user: None,
/// //     auth_pass: None,
/// //     host: Some("www.example.com"),
/// //     port: None,
/// //     path: Some("/"),
/// //     query: None,
/// //     hash: None,
/// //     last_delimiter: None,
/// // }
/// 
/// // This example differs from the `surtr::surt()` function's output, because no 
/// // canonicalization is being performed.
/// assert_eq!(handy_url.get_url(&options), Ok("com,example,www)/".to_string()));
/// ```
#[derive(Debug, Clone)]
pub struct HandyUrl {
    /// http, ftp, dns -> The protocol being used in the URL. Typically before the `://`.
    pub scheme: Option<String>,
    /// Basic Authentication Username
    /// By default this username is stripped from the URL for security reasons during
    /// canonicalization.
    pub auth_user: Option<String>,
    /// Basic Authentication Password.
    /// By default this username is stripped from the URL for security reasons during
    /// canonicalization.
    pub auth_pass: Option<String>,
    /// The host portion of the URL.
    /// This is the domain name or IP address of the server.
    pub host: Option<String>,
    /// The port portion of the URL.
    /// This is the port number of the server.
    pub port: Option<String>,
    /// The path portion of the URL.
    /// This is the path of the resource being requested.
    pub path: Option<String>,
    /// The query paramters. Always seperated from the path by the `?`.
    pub query: Option<String>,
    /// The hash parameters. Similar to the query parameters, however seperated by the '#'.
    /// This signals options which should ONLY be availible to the Client. NOT the webserver.
    pub hash: Option<String>,
    /// Internal option to determine if a blank operator was present. If it was, we should add it to the SURT for accuracy.
    pub last_delimiter: Option<String>,
}

impl HandyUrl {
    // Supplies the default scheme of HTTP or the given Scheme.
    fn add_default_scheme_if_needed(url: &str) -> String {
        if RE_HAS_PROTOCOL.is_match(url) {
            return String::from(url);
        }

        format!("http://{}", url)
    }

    /// Parse a given URL into a HandyUrl Struct.
    /// 
    /// # Arguments
    /// 
    /// `raw_url` - A String containing the URL to parse.
    /// `options` -> A SurtrOptions struct additional options when parsing.
    /// 
    /// # Returns
    /// 
    /// A Result containing a HandyUrl Struct or a SurtrError.
    /// 
    /// # Errors
    /// 
    /// `SurtrError::UrlParseError(String)` - A `SurtrError` indicating that the URL could not be parsed.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use surtr::{HandyUrl,SurtrOptions};
    /// 
    /// let url = "http://www.example.com/";
    /// let handy_url = HandyUrl::parse(url, &SurtrOptions::default()).expect("URL was not parsed.");
    /// 
    /// println!("{handy_url}"); // ->
    /// // HandyUrl {
    /// //    scheme: "http"
    /// //    auth_user: "None"
    /// //    auth_pass: "None"
    /// //    host: "www.example.com"
    /// //    port: "None"
    /// //    path: "/"
    /// //    query: "None"
    /// //    hash: "None"
    /// //    last_delimiter: "None"
    /// // }
    /// ```
    pub fn parse(raw_url: &str, options: &SurtrOptions) -> Result<Self, SurtrError> {
        let mut url: String = raw_url.trim().to_string();
        url = RE_SPACES.replace_all(&url, "").to_string();

        url = HandyUrl::add_default_scheme_if_needed(&url);

        url = RE_MULTIPLE_PROTOCOLS
            .replace(&url, |caps: &regex::Captures| (caps[1]).to_string())
            .to_string();

        let split_url = SplitResult::parse(String::from(&url), options)?;

        // let (host, port) = match split_url.netloc {
        //     Some(nl) => url_split::split_netloc(nl),
        //     None => (None, None),
        // };
        let last_delimiter =
            match split_url.query.is_none() && url.clone().ends_with("?") {
                true => Some(String::from("?")),
                false => None,
            };

        Ok(Self {
            scheme: split_url.scheme,
            auth_user: split_url.netloc.auth_user,
            auth_pass: split_url.netloc.auth_pass,
            host: split_url.netloc.domain,
            port: split_url.netloc.port,
            path: split_url.path,
            query: split_url.query,
            hash: split_url.fragment,
            last_delimiter,
        })
    }

    // Use the public TLD Sources to identify the registered domain of a given Host.
    // This is used to discard subdomains from the SURT.
    fn get_public_suffix(&self) -> Option<String> {
        let suffix = SuffixList::new(TLD_SOURCE, false, None);
        let mut extract = TLDExtract::new(suffix, true).expect("TLD Extract failed to compile successfully.");

        if let Some(host) = &self.host {
            return match extract.extract(host) {
                Ok(t) => t.registered_domain,
                Err(_) => None,
            };
        }

        None
    }

    // Use the public TLD Sources to identify the subdomain of a given Host.
    // This exists as a compatibility with IA's version. This method is unused in Surtr.
    fn _get_public_prefix(&self) -> Option<String> {
        let suffix = SuffixList::new(TLD_SOURCE, false, None);
        let mut extract = TLDExtract::new(suffix, true).expect("TLD Extract failed to compile successfully.");

        if let Some(host) = &self.host {
            return match extract.extract(host) {
                Ok(t) => t.subdomain,
                Err(_) => None,
            };
        }

        None
    }

    /// Recompile the URL as a String, according to the set of user defined options.
    /// 
    /// # Arguments
    /// 
    /// `options` - A `SurtrOptions` struct containing the user defined options.
    ///             This function uses default values for each option.
    /// 
    /// # Returns
    ///
    /// A Result containing a String of the compiled URL or SURT.
    /// 
    /// # Errors
    /// 
    /// `SurtrError::NoSchemeFoundError` - A `SurtrError` indicating that a Scheme was required, but was not present in the URL.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use surtr::{HandyUrl, SurtrOptions};
    /// 
    /// let url = "https://example.com?hello=world";
    /// let mut options = SurtrOptions::default();
    /// options.set("surt", true);
    /// options.set("with_scheme", true);
    /// 
    /// let handy_url = HandyUrl::parse(url, &options).unwrap();
    /// 
    /// assert_eq!(handy_url.get_url(&options), Ok("https://(com,example)/?hello=world".to_string()));
    /// ```
    pub fn get_url(&self, options: &SurtrOptions) -> Result<String, SurtrError> {
        let mut host_src = self.host.clone();

        // Host
        if options.get_or("public_suffix", false) && host_src.is_some() {
            host_src = self.get_public_suffix();
        }
        if options.get_or("surt", false) {
            if let Some(hst) = host_src {
                host_src = Some(host_to_surt(
                    hst.clone(),
                    options.get_or("reverse_ipaddr", true),
                ))
            }
        }

        // Scheme
        let mut scheme_parts: Vec<&str> = vec![];
        if options.get_or("with_scheme", true) {
            match &self.scheme {
                Some(sch) => scheme_parts.push(sch),
                None => return Err(SurtrError::NoSchemeFoundError),
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
                Some(sch) => scheme_parts.push(sch),
                None => return Err(SurtrError::NoSchemeFoundError),
            };
            scheme_parts.push(":");
        } else {
            scheme_parts.push("");
        }

        let mut output_string: String = scheme_parts.join("");

        if let Some(host) = &host_src {
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
            self.scheme.clone().unwrap_or("None".to_string())
        );
        middle = format!(
            "{}\n    auth_user: \"{}\"",
            middle,
            self.auth_user.clone().unwrap_or("None".to_string())
        );
        middle = format!(
            "{}\n    auth_pass: \"{}\"",
            middle,
            self.auth_pass.clone().unwrap_or("None".to_string())
        );
        middle = format!(
            "{}\n    host: \"{}\"",
            middle,
            self.host.clone().unwrap_or("None".to_string())
        );
        middle = format!(
            "{}\n    port: \"{}\"",
            middle,
            self.port.clone().unwrap_or("None".to_string())
        );
        middle = format!(
            "{}\n    path: \"{}\"",
            middle,
            self.path.clone().unwrap_or("None".to_string())
        );
        middle = format!(
            "{}\n    query: \"{}\"",
            middle,
            self.query.clone().unwrap_or("None".to_string())
        );
        middle = format!(
            "{}\n    hash: \"{}\"",
            middle,
            self.hash.clone().unwrap_or("None".to_string())
        );
        middle = format!(
            "{}\n    last_delimiter: \"{}\"",
            middle,
            self.last_delimiter
                .clone()
                .unwrap_or("None".to_string())
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
            HandyUrl::parse("http://www.archive.org/index.html#foo", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org/index.html#foo"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org/", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org/"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org?", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org?"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org:8080/index.html?query#foo", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org:8080/index.html?query#foo"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org:8080/index.html?#foo", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org:8080/index.html#foo"
        );
        assert_eq!(
            HandyUrl::parse("http://www.archive.org:8080?#foo", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.archive.org:8080/#foo"
        );
        assert_eq!(
            HandyUrl::parse("http://bücher.ch:8080?#foo", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://bücher.ch:8080/#foo"
        );
        assert_eq!(
            HandyUrl::parse("dns:bücher.ch", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "dns:bücher.ch"
        );
        // assert_eq!(HandyUrl::parse("http://bücher.ch:8080?#foo").unwrap().get_url(&opts).unwrap(), "http://b\xfccher.ch:8080/#foo");
        // assert_eq!(HandyUrl::parse("dns:bücher.ch").unwrap().get_url(&opt).unwrap(), "dns:b\xfccher.ch");

        // From Tymm:
        assert_eq!(
            HandyUrl::parse("http:////////////////www.vikings.com", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "http://www.vikings.com/"
        );
        assert_eq!(
            HandyUrl::parse("http://https://order.1and1.com", &opts)
                .unwrap()
                .get_url(&opts)
                .unwrap(),
            "https://order.1and1.com"
        );

        // From Common Crawl, host ends with ":" without a port number
        assert_eq!(
            HandyUrl::parse(
                "http://mineral.galleries.com:/minerals/silicate/chabazit/chabazit.htm", &opts
            )
            .unwrap()
            .get_url(&opts)
            .unwrap(),
            "http://mineral.galleries.com/minerals/silicate/chabazit/chabazit.htm"
        );

        assert_eq!(
            HandyUrl::parse("mailto:bot@archive.org", &opts)
                .unwrap()
                .scheme
                .unwrap(),
            "mailto".to_string()
        );
        assert_eq!(
            HandyUrl::parse("mailto:bot@archive.org", &opts)
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
        assert_eq!(&url._get_public_prefix().unwrap(), "www");

        url.host = Some("www.amazon.co.uk".to_string());
        assert_eq!(&url._get_public_prefix().unwrap(), "www");

        url.host = Some("www.images.amazon.co.uk".to_string());
        assert_eq!(&url._get_public_prefix().unwrap(), "www.images");

        url.host = Some("funky-images.fancy.co.jp".to_string());
        assert_eq!(&url._get_public_prefix().unwrap(), "funky-images");
    }
}
