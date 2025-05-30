//! A Rust based Sort-friendly URI Reordering Transform.
//! 
//! This crate is a Rust port of the [SURT] implementation, first produced by [The Internet Archive].
//! 
//! The crate intends to be as compatible as possible to the original [IA implementation]. 
//! 
//! Currently, custom Canonicalization functions are missing from the Public Interface.
//! They are being considered as part of a future release.
//! 
//! [SURT]: http://crawler.archive.org/articles/user_manual/glossary.html#surt
//! [The Internet Archive]: https://github.com/internetarchive/surt
//! [IA implementation]: https://github.com/internetarchive/surt

mod error;
mod handy_url;
mod options;
mod canonicalizers;
mod regex_transformer;
pub mod url_split;

pub use options::SurtrOptions;
pub use handy_url::HandyUrl;
pub use error::SurtrError;


/// Returns the Result of a SURT operation.
/// 
/// # Arguments
/// 
/// * `url` - The URL to be transformed.
/// * `options` - The options to be used for the transformation.
/// 
/// # Returns
/// 
/// A Result containing the transformed URL, or an error if the URL is invalid.
/// 
/// # Examples
/// 
/// Basic Example:
/// 
/// ```rust
/// use surtr::surt;
/// 
/// let url = "http://www.example.com/";
/// 
/// let result = surt(url, None).unwrap();
/// println!("{result}"); // -> com,example)/
/// ```
/// 
/// Using Options to Adjust the output:
/// 
/// ```rust
/// use surtr::{surt, SurtrOptions};
/// 
/// let url = "http://www.example.com/";
/// let mut options = SurtrOptions::default();
/// options.set("with_scheme", true);
/// 
/// let result = surt(url, Some(options)).unwrap();
/// println!("{result}"); // -> http://(com,example)/
/// ```
pub fn surt(
    url: &str,
    options: Option<options::SurtrOptions>,
) -> Result<String, error::SurtrError> {
    let mut s_options: options::SurtrOptions = match options {
        Some(opt) => opt.clone(),
        None => options::SurtrOptions::default(),
    };

    // Set Default
    if s_options.get("surt").is_none() {
        s_options.set("surt", true);
    }
    if s_options.get("with_scheme").is_none() {
        s_options.set("with_scheme", false);
    }

    // Default
    _surt(url, &s_options)
}

fn _surt(
    url: &str,
    options: &options::SurtrOptions,
) -> Result<String, error::SurtrError> {
    // Hardcoded Workaround for filedesc
    if url.starts_with("filedesc") {
        return Ok(url.to_string());
    }

    // Parse URL
    let mut hurl = handy_url::HandyUrl::parse(url, options)?;

    // Canonicalize URL
    hurl = canonicalizers::default::canonicalize(hurl, options)?;

    // Build String URL
    hurl.get_url(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surt() {
        // These tests are from WaybackURLKeyMakerTest.java
        // assert_eq!(surt("", None, None).unwrap(), "-");
        assert_eq!(
            surt("filedesc:foo.arc.gz", None).unwrap(),
            "filedesc:foo.arc.gz"
        );
        assert_eq!(
            surt("filedesc:/foo.arc.gz", None).unwrap(),
            "filedesc:/foo.arc.gz"
        );
        assert_eq!(
            surt("filedesc://foo.arc.gz", None).unwrap(),
            "filedesc://foo.arc.gz"
        );
        assert_eq!(
            surt("warcinfo:foo.warc.gz", None).unwrap(),
            "warcinfo:foo.warc.gz"
        );
        assert_eq!(
            surt("dns:alexa.com", None).unwrap(),
            "dns:alexa.com"
        );
        assert_eq!(
            surt("dns:archive.org", None).unwrap(),
            "dns:archive.org"
        );

        assert_eq!(
            surt("http://www.archive.org/", None).unwrap(),
            "org,archive)/"
        );
        assert_eq!(
            surt("http://archive.org/", None).unwrap(),
            "org,archive)/"
        );
        assert_eq!(
            surt("http://archive.org/goo/", None).unwrap(),
            "org,archive)/goo"
        );
        assert_eq!(
            surt("http://archive.org/goo/?", None).unwrap(),
            "org,archive)/goo"
        );
        assert_eq!(
            surt("http://archive.org/goo/?b&a", None).unwrap(),
            "org,archive)/goo?a&b"
        );
        assert_eq!(
            surt("http://archive.org/goo/?a=2&b&a=1", None).unwrap(),
            "org,archive)/goo?a=1&a=2&b"
        );

        // TODO: Default options being overriden here - implement them.

        // trailing comma mode
        let mut trailing_comma = options::SurtrOptions::default();
        trailing_comma.set("trailing_comma", true);
        trailing_comma.set("surt", true);
        trailing_comma.set("with_scheme", false);

        assert_eq!(
            surt(
                "http://archive.org/goo/?a=2&b&a=1",
                Some(trailing_comma.clone())
            )
            .unwrap(),
            "org,archive,)/goo?a=1&a=2&b"
        );
        assert_eq!(
            surt("dns:archive.org", Some(trailing_comma.clone())).unwrap(),
            "dns:archive.org"
        );
        assert_eq!(
            surt("warcinfo:foo.warc.gz", Some(trailing_comma)).unwrap(),
            "warcinfo:foo.warc.gz"
        );

        // PHP session id:
        assert_eq!(surt("http://archive.org/index.php?PHPSESSID=0123456789abcdefghijklemopqrstuv&action=profile;u=4221", None).unwrap(), "org,archive)/index.php?action=profile;u=4221");

        // WHOIS url:
        assert_eq!(
            surt("whois://whois.isoc.org.il/shaveh.co.il", None).unwrap(),
            "il,org,isoc,whois)/shaveh.co.il"
        );

        // Yahoo web bug. See https://github.com/internetarchive/surt/issues/1
        assert_eq!(
            surt(
            "http://visit.webhosting.yahoo.com/visit.gif?&r=http%3A//web.archive.org/web/20090517140029/http%3A//anthonystewarthead.electric-chi.com/&b=Netscape%205.0%20%28Windows%3B%20en-US%29&s=1366x768&o=Win32&c=24&j=true&v=1.2",
            None).unwrap(),
            "com,yahoo,webhosting,visit)/visit.gif?&b=netscape%205.0%20(windows;%20en-us)&c=24&j=true&o=win32&r=http://web.archive.org/web/20090517140029/http://anthonystewarthead.electric-chi.com/&s=1366x768&v=1.2");
        // Simple customization:
        // Removing canonicalizer functions
        // assert_eq!(
        //     surt(
        //         "http://www.example.com/"),
        //         Some(Box::new(basic_canonicalizer)),
        //         None
        //     )
        //     .unwrap(),
        //     "com,example,www)/"
        // );
        assert_eq!(
            surt("mailto:foo@example.com", None).unwrap(),
            "mailto:foo@example.com"
        );

        let mut def_options = options::SurtrOptions::default();
        def_options.set("surt", true);
        def_options.set("with_scheme", false);

        assert_eq!(
            surt(
                "http://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);

                    tmp
                })
            )
            .unwrap(),
            "http://(com,example)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("host_massage", true);

                    tmp
                })
            )
            .unwrap(),
            "http://(com,example)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", false);

                    tmp
                })
            )
            .unwrap(),
            "com,example)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("trailing_comma", true);

                    tmp
                })
            )
            .unwrap(),
            "http://(com,example,)/"
        );
        assert_eq!(
            surt(
                "https://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("trailing_comma", true);

                    tmp
                })
            )
            .unwrap(),
            "https://(com,example,)/"
        );
        assert_eq!(
            surt(
                "ftp://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", false);
                    tmp.set("trailing_comma", true);

                    tmp
                })
            )
            .unwrap(),
            "com,example,)/"
        );
        assert_eq!(
            surt(
                "ftp://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", false);
                    tmp.set("trailing_comma", false);

                    tmp
                })
            )
            .unwrap(),
            "com,example)/"
        );
        assert_eq!(
            surt(
                "ftp://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("trailing_comma", true);

                    tmp
                })
            )
            .unwrap(),
            "ftp://(com,example,)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("host_massage", false);

                    tmp
                })
            )
            .unwrap(),
            "http://(com,example,www)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", false);
                    tmp.set("host_massage", false);

                    tmp
                })
            )
            .unwrap(),
            "com,example,www)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("trailing_comma", true);
                    tmp.set("host_massage", false);

                    tmp
                })
            )
            .unwrap(),
            "http://(com,example,www,)/"
        );
        assert_eq!(
            surt(
                "https://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("trailing_comma", true);
                    tmp.set("host_massage", false);

                    tmp
                })
            )
            .unwrap(),
            "https://(com,example,www,)/"
        );
        assert_eq!(
            surt(
                "ftp://www.example.com/",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("trailing_comma", true);
                    tmp.set("host_massage", false);

                    tmp
                })
            )
            .unwrap(),
            "ftp://(com,example,www,)/"
        );

        assert_eq!(
            surt(
                "mailto:foo@example.com",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);

                    tmp
                })
            )
            .unwrap(),
            "mailto:foo@example.com"
        );
        assert_eq!(
            surt(
                "mailto:foo@example.com",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("trailing_comma", true);

                    tmp
                })
            )
            .unwrap(),
            "mailto:foo@example.com"
        );
        assert_eq!(
            surt(
                "mailto:foo@example.com",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("trailing_comma", true);
                    tmp
                })
            )
            .unwrap(),
            "mailto:foo@example.com"
        );
        assert_eq!(
            surt(
                "dns:archive.org",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);

                    tmp
                })
            )
            .unwrap(),
            "dns:archive.org"
        );
        assert_eq!(
            surt(
                "dns:archive.org",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("trailing_comma", true);

                    tmp
                })
            )
            .unwrap(),
            "dns:archive.org"
        );
        assert_eq!(
            surt(
                "dns:archive.org",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("trailing_comma", true);
                    tmp
                })
            )
            .unwrap(),
            "dns:archive.org"
        );
        assert_eq!(
            surt(
                "whois://whois.isoc.org.il/shaveh.co.il",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);

                    tmp
                })
            )
            .unwrap(),
            "whois://(il,org,isoc,whois)/shaveh.co.il"
        );
        assert_eq!(
            surt(
                "whois://whois.isoc.org.il/shaveh.co.il",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("trailing_comma", true);

                    tmp
                })
            )
            .unwrap(),
            "il,org,isoc,whois,)/shaveh.co.il"
        );
        assert_eq!(
            surt(
                "whois://whois.isoc.org.il/shaveh.co.il",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("trailing_comma", true);

                    tmp
                })
            )
            .unwrap(),
            "whois://(il,org,isoc,whois,)/shaveh.co.il"
        );
        assert_eq!(
            surt(
                "warcinfo:foo.warc.gz",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("trailing_comma", true);

                    tmp
                })
            )
            .unwrap(),
            "warcinfo:foo.warc.gz"
        );
        assert_eq!(
            surt(
                "warcinfo:foo.warc.gz",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);

                    tmp
                })
            )
            .unwrap(),
            "warcinfo:foo.warc.gz"
        );

        assert_eq!(
            surt(
                "warcinfo:foo.warc.gz",
                Some({
                    let mut tmp = def_options.clone();
                    tmp.set("with_scheme", true);
                    tmp.set("trailing_comma", true);

                    tmp
                })
            )
            .unwrap(),
            "warcinfo:foo.warc.gz"
        );
    }

    #[test]
    #[should_panic]
    fn test_surt_query() {
        // This is the desired behaviour according to the original tests: "a bug not yet fixed for compatibility concern"
        // https://github.com/internetarchive/surt/blob/master/tests/test_surt.py#L338
        assert_eq!(
            surt(
                "http://example.com/script?type=a+b+%26+c&grape=wine",
                None
            )
            .unwrap(),
            "com,example)/script?grape=wine&type=a+b+%26+c"
        );

        // Code currently outputs: "com,example)/script?+c&grape=wine&type=a+b+"
    }

    #[test]
    fn test_surt_nonascii() {
        assert_eq!(
            surt("http://example.com/app?item=Wroc%C5%82aw", None).unwrap(),
            "com,example)/app?item=wroc%c5%82aw"
        )
    }

    #[test]
    fn test_surt_ipaddress() {
        let mut reverse_ip_opts = options::SurtrOptions::default();
        reverse_ip_opts.set("reverse_ipaddr", false);
        reverse_ip_opts.set("surt", true);
        reverse_ip_opts.set("with_scheme", false);

        assert_eq!(
            surt(
                "http://www.example.com/",
                Some(reverse_ip_opts.clone()),
            )
            .unwrap(),
            "com,example)/"
        );
        assert_eq!(
            surt("http://192.168.1.254/info/", None).unwrap(),
            "254,1,168,192)/info"
        );
        assert_eq!(
            surt(
                "http://192.168.1.254/info/",
                Some(reverse_ip_opts.clone()),
            )
            .unwrap(),
            "192.168.1.254)/info"
        );

        reverse_ip_opts.set("reverse_ipaddr", true);
        assert_eq!(
            surt(
                "http://192.168.1.254/info/",
                Some(reverse_ip_opts.clone()),
            )
            .unwrap(),
            "254,1,168,192)/info"
        );
    }
}
