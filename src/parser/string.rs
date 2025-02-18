use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::{char, none_of},
    combinator::{map, value, verify},
    error::context,
    multi::fold_many0,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use nom_language::error::VerboseError;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Fragment<'a> {
    Normal(&'a str),
    IgnoredEscape(char),
    Escaped(char),
}

/// Parse a descriptor string which is not surrounded by quotes
pub(crate) fn descriptor(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    context(
        "descriptor string",
        fold_many0(
            alt((escaped, descriptor_normal)),
            String::new,
            |mut body, fragment| {
                match fragment {
                    Fragment::Normal(normal) => body.push_str(normal),
                    Fragment::Escaped(escaped) => body.push(escaped),
                    Fragment::IgnoredEscape(ignored) => body.push(ignored),
                }

                body
            },
        ),
    )
    .parse(input)
}

/// Parse a label string which includes surrounding quotes
pub(crate) fn label(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    context(
        "label string",
        delimited(
            char('"'),
            fold_many0(
                alt((escaped, label_normal)),
                String::new,
                |mut body, fragment| {
                    match fragment {
                        Fragment::Normal(normal) => body.push_str(normal),
                        Fragment::Escaped(escaped) => body.push(escaped),
                        Fragment::IgnoredEscape(ignored) => body.push(ignored),
                    }

                    body
                },
            ),
            char('"'),
        ),
    )
    .parse(input)
}

fn escaped(input: &str) -> IResult<&str, Fragment, VerboseError<&str>> {
    preceded(
        char('\\'),
        alt((
            value(Fragment::Escaped('\n'), char('n')),
            value(Fragment::Escaped('\"'), char('"')),
            value(Fragment::Escaped('\\'), char('\\')),
            map(none_of("\\\"n"), Fragment::IgnoredEscape),
        )),
    )
    .parse(input)
}

fn descriptor_normal(input: &str) -> IResult<&str, Fragment, VerboseError<&str>> {
    map(
        verify(is_not("\\\n"), |s: &str| !s.is_empty()),
        Fragment::Normal,
    )
    .parse(input)
}

fn label_normal(input: &str) -> IResult<&str, Fragment, VerboseError<&str>> {
    map(
        verify(is_not("\"\\\n"), |s: &str| !s.is_empty()),
        Fragment::Normal,
    )
    .parse(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::parse;
    use nom::sequence::terminated;
    use rstest::rstest;

    #[rstest]
    #[case("hello world!", "hello world!")]
    #[case("hello \"world!\"", "hello \"world!\"")]
    #[case("\\n", "\n")]
    #[case("\\\\", "\\")]
    #[case("\\\"", "\"")]
    fn descriptor(#[case] input: &str, #[case] expected: &str) {
        let (rest, result) = parse(super::descriptor, input);

        assert_eq!(expected.to_string(), result);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }

    #[rstest]
    #[case("hello world!\n", Fragment::Normal("hello world!"))]
    fn descriptor_normal(#[case] input: &str, #[case] expected: Fragment) {
        let (rest, result) = parse(terminated(super::descriptor_normal, char('\n')), input);

        assert_eq!(expected, result);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }

    #[rstest]
    #[case("\\\"", Fragment::Escaped('\"'))]
    #[case("\\\\", Fragment::Escaped('\\'))]
    #[case("\\n", Fragment::Escaped('\n'))]
    #[case("\\x", Fragment::IgnoredEscape('x'))]
    fn escaped(#[case] input: &str, #[case] expected: Fragment) {
        let (rest, result) = parse(super::escaped, input);

        assert_eq!(expected, result);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }

    #[rstest]
    #[case(r#""hello world!""#, "hello world!")]
    #[case(r#""\n""#, "\n")]
    #[case(r#""\\""#, "\\")]
    #[case(r#""\"""#, "\"")]
    fn label(#[case] input: &str, #[case] expected: &str) {
        let (rest, result) = parse(super::label, input);

        assert_eq!(expected.to_string(), result);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }
}
