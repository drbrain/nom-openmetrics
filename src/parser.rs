mod label;
mod number;

use crate::Metric;
pub use label::{label, labels};
use nom::{
    bytes::complete::{take_while, take_while1},
    combinator::{map, opt, recognize},
    error::{context, VerboseError},
    sequence::{preceded, tuple},
    IResult,
};
pub use number::number;

fn is_metric_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == ':'
}

fn is_metric_name_end(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == ':'
}

fn metric(input: &str) -> IResult<&str, Metric, VerboseError<&str>> {
    context(
        "metric",
        map(tuple((metric_name, opt(labels))), |(name, labels)| {
            if let Some(labels) = labels {
                Metric::with_labels(name, labels)
            } else {
                Metric::new(name)
            }
        }),
    )(input)
}

/// Matches a metric name `[a-zA-Z_:][a-zA-Z0-9_:]*`
fn metric_name(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    context(
        "metric name",
        recognize(preceded(
            take_while1(is_metric_name_start),
            take_while(is_metric_name_end),
        )),
    )(input)
}

/// Matches a metric value
fn metric_value(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    context("metric value", number)(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::parse;
    use nom::error::VerboseErrorKind;
    use rstest::rstest;

    #[rstest]
    #[case("up", Metric::new("up"))]
    #[case(r#"up{job="prometheus"}"#, Metric::new("up").add_label("job", "prometheus"))]
    #[case(r#"up{job="☃"}"#, Metric::new("up").add_label("job", "☃"))]
    fn metric(#[case] input: &str, #[case] expected: Metric<'_>) {
        use crate::test::parse;

        let (rest, metric) = parse(super::metric, input);

        assert_eq!(expected, metric, "input: {input} metric: {metric:?}");
        assert!(rest.is_empty());
    }

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
