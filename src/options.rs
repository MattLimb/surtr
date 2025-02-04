#[derive(Debug, Clone, Copy)]
pub struct SurtrOptions {
    // URL Options
    pub surt: bool,
    pub public_suffix: bool,
    pub trailing_comma: bool,
    pub reverse_ipaddr: bool,
    pub with_scheme: bool,

    // Canonicalizer Options
    pub host_lowercase: bool,
    pub host_massage: bool,
    pub auth_strip_user: bool,
    pub auth_strip_pass: bool,
    pub port_strip_default: bool,
    pub path_strip_empty: bool,
    pub path_lowercase: bool,
    pub path_strip_session_id: bool,
    pub path_strip_trailing_slash_unless_empty: bool,
    pub query_lowercase: bool,
    pub query_strip_session_id: bool,
    pub query_strip_empty: bool,
    pub query_alpha_reorder: bool,
    pub hash_strip: bool,
}

impl Default for SurtrOptions {
    fn default() -> Self {
        Self {
            // URL Options
            surt: false,
            public_suffix: false,
            trailing_comma: false,
            reverse_ipaddr: true,
            with_scheme: true,

            // Canonicalizer Options
            host_lowercase: true,
            host_massage: true,
            auth_strip_user: true,
            auth_strip_pass: true,
            port_strip_default: true,
            path_strip_empty: false,
            path_lowercase: true,
            path_strip_session_id: true,
            path_strip_trailing_slash_unless_empty: true,
            query_lowercase: true,
            query_strip_session_id: true,
            query_strip_empty: true,
            query_alpha_reorder: true,
            hash_strip: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonicalizers::{default as def, ia};
    use crate::handy_url::HandyUrl;

    #[test]
    fn test_ia_ptions() {
        let mut options = SurtrOptions::default();
        let url = "http://example.com/foo?X=Y".to_string();

        assert_eq!(
            ia::canonicalize(HandyUrl::parse(&url).unwrap(), &options)
                .unwrap()
                .get_url(&options)
                .unwrap(),
            "http://example.com/foo?x=y"
        );

        options.query_lowercase = false;
        assert_eq!(
            ia::canonicalize(HandyUrl::parse(&url).unwrap(), &options)
                .unwrap()
                .get_url(&options)
                .unwrap(),
            "http://example.com/foo?X=Y"
        );
    }

    #[test]
    fn test_default_ptions() {
        let mut options = SurtrOptions::default();
        let url = "http://example.com/foo?X=Y".to_string();

        assert_eq!(
            def::canonicalize(HandyUrl::parse(&url).unwrap(), &options)
                .unwrap()
                .get_url(&options)
                .unwrap(),
            "http://example.com/foo?x=y"
        );

        options.query_lowercase = false;
        assert_eq!(
            def::canonicalize(HandyUrl::parse(&url).unwrap(), &options)
                .unwrap()
                .get_url(&options)
                .unwrap(),
            "http://example.com/foo?X=Y"
        );
    }
}
