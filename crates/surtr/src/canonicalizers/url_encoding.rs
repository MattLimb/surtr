use percent_encoding::{percent_decode_str, percent_encode, AsciiSet, NON_ALPHANUMERIC};

const FRAGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'!')
    .remove(b'"')
    .remove(b'$')
    .remove(b'&')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')')
    .remove(b'*')
    .remove(b'+')
    .remove(b',')
    .remove(b'-')
    .remove(b'.')
    .remove(b'/')
    .remove(b':')
    .remove(b';')
    .remove(b'<')
    .remove(b'=')
    .remove(b'>')
    .remove(b'?')
    .remove(b'@')
    .remove(b'[')
    .remove(b'\\')
    .remove(b']')
    .remove(b'^')
    .remove(b'_')
    .remove(b'`')
    .remove(b'{')
    .remove(b'|')
    .remove(b'}')
    .remove(b'~')
    // Add the space character
    .add(b' ');

pub fn minimal_escape(input: String) -> Result<String, String> {
    Ok(escape_once(unescape_repeatedly(input)?))
}

pub fn escape_once(input: String) -> String {
    percent_encode(&input.into_bytes(), FRAGMENT).to_string()
}

pub fn unescape_repeatedly(input: String) -> Result<String, String> {
    let mut working_input = input.clone();

    loop {
        let un = match percent_decode_str(&working_input).decode_utf8() {
            Ok(t) => t.to_string(),
            Err(e) => return Err(format!("{:?}", e)),
        };

        if un == working_input {
            return Ok(working_input);
        }

        working_input = un;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_repeatedly() {
        // The tests are copied from GoogleURLCanonicalizerTest.java
        assert_eq!(
            unescape_repeatedly("%!A%21%21%25".to_string()).unwrap(),
            "%!A!!%"
        );
        assert_eq!(unescape_repeatedly("%".to_string()).unwrap(), "%");
        assert_eq!(unescape_repeatedly("%2".to_string()).unwrap(), "%2");
        assert_eq!(unescape_repeatedly("%25".to_string()).unwrap(), "%");
        assert_eq!(unescape_repeatedly("%25%".to_string()).unwrap(), "%%");
        assert_eq!(unescape_repeatedly("%2525".to_string()).unwrap(), "%");
        assert_eq!(unescape_repeatedly("%252525".to_string()).unwrap(), "%");
        assert_eq!(unescape_repeatedly("%25%32%35".to_string()).unwrap(), "%");
    }
}
