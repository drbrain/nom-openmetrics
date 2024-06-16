mod label;
mod metric;

use label::Label;
use metric::Metric;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_till, take_while, take_while1},
    character::complete::{digit0, digit1},
    combinator::{map, map_res, opt, recognize},
    error::{context, VerboseError},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};

fn exponent(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    recognize(tuple((tag("e"), opt(sign), digit1)))(input)
}

fn infinity(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    context(
        "infinity",
        map(
            terminated(
                opt(sign),
                alt((tag_no_case("infinity"), tag_no_case("inf"))),
            ),
            |sign| {
                eprintln!("sign: {:?}", sign.unwrap_or(1.0));
                f64::INFINITY * sign.unwrap_or(1.0)
            },
        ),
    )(input)
}

fn is_metric_label_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_metric_label_end(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_metric_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == ':'
}

fn is_metric_name_end(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == ':'
}

fn label(input: &str) -> IResult<&str, Label, VerboseError<&str>> {
    context(
        "label",
        map(
            separated_pair(metric_label, tag("="), label_value),
            |(name, value)| Label { name, value },
        ),
    )(input)
}

fn labels(input: &str) -> IResult<&str, Vec<Label>, VerboseError<&str>> {
    context(
        "labels",
        delimited(tag("{"), separated_list0(tag(","), label), tag("}")),
    )(input)
}

// FIX: Does not parse escaped characters \", \\, \n
fn label_value(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(tag("\""), take_till(|c| c == '"'), tag("\""))(input)
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

fn nan(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    map(tag_no_case("nan"), |_| f64::NAN)(input)
}

fn number(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    context("number", alt((real_number, infinity, nan)))(input)
}

fn real_number(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    map(
        tuple((
            opt(sign),
            map_res(
                alt((
                    recognize(tuple((
                        digit1,
                        opt(terminated(tag("."), digit0)),
                        opt(exponent),
                    ))),
                    recognize(tuple((digit0, tag("."), digit1, opt(exponent)))),
                    digit1,
                )),
                |digits: &str| digits.parse::<f64>(),
            ),
        )),
        |(sign, digits)| sign.unwrap_or(1.0) * digits,
    )(input)
}

fn sign(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    map(alt((tag("-"), tag("+"))), |sign| match sign {
        "-" => -1.0,
        "+" => 1.0,
        _ => unreachable!(),
    })(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_float_eq::*;
    use nom::{
        error::{convert_error, VerboseErrorKind},
        Parser,
    };
    use rstest::rstest;

    fn parse<'a, O, F>(mut parser: F, input: &'a str) -> (&'a str, O)
    where
        F: Parser<&'a str, O, VerboseError<&'a str>>,
    {
        let result = parser.parse(input);

        match result {
            Ok(ok) => ok,
            Err(error) => match error {
                nom::Err::Incomplete(needed) => panic!("{needed:?}"),
                nom::Err::Error(error) | nom::Err::Failure(error) => {
                    panic!("{}", convert_error(input, error))
                }
            },
        }
    }

    #[rstest]
    #[case("e+0", "e+0")]
    #[case("e-0", "e-0")]
    #[case("e0", "e0")]
    #[case("e+12345", "e+12345")]
    fn exponent(#[case] input: &str, #[case] expected: &str) {
        let (rest, result) = parse(super::exponent, input);

        assert_eq!(expected, result);
        assert!(rest.is_empty());
    }

    #[rstest]
    #[case("+Inf", f64::INFINITY)]
    #[case("+Infinity", f64::INFINITY)]
    #[case("-InFiniTY", -f64::INFINITY)]
    #[case("-Inf", -f64::INFINITY)]
    #[case("-Infinity", -f64::INFINITY)]
    #[case("InFinItY", f64::INFINITY)]
    #[case("Inf", f64::INFINITY)]
    #[case("inF", f64::INFINITY)]
    #[case("Infinity", f64::INFINITY)]
    #[case("inf", f64::INFINITY)]
    #[case("inF", f64::INFINITY)]
    #[case("infinity", f64::INFINITY)]
    fn infinity(#[case] input: &str, #[case] expected: f64) {
        let (rest, result) = parse(super::infinity, input);

        assert_eq!(expected, result, "input: {input}");
        assert!(rest.is_empty(), "unparsed input");
    }

    #[rstest]
    #[case(r#"job="prometheus""#, Label::new("job", "prometheus"))]
    #[case(r#"job="☃""#, Label::new("job", "☃"))]
    fn label(#[case] input: &str, #[case] expected: Label<'_>) {
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

    #[rstest]
    #[case("up", Metric::new("up"))]
    #[case(r#"up{job="prometheus"}"#, Metric::new("up").add_label("job", "prometheus"))]
    #[case(r#"up{job="☃"}"#, Metric::new("up").add_label("job", "☃"))]
    fn metric(#[case] input: &str, #[case] expected: Metric<'_>) {
        let (rest, metric) = parse(super::metric, input);

        assert_eq!(expected, metric, "input: {input} metric: {metric:?}");
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

    #[rstest]
    #[case("nan", f64::NAN)]
    #[case("NaN", f64::NAN)]
    #[case("nAN", f64::NAN)]
    fn nan(#[case] input: &str, #[case] expected: f64) {
        let (rest, result) = parse(super::nan, input);

        assert!(result.is_nan(), "non NaN: {result}");
        assert_eq!(
            expected.is_sign_positive(),
            result.is_sign_positive(),
            "wrong sign"
        );
        assert!(rest.is_empty(), "unparsed input");
    }

    #[rstest]
    #[case("+1", 1.0)]
    #[case("+1.", 1.0)]
    #[case("+1.2", 1.2)]
    #[case("+20", 20.0)]
    #[case("+345", 345.0)]
    #[case("-1", -1.0)]
    #[case("-1.", -1.0)]
    #[case("-1.2", -1.2)]
    #[case("-20", -20.0)]
    #[case("-345", -345.0)]
    #[case("1", 1.0)]
    #[case("1.", 1.0)]
    #[case("1.2", 1.2)]
    #[case("20", 20.0)]
    #[case("345", 345.0)]
    #[case("3e5", 300000.0)]
    #[case("3e-5", 0.00003)]
    #[case(".456", 0.456)]
    #[case(".1e-2", 0.001)]
    fn number(#[case] input: &str, #[case] expected: f64) {
        let (rest, result) = parse(super::number, input);

        assert_float_relative_eq!(expected, result, 0.01);
        assert!(rest.is_empty(), "unparsed input");
    }
}
