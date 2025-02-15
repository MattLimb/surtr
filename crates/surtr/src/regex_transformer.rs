use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref RE_IP_ADDRESS: Regex = Regex::new(r#"(?:(?:\d{1,3}\.){3}\d{1,3})$"#).unwrap();
    static ref RE_PATH_SESSIONID: Vec<Regex> = vec![
        RegexBuilder::new(r#"^(.*/)(\((?:[a-z]\([0-9a-z]{24}\))+\)/)([^\?]+\.aspx.*)$"#)
            .case_insensitive(true)
            .build()
            .unwrap(),
        RegexBuilder::new(r#"^(.*/)(\([0-9a-z]{24}\)/)([^\?]+\.aspx.*)$"#)
            .case_insensitive(true)
            .build()
            .unwrap(),
    ];
    static ref RE_QUERY_SESSIONID: Vec<Regex> = vec![
        RegexBuilder::new(r#"^(.*)(?:jsessionid=[0-9a-zA-Z]{32})(?:&(.*))?$"#)
            .case_insensitive(true)
            .build()
            .unwrap(),
        RegexBuilder::new(r#"^(.*)(?:phpsessid=[0-9a-zA-Z]{32})(?:&(.*))?$"#)
            .case_insensitive(true)
            .build()
            .unwrap(),
        RegexBuilder::new(r#"^(.*)(?:sid=[0-9a-zA-Z]{32})(?:&(.*))?$"#)
            .case_insensitive(true)
            .build()
            .unwrap(),
        RegexBuilder::new(r#"^(.*)(?:ASPSESSIONID[a-zA-Z]{8}=[a-zA-Z]{24})(?:&(.*))?$"#)
            .case_insensitive(true)
            .build()
            .unwrap(),
        RegexBuilder::new(r#"^(.*)(?:cfid=[^&]+&cftoken=[^&]+)(?:&(.*))?$"#)
            .case_insensitive(true)
            .build()
            .unwrap(),
    ];
}

pub fn host_to_surt(host: String, reverse_ipaddr: bool) -> String {
    if !reverse_ipaddr && RE_IP_ADDRESS.is_match(&host) {
        return host;
    }

    let mut parts: Vec<&str> = host.split(".").collect();
    parts.reverse();
    return parts.join(",");
}

pub fn strip_path_session_id(path_input: String) -> String {
    let mut path = path_input;

    for pat in RE_PATH_SESSIONID.iter() {
        if let Some(captures) = pat.captures(&path) {
            let cap_1 = captures.get(1);
            let cap_3 = captures.get(3);

            if cap_1.is_some() && cap_3.is_some() {
                path = format!("{}{}", cap_1.unwrap().as_str(), cap_3.unwrap().as_str());
            }
        }
    }

    path
}

pub fn strip_query_session_id(query_input: String) -> String {
    let mut query = query_input;

    for pat in RE_QUERY_SESSIONID.iter() {
        if let Some(captures) = pat.captures(&query) {
            let cap_1 = captures.get(1);
            let cap_2 = captures.get(2);

            if let Some(c1) = cap_1 {
                if let Some(c2) = cap_2 {
                    query = format!("{}{}", c1.as_str(), c2.as_str())
                } else {
                    query = c1.as_str().to_string();
                }
            }
        }
    }

    query
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_path_session_id() {
        // These tests are from IAURLCanonicalizerTest.java
        // Check ASP_SESSIONID2:
        assert_eq!(
            strip_path_session_id("/(S(4hqa0555fwsecu455xqckv45))/mileg.aspx".to_string()),
            "/mileg.aspx"
        );

        // Check ASP_SESSIONID2 (again):
        assert_eq!(
            strip_path_session_id("/(4hqa0555fwsecu455xqckv45)/mileg.aspx".to_string()),
            "/mileg.aspx"
        );

        // Check ASP_SESSIONID3:
        assert_eq!(strip_path_session_id("/(a(4hqa0555fwsecu455xqckv45)S(4hqa0555fwsecu455xqckv45)f(4hqa0555fwsecu455xqckv45))/mileg.aspx?page=sessionschedules".to_string()), "/mileg.aspx?page=sessionschedules");

        // "@" in path:
        assert_eq!(
            strip_path_session_id("/photos/36050182@N05/".to_string()),
            "/photos/36050182@N05/"
        );
    }

    #[test]
    fn test_strip_query_session_id() {
        let str32id: String = "0123456789abcdefghijklemopqrstuv".to_string();

        assert_eq!(
            strip_query_session_id(format!("?jsessionid={}", str32id)),
            "?"
        );

        // Test that we don"t strip if not 32 chars only.
        assert_eq!(
            strip_query_session_id(format!("?jsessionid={}0", str32id)),
            "?jsessionid=0123456789abcdefghijklemopqrstuv0"
        );

        // Test what happens when followed by another key/value pair.
        assert_eq!(
            strip_query_session_id(format!("?jsessionid={}&x=y", str32id)),
            "?x=y"
        );

        // Test what happens when followed by another key/value pair and
        // prefixed by a key/value pair.
        assert_eq!(
            strip_query_session_id(format!("?one=two&jsessionid={}&x=y", str32id)),
            "?one=two&x=y"
        );

        // Test what happens when prefixed by a key/value pair.
        assert_eq!(
            strip_query_session_id(format!("?one=two&jsessionid={}", str32id)),
            "?one=two&"
        );

        // Test aspsession.
        assert_eq!(
            strip_query_session_id(
                "?aspsessionidABCDEFGH=ABCDEFGHIJKLMNOPQRSTUVWX&x=y".to_string()
            ),
            "?x=y"
        );

        // Test archive phpsession.
        assert_eq!(
            strip_query_session_id(format!("?phpsessid={}&x=y", str32id)),
            "?x=y"
        );

        // With prefix too.
        assert_eq!(
            strip_query_session_id(format!("?one=two&phpsessid={}&x=y", str32id)),
            "?one=two&x=y"
        );

        // With only prefix
        assert_eq!(
            strip_query_session_id(format!("?one=two&phpsessid={}", str32id)),
            "?one=two&"
        );

        // Test sid.
        assert_eq!(
            strip_query_session_id("?sid=9682993c8daa2c5497996114facdc805&x=y".to_string()),
            "?x=y"
        );

        // Igor test.
        assert_eq!(
            strip_query_session_id(format!(
                "?sid=9682993c8daa2c5497996114facdc805&jsessionid={}",
                str32id
            )),
            "?"
        );

        assert_eq!(
            strip_query_session_id(
                "?CFID=1169580&CFTOKEN=48630702&dtstamp=22%2F08%2F2006%7C06%3A58%3A11".to_string()
            ),
            "?dtstamp=22%2F08%2F2006%7C06%3A58%3A11"
        );
        assert_eq!(
            strip_query_session_id(
                "?CFID=12412453&CFTOKEN=15501799&dt=19_08_2006_22_39_28".to_string()
            ),
            "?dt=19_08_2006_22_39_28"
        );
        assert_eq!(strip_query_session_id("?CFID=14475712&CFTOKEN=2D89F5AF-3048-2957-DA4EE4B6B13661AB&r=468710288378&m=forgotten".to_string()), "?r=468710288378&m=forgotten");
        assert_eq!(
            strip_query_session_id(
                "?CFID=16603925&CFTOKEN=2AE13EEE-3048-85B0-56CEDAAB0ACA44B8".to_string()
            ),
            "?"
        );
        assert_eq!(
            strip_query_session_id(
                "?CFID=4308017&CFTOKEN=63914124&requestID=200608200458360%2E39414378".to_string()
            ),
            "?requestID=200608200458360%2E39414378"
        );
    }

    #[test]
    fn test_host_to_surt() {
        assert_eq!(
            host_to_surt("www.archive.org".to_string(), true),
            "org,archive,www"
        );
        assert_eq!(
            host_to_surt("www.archive.org".to_string(), false),
            "org,archive,www"
        );

        assert_eq!(host_to_surt("123.123.net".to_string(), true), "net,123,123");
        assert_eq!(
            host_to_surt("123.123.net".to_string(), false),
            "net,123,123"
        );

        assert_eq!(
            host_to_surt("100.100.100.100.org".to_string(), true),
            "org,100,100,100,100"
        );
        assert_eq!(
            host_to_surt("100.100.100.100.org".to_string(), false),
            "org,100,100,100,100"
        );

        assert_eq!(
            host_to_surt("123.45.167.89".to_string(), true),
            "89,167,45,123"
        );
        assert_eq!(
            host_to_surt("123.45.167.89".to_string(), false),
            "123.45.167.89"
        );

        assert_eq!(
            host_to_surt("10.162.1024.3".to_string(), true),
            "3,1024,162,10"
        );
        assert_eq!(
            host_to_surt("10.162.1024.3".to_string(), false),
            "3,1024,162,10"
        );

        assert_eq!(
            host_to_surt("990.991.992.993".to_string(), true),
            "993,992,991,990"
        );
        assert_eq!(
            host_to_surt("990.991.992.993".to_string(), false),
            "990.991.992.993"
        );
    }
}
