use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SurtrOptions {
    pub options: HashMap<String, bool>,
}

impl SurtrOptions {
    pub fn get(&self, option: &str) -> Option<bool> {
        match self.options.get(option) {
            Some(b) => Some(*b),
            None => None,
        }
    }

    pub fn get_or(&self, option: &str, or: bool) -> bool {
        if let Some(opt) = self.get(option) {
            return opt;
        }

        or
    }

    pub fn set(&mut self, option: &str, value: bool) {
        self.options.insert(option.to_string(), value);
    }

    pub fn as_items(&self) -> Vec<(String, bool)> {
        self.options.clone().into_iter().collect()
    }
}

impl Default for SurtrOptions {
    fn default() -> Self {
        Self {
            options: HashMap::new(),
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

        options.set("query_lowercase", false);
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

        options.set("query_lowercase", false);
        assert_eq!(
            def::canonicalize(HandyUrl::parse(&url).unwrap(), &options)
                .unwrap()
                .get_url(&options)
                .unwrap(),
            "http://example.com/foo?X=Y"
        );
    }
}
