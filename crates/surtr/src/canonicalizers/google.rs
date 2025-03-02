use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use ascii::AsAsciiStr;
use idna::domain_to_ascii;
use lazy_static::lazy_static;
use regex::Regex;

use crate::canonicalizers::url_encoding::{escape_once, minimal_escape, unescape_repeatedly};
use crate::error::SurtrError;
use crate::handy_url::HandyUrl;
use crate::options::SurtrOptions;

lazy_static! {
    static ref RE_OCTAL_IP: Regex =
        Regex::new(r#"^(0[0-7]*)(\.[0-7]+)?(\.[0-7]+)?(\.[0-7]+)?$"#).unwrap();
    static ref RE_DECIMAL_IP: Regex =
        Regex::new(r#"^([1-9][0-9]*)(\.[0-9]+)?(\.[0-9]+)?(\.[0-9]+)?$"#).unwrap();
}

pub fn canonicalize(url_input: HandyUrl, _options: &SurtrOptions) -> Result<HandyUrl, SurtrError> {
    let mut url: HandyUrl = url_input;
    url.hash = None;

    if let Some(auth_user) = url.auth_user {
        url.auth_user = Some(minimal_escape(auth_user)?);
    }
    if let Some(auth_pass) = url.auth_pass {
        url.auth_pass = Some(minimal_escape(auth_pass)?);
    }
    if let Some(query) = url.query {
        url.query = Some(minimal_escape(query)?);
    }

    if let Some(host) = url.host {
        // I should be able to always unwrap this because Strings in
        // Rust are utf-8. Since I put one in, the only way it isn't is if
        // the library has done something.
        let mut tmp_host = unescape_repeatedly(host).unwrap();

        if tmp_host.as_ascii_str().is_err() {
            match domain_to_ascii(&tmp_host) {
                Ok(s) => tmp_host = s.to_string(),
                Err(_) => (),
            };
        }

        tmp_host = tmp_host.replace("..", ".").trim_matches('.').to_string();
        if let Some(ip) = attempt_ip_formats(tmp_host.clone()) {
            tmp_host = ip;
        } else {
            tmp_host = escape_once(tmp_host.to_lowercase())
        }

        url.host = Some(tmp_host)
    }

    let mut path: Option<String> = url.path.clone();

    // Emulate the start of unescapeRepeaty
    if let Some(p) = path {
        path = Some(unescape_repeatedly(p).unwrap());
    }

    if url.host.is_some() {
        // Emulate the start of normalizePath
        if let Some(pth) = path {
            path = Some(normalize_path(pth));
        } else {
            path = Some(normalize_path("/".to_string()));
        }
    }

    if let Some(pth) = path {
        url.path = Some(escape_once(pth))
    }

    Ok(url)
}

fn coerce_ip(input: &str) -> String {
    let mut parts: Vec<u8> = vec![];

    for f in input.split(".") {
        let decimal_int: u32 = u32::from_str_radix(f, 10).unwrap();
        let prefix: &str = match decimal_int > 255 || decimal_int < 16 {
            true => "0",
            false => "",
        };
        let as_hex: String = format!("{}{:X}", prefix, decimal_int);

        parts.extend(as_hex.into_bytes());
    }

    let as_dec: Vec<String> = parts
        .chunks(2)
        .map(|chunk| {
            u32::from_str_radix(unsafe { &String::from_utf8_unchecked(chunk.to_vec()) }, 16)
                .unwrap()
                .to_string()
        })
        .collect();
    as_dec.join(".")
}

pub fn attempt_ip_formats(host: String) -> Option<String> {
    if let Ok(host_digit) = u32::from_str_radix(&host, 10) {
        let ip_addr = Ipv4Addr::from(host_digit & 0xffffffff);
        return Some(ip_addr.to_string());
    } else if let Ok(host_digit) = u128::from_str_radix(&host, 10) {
        let ip_addr = Ipv6Addr::from(host_digit & 0xffffffff);
        return match ip_addr.to_ipv4() {
            Some(ip) => Some(ip.to_string()),
            None => None,
        };
    } else {
        if RE_DECIMAL_IP.is_match(&host) {
            return match IpAddr::from_str(&coerce_ip(&host)) {
                Ok(ip) => Some(ip.to_string()),
                Err(_) => None,
            };
        } else if RE_OCTAL_IP.is_match(&host) {
            let parts: Vec<String> = host
                .split('.')
                .into_iter()
                .map(|f| u32::from_str_radix(f, 8).unwrap().to_string())
                .collect();

            return match IpAddr::from_str(&parts.join(".")) {
                Ok(ip) => Some(ip.to_string()),
                Err(_) => None,
            };
        }
    }

    None
}

fn normalize_path(input: String) -> String {
    let paths = input.split("/");
    let mut kept_paths: Vec<&str> = vec![];
    let mut first: bool = true;

    for p in paths {
        if first {
            first = false;
            continue;
        } else if p == "." {
            continue;
        } else if p == ".." {
            if kept_paths.len() > 0 {
                kept_paths.pop();
            } else {
                kept_paths.push(p);
            }
        } else {
            kept_paths.push(p);
        }
    }

    let mut output: String = "/".to_string();

    let kept_length = kept_paths.len();
    if kept_length > 0 {
        for i in 0..(kept_length - 1) {
            let p = kept_paths[i];
            if p.len() > 0 {
                output = format!("{}{}/", output, p);
            }
        }

        output += kept_paths[kept_length - 1]
    }

    output
}

#[cfg(test)]
mod tests {
    use crate::options::SurtrOptions;

    use super::*;

    #[test]
    fn test_google_canonicalize() {
        let def_options = SurtrOptions::default();
        // The tests are copied from GoogleURLCanonicalizerTest.java
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://host/%25%32%35").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://host/%25"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://host/%25%32%35%25%32%35").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://host/%25%25"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://host/%2525252525252525").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://host/%25"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://host/asdf%25%32%35asd").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://host/asdf%25asd"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://host/%%%25%32%35asd%%").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://host/%25%25%25asd%25%25"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.google.com/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.google.com/"
        );
        assert_eq!(canonicalize(HandyUrl::parse("http://%31%36%38%2e%31%38%38%2e%39%39%2e%32%36/%2E%73%65%63%75%72%65/%77%77%77%2E%65%62%61%79%2E%63%6F%6D/").unwrap(), &def_options).unwrap().get_url(&def_options).unwrap(), "http://168.188.99.26/.secure/www.ebay.com/");
        assert_eq!(canonicalize(HandyUrl::parse("http://195.127.0.11/uploads/%20%20%20%20/.verify/.eBaysecure=updateuserdataxplimnbqmn-xplmvalidateinfoswqpcmlx=hgplmcx/").unwrap(), &def_options).unwrap().get_url(&def_options).unwrap(), "http://195.127.0.11/uploads/%20%20%20%20/.verify/.eBaysecure=updateuserdataxplimnbqmn-xplmvalidateinfoswqpcmlx=hgplmcx/");
        assert_eq!(canonicalize(HandyUrl::parse("http://host%23.com/%257Ea%2521b%2540c%2523d%2524e%25f%255E00%252611%252A22%252833%252944_55%252B").unwrap(), &def_options).unwrap().get_url(&def_options).unwrap(), "http://host%23.com/~a!b@c%23d$e%25f^00&11*22(33)44_55+");
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://3279880203/blah").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://195.127.0.11/blah"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.google.com/blah/..").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.google.com/"
        );
        assert_eq!(
            canonicalize(HandyUrl::parse("www.google.com/").unwrap(), &def_options)
                .unwrap()
                .get_url(&def_options)
                .unwrap(),
            "http://www.google.com/"
        );
        assert_eq!(
            canonicalize(HandyUrl::parse("www.google.com").unwrap(), &def_options)
                .unwrap()
                .get_url(&def_options)
                .unwrap(),
            "http://www.google.com/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.evil.com/blah#frag").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.evil.com/blah"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.GOOgle.com/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.google.com/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.google.com.../").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.google.com/"
        );

        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.google.com/foo\tbar\rbaz\n2").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.google.com/foobarbaz2"
        );

        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.google.com/q?").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.google.com/q?"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.google.com/q?r?").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.google.com/q?r?"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.google.com/q?r?s").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.google.com/q?r?s"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://evil.com/foo#bar#baz").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://evil.com/foo"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://evil.com/foo;").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://evil.com/foo;"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://evil.com/foo?bar;").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://evil.com/foo?bar;"
        );

        //This test case differs from the Java version. The Java version returns
        //"http://%01%80.com/" for this case. If idna/punycode encoding of a hostname
        //is not possible, the python version encodes unicode domains as utf-8 before
        //percent encoding, so we get "http://%01%C2%80.com/");
        // assert_eq!(print(canonicalize(HandyUrl::parse("http://\u0001\u0080.com/").unwrap(), &def_options).unwrap().get_url(&def_options).unwrap().unwrap(), &def_options) http://%01%C2%80.com/
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://\u{0001}\u{0080}.com/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://%01%C2%80.com/"
        );

        //Add these unicode tests:
        // assert_eq!(print(canonicalize(HandyUrl::parse("B\xfccher.ch:8080").unwrap(), &def_options).unwrap().get_url(&def_options).unwrap().unwrap(), &def_options) http://xn--bcher-kva.ch:8080/
        // assert_eq!(print(canonicalize(HandyUrl::parse("☃.com").unwrap(), &def_options).unwrap().get_url(&def_options).unwrap().unwrap(), &def_options), http://xn--n3h.com/
        // assert_eq!(canonicalize(HandyUrl::parse("B\xfccher.ch:8080").unwrap(), &def_options).unwrap().get_url(&def_options).unwrap(), "http://xn--bcher-kva.ch:8080/");
        assert_eq!(
            canonicalize(HandyUrl::parse("☃.com").unwrap(), &def_options)
                .unwrap()
                .get_url(&def_options)
                .unwrap(),
            "http://xn--n3h.com/"
        );

        //Add these percent-encoded unicode tests
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.t%EF%BF%BD%04.82.net/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.t%EF%BF%BD%04.82.net/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://notrailingslash.com").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://notrailingslash.com/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.gotaport.com:1234/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.gotaport.com:1234/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("  http://www.google.com/  ").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://www.google.com/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http:// leadingspace.com/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://%20leadingspace.com/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://%20leadingspace.com/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://%20leadingspace.com/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("%20leadingspace.com/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://%20leadingspace.com/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("https://www.securesite.com/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "https://www.securesite.com/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://host.com/ab%23cd").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://host.com/ab%23cd"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://host.com//twoslashes?more//slashes").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://host.com/twoslashes?more//slashes"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("mailto:foo@example.com").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "mailto:foo@example.com"
        );
    }

    #[test]
    fn test_attempt_ip_formats() {
        // The tests are copied from GoogleURLCanonicalizerTest.java
        // assert_eq!(attempt_ip_formats(None).is_none()); // Test cannot be performed - this should be handled upstream of this function

        assert!(attempt_ip_formats("www.foo.com".to_string()).is_none());
        assert_eq!(
            &attempt_ip_formats("127.0.0.1".to_string()).unwrap(),
            "127.0.0.1"
        );
        assert_eq!(
            &attempt_ip_formats("017.0.0.1".to_string()).unwrap(),
            "15.0.0.1"
        );
        assert_eq!(
            &attempt_ip_formats("168.188.99.26".to_string()).unwrap(),
            "168.188.99.26"
        );
        // java version returns null, ours returns the correct ipv4
        assert_eq!(
            &attempt_ip_formats("10.0.258".to_string()).unwrap(),
            "10.0.1.2"
        );
        assert!(attempt_ip_formats("1.2.3.256".to_string()).is_none());

        // ARC files from the wayback machine"s liveweb proxy contain numeric
        // hostnames > 2^32 for some reason. We"ll copy the behavior of the java code.
        assert_eq!(
            &attempt_ip_formats("39024579298".to_string()).unwrap(),
            "22.11.210.226"
        );
    }
}
