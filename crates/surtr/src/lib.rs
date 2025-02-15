pub mod error;
pub mod handy_url;
pub mod options;

mod canonicalizers;
mod regex_transformer;
mod url_split;

pub type Canonicalizer<'a> = Box<
    dyn FnOnce(
            handy_url::HandyUrl,
            &options::SurtrOptions,
        ) -> Result<handy_url::HandyUrl, error::SaturError>
        + 'a,
>;

pub fn surt<'a>(
    url: Option<&str>,
    canonicalizer: Option<Canonicalizer>,
    options: Option<options::SurtrOptions>,
) -> Result<String, error::SaturError> {
    if url == Some("") || url.is_none() {
        return Ok("-".to_string());
    }

    let canon: Canonicalizer = match canonicalizer {
        Some(c) => c,
        None => Box::new(canonicalizers::default::canonicalize),
    };

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
    _surt(url.unwrap(), canon, &s_options)
}

fn _surt<'a>(
    url: &str,
    canonicalizer: Canonicalizer,
    options: &options::SurtrOptions,
) -> Result<String, error::SaturError> {
    // Hardcoded Workaround for filedesc
    if url.starts_with("filedesc") {
        return Ok(url.to_string());
    }

    // Parse URL
    let mut hurl = handy_url::HandyUrl::parse(url)?;

    // Canonicalize URL
    hurl = canonicalizer(hurl, options)?;

    // Build String URL
    hurl.get_url(options)
}

#[cfg(test)]
mod tests {
    use crate::error::SaturError;

    use super::*;

    fn basic_canonicalizer(
        url_input: handy_url::HandyUrl,
        _options: &options::SurtrOptions,
    ) -> Result<handy_url::HandyUrl, SaturError> {
        Ok(url_input)
    }

    #[test]
    fn test_surt() {
        // These tests are from WaybackURLKeyMakerTest.java
        // assert_eq!(surt("", None, None).unwrap(), "-");
        assert_eq!(
            surt(Some("filedesc:foo.arc.gz"), None, None).unwrap(),
            "filedesc:foo.arc.gz"
        );
        assert_eq!(
            surt(Some("filedesc:/foo.arc.gz"), None, None).unwrap(),
            "filedesc:/foo.arc.gz"
        );
        assert_eq!(
            surt(Some("filedesc://foo.arc.gz"), None, None).unwrap(),
            "filedesc://foo.arc.gz"
        );
        assert_eq!(
            surt(Some("warcinfo:foo.warc.gz"), None, None).unwrap(),
            "warcinfo:foo.warc.gz"
        );
        assert_eq!(
            surt(Some("dns:alexa.com"), None, None).unwrap(),
            "dns:alexa.com"
        );
        assert_eq!(
            surt(Some("dns:archive.org"), None, None).unwrap(),
            "dns:archive.org"
        );

        assert_eq!(
            surt(Some("http://www.archive.org/"), None, None).unwrap(),
            "org,archive)/"
        );
        assert_eq!(
            surt(Some("http://archive.org/"), None, None).unwrap(),
            "org,archive)/"
        );
        assert_eq!(
            surt(Some("http://archive.org/goo/"), None, None).unwrap(),
            "org,archive)/goo"
        );
        assert_eq!(
            surt(Some("http://archive.org/goo/?"), None, None).unwrap(),
            "org,archive)/goo"
        );
        assert_eq!(
            surt(Some("http://archive.org/goo/?b&a"), None, None).unwrap(),
            "org,archive)/goo?a&b"
        );
        assert_eq!(
            surt(Some("http://archive.org/goo/?a=2&b&a=1"), None, None).unwrap(),
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
                Some("http://archive.org/goo/?a=2&b&a=1"),
                None,
                Some(trailing_comma.clone())
            )
            .unwrap(),
            "org,archive,)/goo?a=1&a=2&b"
        );
        assert_eq!(
            surt(Some("dns:archive.org"), None, Some(trailing_comma.clone())).unwrap(),
            "dns:archive.org"
        );
        assert_eq!(
            surt(Some("warcinfo:foo.warc.gz"), None, Some(trailing_comma)).unwrap(),
            "warcinfo:foo.warc.gz"
        );

        // PHP session id:
        assert_eq!(surt(Some("http://archive.org/index.php?PHPSESSID=0123456789abcdefghijklemopqrstuv&action=profile;u=4221"), None, None).unwrap(), "org,archive)/index.php?action=profile;u=4221");

        // WHOIS url:
        assert_eq!(
            surt(Some("whois://whois.isoc.org.il/shaveh.co.il"), None, None).unwrap(),
            "il,org,isoc,whois)/shaveh.co.il"
        );

        // Yahoo web bug. See https://github.com/internetarchive/surt/issues/1
        assert_eq!(
            surt(
            Some("http://visit.webhosting.yahoo.com/visit.gif?&r=http%3A//web.archive.org/web/20090517140029/http%3A//anthonystewarthead.electric-chi.com/&b=Netscape%205.0%20%28Windows%3B%20en-US%29&s=1366x768&o=Win32&c=24&j=true&v=1.2"),
            None,
            None).unwrap(),
            "com,yahoo,webhosting,visit)/visit.gif?&b=netscape%205.0%20(windows;%20en-us)&c=24&j=true&o=win32&r=http://web.archive.org/web/20090517140029/http://anthonystewarthead.electric-chi.com/&s=1366x768&v=1.2");
        // Simple customization:
        assert_eq!(
            surt(
                Some("http://www.example.com/"),
                Some(Box::new(basic_canonicalizer)),
                None
            )
            .unwrap(),
            "com,example,www)/"
        );
        assert_eq!(
            surt(Some("mailto:foo@example.com"), None, None).unwrap(),
            "mailto:foo@example.com"
        );

        let mut def_options = options::SurtrOptions::default();
        def_options.set("surt", true);
        def_options.set("with_scheme", false);

        assert_eq!(
            surt(
                Some("http://www.example.com/"),
                None,
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
                Some("http://www.example.com/"),
                None,
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
                Some("http://www.example.com/"),
                None,
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
                Some("http://www.example.com/"),
                None,
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
                Some("https://www.example.com/"),
                None,
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
                Some("ftp://www.example.com/"),
                None,
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
                Some("ftp://www.example.com/"),
                None,
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
                Some("ftp://www.example.com/"),
                None,
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
                Some("http://www.example.com/"),
                None,
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
                Some("http://www.example.com/"),
                None,
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
                Some("http://www.example.com/"),
                None,
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
                Some("https://www.example.com/"),
                None,
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
                Some("ftp://www.example.com/"),
                None,
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
                Some("mailto:foo@example.com"),
                None,
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
                Some("mailto:foo@example.com"),
                None,
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
                Some("mailto:foo@example.com"),
                None,
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
                Some("dns:archive.org"),
                None,
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
                Some("dns:archive.org"),
                None,
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
                Some("dns:archive.org"),
                None,
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
                Some("whois://whois.isoc.org.il/shaveh.co.il"),
                None,
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
                Some("whois://whois.isoc.org.il/shaveh.co.il"),
                None,
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
                Some("whois://whois.isoc.org.il/shaveh.co.il"),
                None,
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
                Some("warcinfo:foo.warc.gz"),
                None,
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
                Some("warcinfo:foo.warc.gz"),
                None,
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
                Some("warcinfo:foo.warc.gz"),
                None,
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
                Some("http://example.com/script?type=a+b+%26+c&grape=wine"),
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
            surt(Some("http://example.com/app?item=Wroc%C5%82aw"), None, None).unwrap(),
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
                Some("http://www.example.com/"),
                None,
                Some(reverse_ip_opts.clone()),
            )
            .unwrap(),
            "com,example)/"
        );
        assert_eq!(
            surt(Some("http://192.168.1.254/info/"), None, None,).unwrap(),
            "254,1,168,192)/info"
        );
        assert_eq!(
            surt(
                Some("http://192.168.1.254/info/"),
                None,
                Some(reverse_ip_opts.clone()),
            )
            .unwrap(),
            "192.168.1.254)/info"
        );

        reverse_ip_opts.set("reverse_ipaddr", true);
        assert_eq!(
            surt(
                Some("http://192.168.1.254/info/"),
                None,
                Some(reverse_ip_opts.clone()),
            )
            .unwrap(),
            "254,1,168,192)/info"
        );
    }
}
