use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::char,
    combinator::{map, map_res, opt},
    error::context,
    number::complete::recognize_float,
    sequence::terminated,
    IResult, Parser,
};
use nom_language::error::VerboseError;

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
    )
    .parse(input)
}

fn nan(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    map(tag_no_case("nan"), |_| f64::NAN).parse(input)
}

/// Parse a number
pub(crate) fn number(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    context("number", alt((real_number, infinity, nan))).parse(input)
}

fn real_number(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    map_res(recognize_float, |float: &str| float.parse::<f64>()).parse(input)
}

fn sign(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    map(alt((char('-'), char('+'))), |sign| match sign {
        '-' => -1.0,
        '+' => 1.0,
        _ => unreachable!(),
    })
    .parse(input)
}

#[cfg(test)]
mod test {
    use crate::test::parse;
    use assert_float_eq::*;
    use rstest::rstest;

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
    #[case("0.567", 0.567)]
    #[case("0.5e-8", 0.000000005)]
    fn number(#[case] input: &str, #[case] expected: f64) {
        let (rest, result) = parse(super::number, input);

        assert!(rest.is_empty(), "unparsed input (result: {result})");
        assert_float_relative_eq!(expected, result, 0.01);
    }
}
