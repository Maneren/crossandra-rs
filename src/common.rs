//! A collection of common patterns for use in tokenizers.
use const_format::formatcp;

const STRING_BASE: &str = r"(?:\\.|[^\\])*?";
const INT_BASE: &str = r"[0-9](?:[0-9_]*[0-9])?";
const FLOAT_BASE: &str = concat!(
    r"[0-9](?:[0-9_]*[0-9])?",                // integer part (required)
    r"(?:[eE][+\-]?[0-9](?:[0-9_]*[0-9])?)",  // exponent (required)
    r"|",                                     // or
    r"(?:",                                   // mantissa {
    r"[0-9](?:[0-9_]*[0-9])?",                //   integer part (required)
    r"\.(?:[0-9](?:[0-9_]*[0-9])?)?",         //   decimal part (optional)
    r"|",                                     //   or
    r"\.[0-9](?:[0-9_]*[0-9])?",              //   decimal part (required)
    r")",                                     // }
    r"(?:[eE][+\-]?[0-9](?:[0-9_]*[0-9])?)?"  // exponent (optional)
);

/// A single character enclosed in single quotes (e.g. `'h'`).
pub const CHAR: (&str, &str) = ("char", r"'(?:\\'|[^'])'");
/// A string enclosed in single quotes (e.g. `'nice fish'`).
pub const SINGLE_QUOTED_STRING: (&str, &str) =
    ("single_quoted_string", formatcp!("'{}'", STRING_BASE));
/// A string enclosed in double quotes (e.g. `"hello there"`).
pub const DOUBLE_QUOTED_STRING: (&str, &str) =
    ("double_quoted_string", formatcp!("\"{}\"", STRING_BASE));
/// An English letter (e.g. `m`). Case insensitive.
pub const LETTER: (&str, &str) = ("letter", r"[A-Za-z]");
/// An English word (e.g. `thread-safe`). Allows non-consecutive hyphens. Case insensitive.
pub const WORD: (&str, &str) = ("word", r"[A-Za-z]+(-[A-Za-z]+)*");
/// A C-like variable name (e.g. `crossandra_rocks`). Can consist of
/// English letters, digits, and underscores. Cannot start with a digit.
pub const C_NAME: (&str, &str) = ("c_name", r"[_A-Za-z][_A-Za-z\d]*");
/// A newline (either `\n` or `\r\n`).
pub const NEWLINE: (&str, &str) = ("newline", r"\r?\n");
/// A single digit (e.g. `7`).
pub const DIGIT: (&str, &str) = ("digit", r"[0-9]");
/// A single hexadecimal digit (e.g. `c`). Case insensitive.
pub const HEXDIGIT: (&str, &str) = ("hexdigit", r"[0-9A-Fa-f]");
/// An unsigned integer (e.g. `2_137`). Underscores can be used as separators.
pub const UNSIGNED_INT: (&str, &str) = ("unsigned_int", INT_BASE);
/// A signed integer (e.g. `-1`). Underscores can be used as separators.
pub const SIGNED_INT: (&str, &str) = ("signed_int", formatcp!("[+\\-]{}", INT_BASE));
/// A decimal value (e.g. `3.14`).
pub const DECIMAL: (&str, &str) = (
    "decimal",
    formatcp!("{}\\.(?:{})?|\\.{}", INT_BASE, INT_BASE, INT_BASE),
);
/// An unsigned floating point value (e.g. `1e3`).
pub const UNSIGNED_FLOAT: (&str, &str) = ("unsigned_float", FLOAT_BASE);
/// A signed floating point value (e.g. `+4.3`).
pub const SIGNED_FLOAT: (&str, &str) = ("signed_float", formatcp!("[+\\-](?:{})", FLOAT_BASE));
/// A string enclosed in either single or double quotes.
pub const STRING: (&str, &str) = (
    "string",
    formatcp!("\"{}\"|\\'{}\'", STRING_BASE, STRING_BASE),
);
/// An unsigned number (either an integer or a float).
pub const UNSIGNED_NUMBER: (&str, &str) =
    ("unsigned_number", formatcp!("{}|{}", FLOAT_BASE, INT_BASE));
/// A signed number (either an integer or a floating point value).
pub const SIGNED_NUMBER: (&str, &str) = (
    "signed_number",
    formatcp!("[+\\-](?:(?:{})|{})", FLOAT_BASE, INT_BASE),
);
/// Any integer value (optional sign).
pub const INT: (&str, &str) = ("int", formatcp!("[+\\-]?{}", INT_BASE));
/// Any floating point value (optional sign).
pub const FLOAT: (&str, &str) = ("float", formatcp!("[+\\-]?(?:{})", FLOAT_BASE));
/// Any number (optional sign).
pub const NUMBER: (&str, &str) = (
    "number",
    formatcp!("[+\\-]?(?:(?:{})|{})", FLOAT_BASE, INT_BASE),
);

#[cfg(test)]
mod tests {
    use crate::{common, error::Error, Tokenizer};

    fn prepare_tokenizer<'a>(pattern: (&'a str, &str)) -> Tokenizer<'a> {
        Tokenizer::default()
            .with_patterns(&[pattern])
            .expect("the pattern should be valid")
    }
    type TestOutcome<'a> = Result<Vec<&'a str>, (char, usize)>;

    fn test_patterns(tokenizer: &Tokenizer<'_>, tests: Vec<(&str, TestOutcome)>) {
        for (inp, out) in tests {
            match (tokenizer.tokenize(inp).find(Result::is_err), out) {
                (
                    Some(Err(Error::BadToken(err_value, err_position))),
                    Err((expected_err_value, expected_err_position)),
                ) => {
                    assert_eq!(err_value, expected_err_value);
                    assert_eq!(err_position, expected_err_position);
                }
                (None, Ok(expected_values)) => {
                    let values = tokenizer
                        .tokenize(inp)
                        .map(Result::unwrap)
                        .map(|token| token.value)
                        .collect::<Vec<_>>();
                    assert_eq!(values, expected_values);
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
            &prepare_tokenizer(common::SINGLE_QUOTED_STRING),
            vec![
                ("'test'", Ok(vec!["'test'"])),
                ("'''", Err(('\'', 2))),
                ("test", Err(('t', 0))),
                ("'test", Err(('\'', 0))),
                ("\\'test'", Err(('\\', 0))),
                ("'\\'test'", Ok(vec!["'\\'test'"])),
                ("'test\\'", Err(('\'', 0))),
                ("'test\\ntest'", Ok(vec!["'test\\ntest'"])),
                ("''", Ok(vec!["''"])),
            ],
        );
    }

    #[test]
    fn double_quoted_string() {
        test_patterns(
            &prepare_tokenizer(common::DOUBLE_QUOTED_STRING),
            vec![
                ("\"test\"", Ok(vec!["\"test\""])),
                ("\"\"\"", Err(('"', 2))),
                ("test", Err(('t', 0))),
                ("\"test", Err(('"', 0))),
                ("\\\"test\"", Err(('\\', 0))),
                (r#""\"test""#, Ok(vec![r#""\"test""#])),
                ("\"test\\\"", Err(('"', 0))),
                ("\"test\\ntest\"", Ok(vec!["\"test\\ntest\""])),
                ("\"\"", Ok(vec!["\"\""])),
            ],
        );
    }

    #[test]
    fn string() {
        test_patterns(
            &prepare_tokenizer(common::STRING),
            vec![("'test'\"test\"", Ok(vec!["'test'", "\"test\""]))],
        );
    }

    #[test]
    fn char() {
        test_patterns(
            &prepare_tokenizer(common::CHAR),
            vec![
                ("'t'", Ok(vec!["'t'"])),
                ("'''", Err(('\'', 0))),
                ("'\\''", Ok(vec!["'\\''"])),
                ("t", Err(('t', 0))),
                ("t'", Err(('t', 0))),
                ("'t", Err(('\'', 0))),
                ("\\'t'", Err(('\\', 0))),
                ("'t\\'", Err(('\'', 0))),
                ("'tt'", Err(('\'', 0))),
                ("''", Err(('\'', 0))),
            ],
        );
    }

    #[test]
    fn letter() {
        test_patterns(
            &prepare_tokenizer(common::LETTER),
            vec![
                ("AZaz", Ok(vec!["A", "Z", "a", "z"])),
                ("Wow!", Err(('!', 3))),
                ("!", Err(('!', 0))),
                ("@", Err(('@', 0))),
                ("|", Err(('|', 0))),
            ],
        );
    }

    #[test]
    fn word() {
        test_patterns(
            &prepare_tokenizer(common::WORD),
            vec![
                ("A", Ok(vec!["A"])),
                ("word", Ok(vec!["word"])),
                (" word", Err((' ', 0))),
                ("-", Err(('-', 0))),
                ("a-", Err(('-', 1))),
                ("-a", Err(('-', 0))),
                ("a-a", Ok(vec!["a-a"])),
                ("a--a", Err(('-', 1))),
                ("thread-safe", Ok(vec!["thread-safe"])),
                ("thread-", Err(('-', 6))),
                ("-jack-o", Err(('-', 0))),
                ("jack-o-lantern", Ok(vec!["jack-o-lantern"])),
            ],
        );
    }

    #[test]
    fn digit() {
        test_patterns(
            &prepare_tokenizer(common::DIGIT),
            vec![
                (
                    "0123456789",
                    Ok(vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]),
                ),
                ("٥", Err(('٥', 0))),
                ("/", Err(('/', 0))),
                (":", Err((':', 0))),
            ],
        );
    }

    #[test]
    fn unsigned_int() {
        test_patterns(
            &prepare_tokenizer(common::UNSIGNED_INT),
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
            &prepare_tokenizer(common::SIGNED_INT),
            vec![
                ("+21", Ok(vec!["+21"])),
                ("-37", Ok(vec!["-37"])),
                ("-142+315", Ok(vec!["-142", "+315"])),
                ("13", Err(('1', 0))),
            ],
        );
    }

    #[test]
    fn decimal() {
        test_patterns(
            &prepare_tokenizer(common::DECIMAL),
            vec![
                ("3.14", Ok(vec!["3.14"])),
                ("3.0", Ok(vec!["3.0"])),
                ("21.37", Ok(vec!["21.37"])),
                ("2_1.37", Ok(vec!["2_1.37"])),
                ("2_1.3_7", Ok(vec!["2_1.3_7"])),
                ("0.92", Ok(vec!["0.92"])),
                ("0000.92", Ok(vec!["0000.92"])),
                (".92", Ok(vec![".92"])),
                ("3.", Ok(vec!["3."])),
                ("3..3", Ok(vec!["3.", ".3"])),
                ("3..", Err(('.', 2))),
                ("3", Err(('3', 0))),
                (".", Err(('.', 0))),
            ],
        );
    }

    #[test]
    fn hexdigit() {
        test_patterns(
            &prepare_tokenizer(common::HEXDIGIT),
            vec![
                ("3Da", Ok(vec!["3", "D", "a"])),
                ("0x", Err(('x', 1))),
                ("g", Err(('g', 0))),
            ],
        );
    }

    #[test]
    fn c_name() {
        test_patterns(
            &prepare_tokenizer(common::C_NAME),
            vec![
                ("W", Ok(vec!["W"])),
                ("_", Ok(vec!["_"])),
                ("word", Ok(vec!["word"])),
                ("two_words", Ok(vec!["two_words"])),
                ("_word", Ok(vec!["_word"])),
                ("_two_words", Ok(vec!["_two_words"])),
                ("0word", Err(('0', 0))),
                ("word0", Ok(vec!["word0"])),
                ("_0word", Ok(vec!["_0word"])),
                ("_word0", Ok(vec!["_word0"])),
                ("0", Err(('0', 0))),
                ("2322", Err(('2', 0))),
                ("wórd", Err(('ó', 1))),
            ],
        );
    }

    #[test]
    fn newline() {
        test_patterns(
            &prepare_tokenizer(common::NEWLINE),
            vec![
                ("\n", Ok(vec!["\n"])),
                ("\r\n", Ok(vec!["\r\n"])),
                ("\r", Err(('\r', 0))),
                ("\\n", Err(('\\', 0))),
            ],
        );
    }

    #[test]
    fn unsigned_float() {
        test_patterns(
            &prepare_tokenizer(common::UNSIGNED_FLOAT),
            vec![
                ("13", Err(('1', 0))),
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
                ("1_0.5_0e-3_0", Ok(vec!["1_0.5_0e-3_0"])),
                ("1.0e", Err(('e', 3))),
            ],
        );
    }

    #[test]
    fn signed_float() {
        test_patterns(
            &prepare_tokenizer(common::SIGNED_FLOAT),
            vec![
                ("+1", Err(('+', 0))),
                ("+1e3", Ok(vec!["+1e3"])),
                ("-1e+3", Ok(vec!["-1e+3"])),
                ("+1e+3.5", Err(('.', 5))),
                ("+1e+3+.5", Ok(vec!["+1e+3", "+.5"])),
                ("-1e-3", Ok(vec!["-1e-3"])),
                ("+1E3", Ok(vec!["+1E3"])),
                ("1E3", Err(('1', 0))),
                ("-1.0e3", Ok(vec!["-1.0e3"])),
                ("+1.0e+3", Ok(vec!["+1.0e+3"])),
                ("-1.0e-3", Ok(vec!["-1.0e-3"])),
                ("-1_0.5_0e-3_0", Ok(vec!["-1_0.5_0e-3_0"])),
                ("+1.0e", Err(('e', 4))),
            ],
        );
    }

    #[test]
    fn unsigned_number() {
        test_patterns(
            &prepare_tokenizer(common::UNSIGNED_NUMBER),
            vec![
                ("1", Ok(vec!["1"])),
                ("1.0", Ok(vec!["1.0"])),
                ("1_0.0_0", Ok(vec!["1_0.0_0"])),
            ],
        );
    }

    #[test]
    fn signed_number() {
        test_patterns(
            &prepare_tokenizer(common::SIGNED_NUMBER),
            vec![
                ("+1", Ok(vec!["+1"])),
                ("+1_0", Ok(vec!["+1_0"])),
                ("-1.0", Ok(vec!["-1.0"])),
                ("1", Err(('1', 0))),
                ("1.0", Err(('1', 0))),
            ],
        );
    }

    #[test]
    fn int() {
        test_patterns(
            &prepare_tokenizer(common::INT),
            vec![(
                "10+200-3000-4_000",
                Ok(vec!["10", "+200", "-3000", "-4_000"]),
            )],
        );
    }

    #[test]
    fn float() {
        test_patterns(
            &prepare_tokenizer(common::FLOAT),
            vec![
                ("8_192.8_3-77641702.4", Ok(vec!["8_192.8_3", "-77641702.4"])),
                ("8.83-77641702.4", Ok(vec!["8.83", "-77641702.4"])),
                ("-497e4815.0+19.", Ok(vec!["-497e4815", ".0", "+19."])),
                ("-25.-7.6320036.8", Ok(vec!["-25.", "-7.6320036", ".8"])),
                ("11.9+8e55009.239", Ok(vec!["11.9", "+8e55009", ".239"])),
                (".7e.68732406+ee", Err(('e', 2))),
                ("5e8336+8.+717.52", Ok(vec!["5e8336", "+8.", "+717.52"])),
                ("5e8336++8.+717.52", Err(('+', 6))),
            ],
        );
    }

    #[test]
    fn number() {
        test_patterns(
            &prepare_tokenizer(common::NUMBER),
            vec![
                ("+8_192.8_3", Ok(vec!["+8_192.8_3"])),
                ("45692.+3795+74-e35.+", Err(('-', 14))),
                ("70-.8-", Err(('-', 5))),
                ("-", Err(('-', 0))),
                (
                    "+491814+4.4677-3412.",
                    Ok(vec!["+491814", "+4.4677", "-3412."]),
                ),
                (".e2..1", Err(('.', 0))),
                ("484-3+798.", Ok(vec!["484", "-3", "+798."])),
                ("2e6121+15+04", Ok(vec!["2e6121", "+15", "+04"])),
                (".537e0-5.56e097e16", Err(('e', 15))),
                ("-40e66.84712889820", Ok(vec!["-40e66", ".84712889820"])),
                ("+683011.+8557+e.76", Err(('+', 13))),
                ("662+2.60.305179", Ok(vec!["662", "+2.60", ".305179"])),
                ("", Ok(vec![])),
                ("26286086801-8+.5", Ok(vec!["26286086801", "-8", "+.5"])),
                ("7179", Ok(vec!["7179"])),
            ],
        );
    }
}
