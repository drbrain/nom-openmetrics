use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{digit0, digit1},
    combinator::{map, map_res, opt, recognize},
    error::{context, VerboseError},
    sequence::{terminated, tuple},
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

fn nan(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    map(tag_no_case("nan"), |_| f64::NAN)(input)
}

pub fn number(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
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
    use crate::test::parse;
    use assert_float_eq::*;
    use rstest::rstest;

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
