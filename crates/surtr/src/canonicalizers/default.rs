use crate::canonicalizers::{google, ia};
use crate::handy_url::HandyUrl;
use crate::options::SurtrOptions;

pub fn canonicalize(url_input: HandyUrl, options: &SurtrOptions) -> Result<HandyUrl, String> {
    let mut url = url_input;

    url = google::canonicalize(url, options)?;
    url = ia::canonicalize(url, options)?;

    Ok(url)
}

#[cfg(test)]
mod tests {
    use crate::options::SurtrOptions;

    use super::*;

    #[test]
    fn test_default_ia_canonicalizer() {
        let def_options = SurtrOptions::default();

        // These tests are from DefaultIAURLCanonicalizerTest.java
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www.alexa.com/").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://alexa.com/"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://archive.org/index.html").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://archive.org/index.html"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://archive.org/index.html?").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://archive.org/index.html"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://archive.org/index.html?a=b").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://archive.org/index.html?a=b"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://archive.org/index.html?b=b&a=b").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://archive.org/index.html?a=b&b=b"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://archive.org/index.html?b=a&b=b&a=b").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://archive.org/index.html?a=b&b=a&b=b"
        );
        assert_eq!(
            canonicalize(
                HandyUrl::parse("http://www34.archive.org/index.html?b=a&b=b&a=b").unwrap(),
                &def_options
            )
            .unwrap()
            .get_url(&def_options)
            .unwrap(),
            "http://archive.org/index.html?a=b&b=a&b=b"
        );
    }
}
