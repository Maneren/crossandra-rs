use lazy_static::lazy_static;

const STRING_BASE: &str = r".*?(?<!\\)(\\\\)*?";
const INT_BASE: &str = r"[0-9](?:[0-9_]*[0-9])?";
const FLOAT_BASE: &str = r"[0-9](?:[0-9_]*[0-9])?(?:[eE][+\-]?[0-9](?:[0-9_]*[0-9])?)|(?:[0-9](?:[0-9_]*[0-9])?\.[0-9]*|\.[0-9]+)(?:[eE][+\-]?[0-9](?:[0-9_]*[0-9])?)?";

lazy_static! {
    pub static ref CHAR: (String, String) = ("char".into(), r"'(?:\\'|[^'])'".into());
    pub static ref SINGLE_QUOTED_STRING: (String, String) =
        ("single_quoted_string".into(), format!("'{STRING_BASE}'"));
    pub static ref DOUBLE_QUOTED_STRING: (String, String) =
        ("double_quoted_string".into(), format!("\"{STRING_BASE}\""));
    pub static ref LETTER: (String, String) = ("letter".into(), r"[A-Za-z]".into());
    pub static ref WORD: (String, String) = ("word".into(), r"[A-Za-z]+".into());
    pub static ref C_NAME: (String, String) = ("c_name".into(), r"[_A-Za-z][_A-Za-z\d]*".into());
    pub static ref NEWLINE: (String, String) = ("newline".into(), r"\r?\n".into());
    pub static ref DIGIT: (String, String) = ("digit".into(), r"[0-9]".into());
    pub static ref HEXDIGIT: (String, String) = ("hexdigit".into(), r"[0-9A-Fa-f]".into());
    pub static ref UNSIGNED_INT: (String, String) = ("unsigned_int".into(), INT_BASE.into());
    pub static ref SIGNED_INT: (String, String) =
        ("signed_int".into(), format!(r"[+\-]{INT_BASE}"));
    pub static ref DECIMAL: (String, String) = (
        "decimal".into(),
        format!(r"{INT_BASE}\.(?:[0-9]+)?|\.[0-9]+")
    );
    pub static ref UNSIGNED_FLOAT: (String, String) = ("unsigned_float".into(), FLOAT_BASE.into());
    pub static ref SIGNED_FLOAT: (String, String) =
        ("signed_float".into(), format!(r"[+\-](?:{FLOAT_BASE})"));
    pub static ref STRING: (String, String) = (
        "string".into(),
        format!("\"{STRING_BASE}\"|'{STRING_BASE}'")
    );
    pub static ref UNSIGNED_NUMBER: (String, String) =
        ("unsigned_number".into(), format!("{FLOAT_BASE}|{INT_BASE}"));
    pub static ref SIGNED_NUMBER: (String, String) = (
        "signed_number".into(),
        format!(r"[+\-](?:(?:{FLOAT_BASE})|{INT_BASE})")
    );
    pub static ref INT: (String, String) = ("int".into(), format!(r"[+\-]?{INT_BASE}"));
    pub static ref FLOAT: (String, String) = ("float".into(), format!(r"[+\-]?(?:{FLOAT_BASE})"));
    pub static ref NUMBER: (String, String) = (
        "number".into(),
        format!(r"[+\-]?(?:(?:{FLOAT_BASE})|{INT_BASE})")
    );
}

#[cfg(test)]
mod tests {
    use crate::{common, error::Error, Token, Tokenizer};

    fn prepare_tokenizer<'a>(pattern: (String, String)) -> Tokenizer<'a> {
        Tokenizer::default()
            .with_convert_crlf(false)
            .with_patterns(vec![pattern])
            .expect("the pattern should be valid")
    }

    fn test_patterns(tokenizer: &Tokenizer<'_>, tests: Vec<(&str, Result<Vec<&str>, char>)>) {
        for (inp, out) in tests {
            match (tokenizer.tokenize(inp), out) {
                (Ok(tokens), Ok(expected_values)) => {
                    let values = tokens
                        .iter()
                        .map(|Token { name: _name, value }| value.clone())
                        .collect::<Vec<String>>();
                    assert_eq!(values, expected_values);
                }
                (Err(Error::BadToken(bad_token)), Err(expected_bad_token)) => {
                    assert_eq!(bad_token, expected_bad_token);
                }
                (res, exp) => {
                    panic!("Mismatched result for input {inp:?}: got {res:?}, expected {exp:?}")
                }
            }
        }
    }

    #[test]
    fn single_quoted_string() {
        test_patterns(
            &prepare_tokenizer(common::SINGLE_QUOTED_STRING.clone()),
            vec![
                ("'test'", Ok(vec!["'test'"])),
                ("'''", Err('\'')),
                ("test", Err('t')),
                ("'test", Err('\'')),
                ("\\'test'", Err('\\')),
                ("'\\'test'", Ok(vec!["'\\'test'"])),
                ("'test\\'", Err('\'')),
                ("''", Ok(vec!["''"])),
            ],
        );
    }

    #[test]
    fn double_quoted_string() {
        test_patterns(
            &prepare_tokenizer(common::DOUBLE_QUOTED_STRING.clone()),
            vec![
                ("\"test\"", Ok(vec!["\"test\""])),
                ("\"\"\"", Err('"')),
                ("test", Err('t')),
                ("\"test", Err('"')),
                ("\\\"test\"", Err('\\')),
                (r#""\"test""#, Ok(vec![r#""\"test""#])),
                ("\"test\\\"", Err('"')),
                ("\"\"", Ok(vec!["\"\""])),
            ],
        );
    }

    #[test]
    fn string() {
        test_patterns(
            &prepare_tokenizer(common::STRING.clone()),
            vec![("'test'\"test\"", Ok(vec!["'test'", "\"test\""]))],
        );
    }

    #[test]
    fn char() {
        test_patterns(
            &prepare_tokenizer(common::CHAR.clone()),
            vec![
                ("'t'", Ok(vec!["'t'"])),
                ("'''", Err('\'')),
                ("'\\''", Ok(vec!["'\\''"])),
                ("t", Err('t')),
                ("t'", Err('t')),
                ("'t", Err('\'')),
                ("\\'t'", Err('\\')),
                ("'t\\'", Err('\'')),
                ("'tt'", Err('\'')),
                ("''", Err('\'')),
            ],
        );
    }

    #[test]
    fn letter() {
        test_patterns(
            &prepare_tokenizer(common::LETTER.clone()),
            vec![
                ("AZaz", Ok(vec!["A", "Z", "a", "z"])),
                ("Wow!", Err('!')),
                ("!", Err('!')),
                ("@", Err('@')),
                ("|", Err('|')),
            ],
        );
    }

    #[test]
    fn word() {
        test_patterns(
            &prepare_tokenizer(common::WORD.clone()),
            vec![
                ("A", Ok(vec!["A"])),
                ("word", Ok(vec!["word"])),
                (" word", Err(' ')),
            ],
        );
    }

    #[test]
    fn digit() {
        test_patterns(
            &prepare_tokenizer(common::DIGIT.clone()),
            vec![
                (
                    "0123456789",
                    Ok(vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]),
                ),
                ("٥", Err('٥')),
                ("/", Err('/')),
                (":", Err(':')),
            ],
        );
    }

    #[test]
    fn unsigned_int() {
        test_patterns(
            &prepare_tokenizer(common::UNSIGNED_INT.clone()),
            vec![
                ("21", Ok(vec!["21"])),
                ("037", Ok(vec!["037"])),
                ("1_000_000", Ok(vec!["1_000_000"])),
                ("1__0", Ok(vec!["1__0"])),
            ],
        );
    }

    #[test]
    fn signed_int() {
        test_patterns(
            &prepare_tokenizer(common::SIGNED_INT.clone()),
            vec![
                ("+21", Ok(vec!["+21"])),
                ("-37", Ok(vec!["-37"])),
                ("-142+315", Ok(vec!["-142", "+315"])),
                ("13", Err('1')),
            ],
        );
    }

    #[test]
    fn decimal() {
        test_patterns(
            &prepare_tokenizer(common::DECIMAL.clone()),
            vec![
                ("3.14", Ok(vec!["3.14"])),
                ("3.0", Ok(vec!["3.0"])),
                ("21.37", Ok(vec!["21.37"])),
                ("0.92", Ok(vec!["0.92"])),
                ("0000.92", Ok(vec!["0000.92"])),
                (".92", Ok(vec![".92"])),
                ("3.", Ok(vec!["3."])),
                ("3..3", Ok(vec!["3.", ".3"])),
                ("3..", Err('.')),
                ("3", Err('3')),
                (".", Err('.')),
            ],
        );
    }

    #[test]
    fn hexdigit() {
        test_patterns(
            &prepare_tokenizer(common::HEXDIGIT.clone()),
            vec![
                ("3Da", Ok(vec!["3", "D", "a"])),
                ("0x", Err('x')),
                ("g", Err('g')),
            ],
        );
    }

    #[test]
    fn c_name() {
        test_patterns(
            &prepare_tokenizer(common::C_NAME.clone()),
            vec![
                ("W", Ok(vec!["W"])),
                ("_", Ok(vec!["_"])),
                ("word", Ok(vec!["word"])),
                ("two_words", Ok(vec!["two_words"])),
                ("_word", Ok(vec!["_word"])),
                ("_two_words", Ok(vec!["_two_words"])),
                ("0word", Err('0')),
                ("word0", Ok(vec!["word0"])),
                ("_0word", Ok(vec!["_0word"])),
                ("_word0", Ok(vec!["_word0"])),
                ("0", Err('0')),
                ("2322", Err('2')),
                ("wórd", Err('ó')),
            ],
        );
    }

    #[test]
    fn newline() {
        test_patterns(
            &prepare_tokenizer(common::NEWLINE.clone()),
            vec![
                ("\n", Ok(vec!["\n"])),
                ("\r\n", Ok(vec!["\r\n"])),
                ("\r", Err('\r')),
                ("\\n", Err('\\')),
            ],
        );
    }

    #[test]
    fn unsigned_float() {
        test_patterns(
            &prepare_tokenizer(common::UNSIGNED_FLOAT.clone()),
            vec![
                ("13", Err('1')),
                ("13.", Ok(vec!["13."])),
                (".13", Ok(vec![".13"])),
                ("1e3", Ok(vec!["1e3"])),
                ("1e+3", Ok(vec!["1e+3"])),
                ("1e+3.5", Ok(vec!["1e+3", ".5"])),
                ("1e-3", Ok(vec!["1e-3"])),
                ("1E3", Ok(vec!["1E3"])),
                (".0e3", Ok(vec![".0e3"])),
                ("1.e5", Ok(vec!["1.e5"])),
                ("1.0e3", Ok(vec!["1.0e3"])),
                ("1.0e+3", Ok(vec!["1.0e+3"])),
                ("1.0e-3", Ok(vec!["1.0e-3"])),
                ("1.0e", Err('e')),
            ],
        );
    }

    #[test]
    fn signed_float() {
        test_patterns(
            &prepare_tokenizer(common::SIGNED_FLOAT.clone()),
            vec![
                ("+1", Err('+')),
                ("+1e3", Ok(vec!["+1e3"])),
                ("-1e+3", Ok(vec!["-1e+3"])),
                ("+1e+3.5", Err('.')),
                ("+1e+3+.5", Ok(vec!["+1e+3", "+.5"])),
                ("-1e-3", Ok(vec!["-1e-3"])),
                ("+1E3", Ok(vec!["+1E3"])),
                ("1E3", Err('1')),
                ("-1.0e3", Ok(vec!["-1.0e3"])),
                ("+1.0e+3", Ok(vec!["+1.0e+3"])),
                ("-1.0e-3", Ok(vec!["-1.0e-3"])),
                ("+1.0e", Err('e')),
            ],
        );
    }

    #[test]
    fn unsigned_number() {
        test_patterns(
            &prepare_tokenizer(common::UNSIGNED_NUMBER.clone()),
            vec![("1", Ok(vec!["1"])), ("1.0", Ok(vec!["1.0"]))],
        );
    }

    #[test]
    fn signed_number() {
        test_patterns(
            &prepare_tokenizer(common::SIGNED_NUMBER.clone()),
            vec![
                ("+1", Ok(vec!["+1"])),
                ("-1.0", Ok(vec!["-1.0"])),
                ("1", Err('1')),
                ("1.0", Err('1')),
            ],
        );
    }

    #[test]
    fn int() {
        test_patterns(
            &prepare_tokenizer(common::INT.clone()),
            vec![("10+200-3000", Ok(vec!["10", "+200", "-3000"]))],
        );
    }

    #[test]
    fn float() {
        test_patterns(
            &prepare_tokenizer(common::FLOAT.clone()),
            vec![
                ("8.83-77641702.4", Ok(vec!["8.83", "-77641702.4"])),
                ("-497e4815.0+19.", Ok(vec!["-497e4815", ".0", "+19."])),
                ("-25.-7.6320036.8", Ok(vec!["-25.", "-7.6320036", ".8"])),
                ("11.9+8e55009.239", Ok(vec!["11.9", "+8e55009", ".239"])),
                (".7e.68732406+ee", Err('e')),
                ("5e8336+8.+717.52", Ok(vec!["5e8336", "+8.", "+717.52"])),
                ("5e8336++8.+717.52", Err('+')),
            ],
        );
    }

    #[test]
    fn number() {
        test_patterns(
            &prepare_tokenizer(common::NUMBER.clone()),
            vec![
                ("45692.+3795+74-e35.+", Err('-')),
                ("70-.8-", Err('-')),
                ("-", Err('-')),
                (
                    "+491814+4.4677-3412.",
                    Ok(vec!["+491814", "+4.4677", "-3412."]),
                ),
                (".e2..1", Err('.')),
                ("484-3+798.", Ok(vec!["484", "-3", "+798."])),
                ("2e6121+15+04", Ok(vec!["2e6121", "+15", "+04"])),
                (".537e0-5.56e097e16", Err('e')),
                ("-40e66.84712889820", Ok(vec!["-40e66", ".84712889820"])),
                ("+683011.+8557+e.76", Err('+')),
                ("662+2.60.305179", Ok(vec!["662", "+2.60", ".305179"])),
                ("", Ok(vec![])),
                ("26286086801-8+.5", Ok(vec!["26286086801", "-8", "+.5"])),
                ("7179", Ok(vec!["7179"])),
            ],
        );
    }
}
