use percent_encoding::{percent_decode_str, percent_encode, AsciiSet, NON_ALPHANUMERIC};

use crate::error::SurtrError;

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

// Percent Decode the input string, then pass through a single pass of percent encoding.
//
// # Arguments
// 
// * `input` - The input string to be escaped.
//
// # Returns
// 
// A Result containing the escaped string, or an error if the input string is not UTF-8 encoded.
pub fn minimal_escape(input: String) -> Result<String, SurtrError> {
    Ok(escape_once(unescape_repeatedly(input)?))
}

// Escape the input string once using Percent Encoding.
//
// # Arguments
// 
// * `input` - The input string to be escaped.
//
// # Returns
// 
// A Result containing the escaped string.
pub fn escape_once(input: String) -> String {
    percent_encode(&input.into_bytes(), FRAGMENT).to_string()
}

// Decode the input String until no percent encoded substrings remain.
//
// # Arguments
// 
// * `input` - The input string to be unescaped.
//
// # Returns
// 
// A Result containing the unescaped string, or an error if the input string is not UTF-8 encoded.
pub fn unescape_repeatedly(input: String) -> Result<String, SurtrError> {
    let mut working_input = input.clone();

    loop {
        let un = match percent_decode_str(&working_input).decode_utf8() {
            Ok(t) => t.to_string(),
            Err(e) => {
                return Err(SurtrError::CanonicalizerError(format!(
                    "provided string is not UTF-8 encoded {}",
                    e
                )))
            }
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
