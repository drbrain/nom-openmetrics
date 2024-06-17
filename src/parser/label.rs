use crate::{parser::string, Label};
use nom::{
    bytes::complete::{tag, take_while, take_while1},
    combinator::{map, recognize},
    error::{context, VerboseError},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair},
    IResult,
};

fn is_metric_label_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_metric_label_end(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub fn label(input: &str) -> IResult<&str, Label, VerboseError<&str>> {
    context(
        "label",
        map(
            separated_pair(metric_label, tag("="), label_value),
            |(name, value)| Label { name, value },
        ),
    )(input)
}

pub fn labels(input: &str) -> IResult<&str, Vec<Label>, VerboseError<&str>> {
    context(
        "labels",
        delimited(tag("{"), separated_list0(tag(","), label), tag("}")),
    )(input)
}

// FIX: Does not parse escaped characters \", \\, \n
fn label_value(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    string(input)
}

/// Matches a metric name `[a-zA-Z_][a-zA-Z0-9_]*`
fn metric_label(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    context(
        "metric label",
        recognize(preceded(
            take_while1(is_metric_label_start),
            take_while(is_metric_label_end),
        )),
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::parse;
    use nom::error::VerboseErrorKind;
    use rstest::rstest;

    #[rstest]
    #[case(r#"job="prometheus""#, Label::new("job", "prometheus"))]
    #[case(r#"job="☃""#, Label::new("job", "☃"))]
    fn label(#[case] input: &str, #[case] expected: Label<'_>) {
        use crate::test::parse;

        let (rest, label) = parse(super::label, input);

        assert_eq!(expected, label, "input: {input}");
        assert!(rest.is_empty());
    }

    #[rstest]
    #[case(r#"{job="prometheus",instance="scrape.example"}"#, vec![Label::new("job", "prometheus"), Label::new("instance", "scrape.example")])]
    fn labels(#[case] input: &str, #[case] expected: Vec<Label<'_>>) {
        let (rest, labels) = parse(super::labels, input);

        assert_eq!(expected, labels, "input: {input}");
        assert!(rest.is_empty());
    }

    #[test]
    fn metric_label_error() {
        let input = "0";

        let Err(nom::Err::Error(error)) = metric_label(input) else {
            unreachable!("input {input} must error");
        };

        let (error_input, VerboseErrorKind::Context(kind)) = error.errors.last().unwrap() else {
            unreachable!("error kind mismatch for {error:?}");
        };

        assert_eq!(&"0", error_input);
        assert_eq!(&"metric label", kind);
    }

    #[rstest]
    #[case("A0", "", "A0")]
    #[case("__name__", "", "__name__")]
    #[case("a0", "", "a0")]
    #[case("name_0_more", "", "name_0_more")]
    #[case("rule:name", ":name", "rule")]
    #[case("up", "", "up")]
    #[case("up{", "{", "up")]
    fn metric_label_ok(
        #[case] input: &str,
        #[case] expected_rest: &str,
        #[case] expected_parsed: &str,
    ) {
        let (rest, parsed) = parse(super::metric_label, input);

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
