use handy_url::HandyUrl;
use options::SurtrOptions;

pub mod canonicalizers;
pub mod handy_url;
pub mod options;
pub mod regex_transformer;
mod url_split;

type Canonicalizer =
    &'static dyn Fn(HandyUrl, &options::SurtrOptions) -> Result<handy_url::HandyUrl, String>;

// Canonicalizers MUST be passed through from the layer above.
pub fn surt(
    url: String,
    canonicalizer: Option<Canonicalizer>,
    options: Option<&options::SurtrOptions>,
) -> Result<String, String> {
    if url.starts_with("filedesc") {
        return Ok(url);
    }

    let working_canon: Canonicalizer;
    if let Some(canon) = canonicalizer {
        working_canon = canon;
    } else {
        working_canon = &canonicalizers::default::canonicalize;
    }

    let working_options: SurtrOptions;
    if let Some(opts) = options {
        working_options = *opts;
    } else {
        working_options = SurtrOptions {
            surt: true,
            with_scheme: false,
            ..SurtrOptions::default()
        };
    }

    let hurl = working_canon(handy_url::HandyUrl::parse(&url)?, &working_options)?;
    hurl.get_url(&working_options)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_canonicalizer(
        url_input: HandyUrl,
        _options: &SurtrOptions,
    ) -> Result<HandyUrl, String> {
        Ok(url_input)
    }

    #[test]
    fn test_surt() {
        // These tests are from WaybackURLKeyMakerTest.java
        // assert_eq!(surt("".to_string(), None, None).unwrap(), "-");
        assert_eq!(
            surt("filedesc:foo.arc.gz".to_string(), None, None).unwrap(),
            "filedesc:foo.arc.gz"
        );
        assert_eq!(
            surt("filedesc:/foo.arc.gz".to_string(), None, None).unwrap(),
            "filedesc:/foo.arc.gz"
        );
        assert_eq!(
            surt("filedesc://foo.arc.gz".to_string(), None, None).unwrap(),
            "filedesc://foo.arc.gz"
        );
        assert_eq!(
            surt("warcinfo:foo.warc.gz".to_string(), None, None).unwrap(),
            "warcinfo:foo.warc.gz"
        );
        assert_eq!(
            surt("dns:alexa.com".to_string(), None, None).unwrap(),
            "dns:alexa.com"
        );
        assert_eq!(
            surt("dns:archive.org".to_string(), None, None).unwrap(),
            "dns:archive.org"
        );

        assert_eq!(
            surt("http://www.archive.org/".to_string(), None, None).unwrap(),
            "org,archive)/"
        );
        assert_eq!(
            surt("http://archive.org/".to_string(), None, None).unwrap(),
            "org,archive)/"
        );
        assert_eq!(
            surt("http://archive.org/goo/".to_string(), None, None).unwrap(),
            "org,archive)/goo"
        );
        assert_eq!(
            surt("http://archive.org/goo/?".to_string(), None, None).unwrap(),
            "org,archive)/goo"
        );
        assert_eq!(
            surt("http://archive.org/goo/?b&a".to_string(), None, None).unwrap(),
            "org,archive)/goo?a&b"
        );
        assert_eq!(
            surt("http://archive.org/goo/?a=2&b&a=1".to_string(), None, None).unwrap(),
            "org,archive)/goo?a=1&a=2&b"
        );

        // TODO: Default options being overriden here - implement them.

        // trailing comma mode
        let trailing_comma = options::SurtrOptions {
            trailing_comma: true,

            surt: true,
            with_scheme: false,
            ..options::SurtrOptions::default()
        };
        assert_eq!(
            surt(
                "http://archive.org/goo/?a=2&b&a=1".to_string(),
                None,
                Some(&trailing_comma)
            )
            .unwrap(),
            "org,archive,)/goo?a=1&a=2&b"
        );
        assert_eq!(
            surt("dns:archive.org".to_string(), None, Some(&trailing_comma)).unwrap(),
            "dns:archive.org"
        );
        assert_eq!(
            surt(
                "warcinfo:foo.warc.gz".to_string(),
                None,
                Some(&trailing_comma)
            )
            .unwrap(),
            "warcinfo:foo.warc.gz"
        );

        // PHP session id:
        assert_eq!(surt("http://archive.org/index.php?PHPSESSID=0123456789abcdefghijklemopqrstuv&action=profile;u=4221".to_string(), None, None).unwrap(), "org,archive)/index.php?action=profile;u=4221");

        // WHOIS url:
        assert_eq!(
            surt(
                "whois://whois.isoc.org.il/shaveh.co.il".to_string(),
                None,
                None
            )
            .unwrap(),
            "il,org,isoc,whois)/shaveh.co.il"
        );

        // Yahoo web bug. See https://github.com/internetarchive/surt/issues/1
        assert_eq!(
            surt(
            "http://visit.webhosting.yahoo.com/visit.gif?&r=http%3A//web.archive.org/web/20090517140029/http%3A//anthonystewarthead.electric-chi.com/&b=Netscape%205.0%20%28Windows%3B%20en-US%29&s=1366x768&o=Win32&c=24&j=true&v=1.2".to_string(),
            None,
            None).unwrap(),
            "com,yahoo,webhosting,visit)/visit.gif?&b=netscape%205.0%20(windows;%20en-us)&c=24&j=true&o=win32&r=http://web.archive.org/web/20090517140029/http://anthonystewarthead.electric-chi.com/&s=1366x768&v=1.2");
        // Simple customization:
        assert_eq!(
            surt(
                "http://www.example.com/".to_string(),
                Some(&basic_canonicalizer),
                None
            )
            .unwrap(),
            "com,example,www)/"
        );
        assert_eq!(
            surt("mailto:foo@example.com".to_string(), None, None).unwrap(),
            "mailto:foo@example.com"
        );

        let def_options = options::SurtrOptions {
            surt: true,
            with_scheme: false,
            ..options::SurtrOptions::default()
        };

        assert_eq!(
            surt(
                "http://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    ..def_options
                })
            )
            .unwrap(),
            "http://(com,example)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    host_massage: true,
                    ..def_options
                })
            )
            .unwrap(),
            "http://(com,example)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: false,
                    ..def_options
                })
            )
            .unwrap(),
            "com,example)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    trailing_comma: true,
                    ..def_options
                })
            )
            .unwrap(),
            "http://(com,example,)/"
        );
        assert_eq!(
            surt(
                "https://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    trailing_comma: true,
                    ..def_options
                })
            )
            .unwrap(),
            "https://(com,example,)/"
        );
        assert_eq!(
            surt(
                "ftp://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: false,
                    trailing_comma: true,
                    ..def_options
                })
            )
            .unwrap(),
            "com,example,)/"
        );
        assert_eq!(
            surt(
                "ftp://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: false,
                    trailing_comma: false,
                    ..def_options
                })
            )
            .unwrap(),
            "com,example)/"
        );
        assert_eq!(
            surt(
                "ftp://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    trailing_comma: true,
                    ..def_options
                })
            )
            .unwrap(),
            "ftp://(com,example,)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    host_massage: false,
                    ..def_options
                })
            )
            .unwrap(),
            "http://(com,example,www)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: false,
                    host_massage: false,
                    ..def_options
                })
            )
            .unwrap(),
            "com,example,www)/"
        );
        assert_eq!(
            surt(
                "http://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    trailing_comma: true,
                    host_massage: false,
                    ..def_options
                })
            )
            .unwrap(),
            "http://(com,example,www,)/"
        );
        assert_eq!(
            surt(
                "https://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    trailing_comma: true,
                    host_massage: false,
                    ..def_options
                })
            )
            .unwrap(),
            "https://(com,example,www,)/"
        );
        assert_eq!(
            surt(
                "ftp://www.example.com/".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    trailing_comma: true,
                    host_massage: false,
                    ..def_options
                })
            )
            .unwrap(),
            "ftp://(com,example,www,)/"
        );

        assert_eq!(
            surt(
                "mailto:foo@example.com".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    ..def_options
                })
            )
            .unwrap(),
            "mailto:foo@example.com"
        );
        assert_eq!(
            surt(
                "mailto:foo@example.com".to_string(),
                None,
                Some(&options::SurtrOptions {
                    trailing_comma: true,
                    ..def_options
                })
            )
            .unwrap(),
            "mailto:foo@example.com"
        );
        assert_eq!(
            surt(
                "mailto:foo@example.com".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    trailing_comma: true,
                    ..def_options
                })
            )
            .unwrap(),
            "mailto:foo@example.com"
        );
        assert_eq!(
            surt(
                "dns:archive.org".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    ..def_options
                })
            )
            .unwrap(),
            "dns:archive.org"
        );
        assert_eq!(
            surt(
                "dns:archive.org".to_string(),
                None,
                Some(&options::SurtrOptions {
                    trailing_comma: true,
                    ..def_options
                })
            )
            .unwrap(),
            "dns:archive.org"
        );
        assert_eq!(
            surt(
                "dns:archive.org".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    trailing_comma: true,
                    ..def_options
                })
            )
            .unwrap(),
            "dns:archive.org"
        );
        assert_eq!(
            surt(
                "whois://whois.isoc.org.il/shaveh.co.il".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    ..def_options
                })
            )
            .unwrap(),
            "whois://(il,org,isoc,whois)/shaveh.co.il"
        );
        assert_eq!(
            surt(
                "whois://whois.isoc.org.il/shaveh.co.il".to_string(),
                None,
                Some(&options::SurtrOptions {
                    trailing_comma: true,
                    ..def_options
                })
            )
            .unwrap(),
            "il,org,isoc,whois,)/shaveh.co.il"
        );
        assert_eq!(
            surt(
                "whois://whois.isoc.org.il/shaveh.co.il".to_string(),
                None,
                Some(&options::SurtrOptions {
                    trailing_comma: true,
                    with_scheme: true,
                    ..def_options
                })
            )
            .unwrap(),
            "whois://(il,org,isoc,whois,)/shaveh.co.il"
        );
        assert_eq!(
            surt(
                "warcinfo:foo.warc.gz".to_string(),
                None,
                Some(&options::SurtrOptions {
                    trailing_comma: true,
                    ..def_options
                })
            )
            .unwrap(),
            "warcinfo:foo.warc.gz"
        );
        assert_eq!(
            surt(
                "warcinfo:foo.warc.gz".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    ..def_options
                })
            )
            .unwrap(),
            "warcinfo:foo.warc.gz"
        );

        assert_eq!(
            surt(
                "warcinfo:foo.warc.gz".to_string(),
                None,
                Some(&options::SurtrOptions {
                    with_scheme: true,
                    trailing_comma: true,
                    ..def_options
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
                "http://example.com/script?type=a+b+%26+c&grape=wine".to_string(),
                None,
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
            surt(
                "http://example.com/app?item=Wroc%C5%82aw".to_string(),
                None,
                None
            )
            .unwrap(),
            "com,example)/app?item=wroc%c5%82aw"
        )
    }

    #[test]
    fn test_surt_ipaddress() {
        let mut reverse_ip_opts = options::SurtrOptions {
            reverse_ipaddr: false,

            surt: true,
            with_scheme: false,

            ..options::SurtrOptions::default()
        };

        assert_eq!(
            surt(
                "http://www.example.com/".to_string(),
                None,
                Some(&reverse_ip_opts),
            )
            .unwrap(),
            "com,example)/"
        );
        assert_eq!(
            surt("http://192.168.1.254/info/".to_string(), None, None,).unwrap(),
            "254,1,168,192)/info"
        );
        assert_eq!(
            surt(
                "http://192.168.1.254/info/".to_string(),
                None,
                Some(&reverse_ip_opts),
            )
            .unwrap(),
            "192.168.1.254)/info"
        );

        reverse_ip_opts.reverse_ipaddr = true;
        assert_eq!(
            surt(
                "http://192.168.1.254/info/".to_string(),
                None,
                Some(&reverse_ip_opts),
            )
            .unwrap(),
            "254,1,168,192)/info"
        );
    }
}
