use std::collections::HashMap;


/// SurtrOptions contains all the options possible for use with Surtr.
/// 
/// The contained options are stored in a HashMap for more flexiable addition or 
/// modification of items. Every item is a toggle switch, taking in a boolean value.
/// 
/// The options are:
///
/// |              Surtr Option              | Stage Affected    | Default | Description                                                                                               |
/// | :------------------------------------- | :---------------  | :------ | :-------------------------------------------------------------------------------------------------------- |
/// |             public_suffix              | SURT Compilation  |  false  | Discard any subdomains in the URL.                                                                        |
/// |                  surt                  | SURT Compilation  |  true   | Return the URL as a SURT. Returns as a valid URL if false.                                                |
/// |             reverse_ipaddr             | SURT Compilation  |  true   | Reverses the IP address in the SURT. Only valid when surt=true                                            |
/// |              with_scheme               | SURT Compilation  |  true   | Includes the scheme (http, dns, ftp) as part of the SURT.                                                 |
/// |             trailing_comma             | SURT Compilation  |  false  | Append a comma after the host portion of the URL.                                                         |
/// |             host_lowercase             | Canonicalization  |  true   | Convert the host portion of the URL into lowercase.                                                       |
/// |              host_massage              | Canonicalization  |  true   | Remove superflous www. from the host.                                                                     |
/// |              auth_exclude              | URL Parsing       |  true   | Ignore the BasicAuth portion of the URL during parsing. This maintins IA behaviour.                        |
/// |            auth_strip_user             | Canonicalization  |  true   | Remove all basic auth from the URL.                                                                       |
/// |            auth_strip_pass             | Canonicalization  |  true   | Remove only the password from basic auth.                                                                 |
/// |           port_strip_default           | Canonicalization  |  true   | Remove the port number if it is the default for the given supported protocol. (http, https are supported) |
/// |            path_strip_empty            | Canonicalization  |  false  | Remove the trailing slash if there is no other path options.                                              |
/// |             path_lowercase             | Canonicalization  |  true   | Convert the path to lowercase.                                                                            |
/// |         path_strip_session_id          | Canonicalization  |  true   | Strip common session ID formats from the path.                                                            |
/// | path_srtip_trailing_slash_unless_empty | Canonicalization  |  true   | Strip the trailing slash from the URL path, unless there is no other path elements.                       |
/// |         query_strip_session_id         | Canonicalization  |  true   | Strip the common session IDs from the query parameters.                                                   |
/// |            query_lowercase             | Canonicalization  |  true   | Convert all elements of the query parameters to lowercase.                                                |
/// |          query_alpha_reorder           | Canonicalization  |  true   | Reorder the query parameters into alphabetical order.                                                     |
/// |           query_strip_empty            | Canonicalization  |  true   | Remove the query parameter ? if there aren't any query parameters.                                        |
/// 
/// # Examples
/// 
/// ```rust
/// use surtr::SurtrOptions;
/// 
/// let mut options = SurtrOptions::default();
/// 
/// options.set("public_suffix", true);
/// options.set("with_scheme", true);
/// options.set("path_lowercase", false);
/// 
/// assert_eq!(options.get("public_suffix"), Some(true));
/// assert_eq!(options.get("with_scheme"), Some(true));
/// assert_eq!(options.get("path_lowercase"), Some(false));
/// 
/// assert_eq!(options.get_or("public_suffix", false), true);
/// assert_eq!(options.get_or("query_strip_session_id", false), false);
/// ```
#[derive(Debug, Clone, Default)]
pub struct SurtrOptions {
    options: HashMap<String, bool>,
}

impl SurtrOptions {

    /// Get the value of an option.
    /// 
    /// Will return None if the given option is not present within the HashMap.
    pub fn get(&self, option: &str) -> Option<bool> {
        self.options.get(option).copied()
    }

    /// Get the value of an option, or a default.
    /// 
    /// This option provides a convienient way to retrieve an option from the HashMap.
    /// This is the primary interface for retrieving Options.
    pub fn get_or(&self, option: &str, or: bool) -> bool {
        if let Some(opt) = self.get(option) {
            return opt;
        }

        or
    }

    /// Set the value of an option.
    /// 
    /// This option provides a convienient way to set an option within the HashMap.
    /// This is the primary interface for setting Options.
    pub fn set(&mut self, option: &str, value: bool) {
        self.options.insert(option.to_string(), value);
    }

}


impl IntoIterator for SurtrOptions {
    type Item = (String, bool);
    type IntoIter = std::collections::hash_map::IntoIter<String, bool>;
    
    /// Get a list of options within the HashMap, and their values.
    /// 
    /// This provides a convienient way to produce a list of configured options.
    /// 
    /// Note: This will NOT provide the default values for missing options.
    fn into_iter(self) -> Self::IntoIter {
        self.options.into_iter()
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
            ia::canonicalize(HandyUrl::parse(&url, &options).unwrap(), &options)
                .unwrap()
                .get_url(&options)
                .unwrap(),
            "http://example.com/foo?x=y"
        );

        options.set("query_lowercase", false);
        assert_eq!(
            ia::canonicalize(HandyUrl::parse(&url, &options).unwrap(), &options)
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
            def::canonicalize(HandyUrl::parse(&url, &options).unwrap(), &options)
                .unwrap()
                .get_url(&options)
                .unwrap(),
            "http://example.com/foo?x=y"
        );

        options.set("query_lowercase", false);
        assert_eq!(
            def::canonicalize(HandyUrl::parse(&url, &options).unwrap(), &options)
                .unwrap()
                .get_url(&options)
                .unwrap(),
            "http://example.com/foo?X=Y"
        );
    }

    #[test]
    fn test_into_iter() {
        let option: Vec<(String, bool)> = vec![
            ("public_suffix".to_string(), true),
            ("with_scheme".to_string(), true),
            ("path_lowercase".to_string(), false),
        ];

        let mut surtr_options = SurtrOptions::default();
        for (key, value) in option.clone() {
            surtr_options.set(&key, value);
        }

        for (key, value) in surtr_options {
            assert_eq!(option.contains(&(key, value)), true);
        }
    }
}
