use nom::{
    bytes::complete::{take_while, take_while1},
    combinator::recognize,
    error::context,
    sequence::preceded,
    IResult, Parser,
};
use nom_language::error::VerboseError;

fn is_metric_name_initial_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == ':'
}

pub(crate) fn is_metric_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == ':'
}

/// Parse a metric name: `[a-zA-Z_:][a-zA-Z0-9_:]*`
pub(crate) fn metric_name(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    context(
        "metric name",
        recognize(preceded(
            take_while1(is_metric_name_initial_char),
            take_while(is_metric_name_char),
        )),
    )
    .parse(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::parse;
    use nom_language::error::VerboseErrorKind;
    use rstest::rstest;

    #[test]
    fn metric_name_error() {
        let input = "0";

        let Err(nom::Err::Error(error)) = metric_name(input) else {
            unreachable!("input {input} must error");
        };

        let (error_input, VerboseErrorKind::Context(kind)) = error.errors.last().unwrap() else {
            unreachable!("error kind mismatch for {error:?}");
        };

        assert_eq!(&"0", error_input);
        assert_eq!(&"metric name", kind);
    }

    #[rstest]
    #[case("A0", "", "A0")]
    #[case("__name__", "", "__name__")]
    #[case("a0", "", "a0")]
    #[case("name_0_more", "", "name_0_more")]
    #[case("rule:name", "", "rule:name")]
    #[case("up", "", "up")]
    #[case("up{", "{", "up")]
    fn metric_name_ok(
        #[case] input: &str,
        #[case] expected_rest: &str,
        #[case] expected_parsed: &str,
    ) {
        let (rest, parsed) = parse(super::metric_name, input);

        assert_eq!(
            expected_parsed, parsed,
            "parsed mismatch, expected {expected_parsed} got {parsed}"
        );
        assert_eq!(
            expected_rest, rest,
            "rest mismatch, expected {expected_rest} got {rest}"
        );
    }
}
