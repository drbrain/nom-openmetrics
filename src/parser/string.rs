use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::char,
    combinator::{map, value, verify},
    error::context,
    multi::fold_many0,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use nom_language::error::VerboseError;

#[derive(Clone)]
enum Fragment<'a> {
    Normal(&'a str),
    Escaped(char),
}

/// Parse a descriptor string which is not surrounded by quotes
pub fn descriptor(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    context(
        "descriptor string",
        fold_many0(
            alt((escaped, descriptor_normal)),
            String::new,
            |mut body, fragment| {
                match fragment {
                    Fragment::Normal(normal) => body.push_str(normal),
                    Fragment::Escaped(escaped) => body.push(escaped),
                }

                body
            },
        ),
    )
    .parse(input)
}

/// Parse a label string which includes surrounding quotes
pub fn label(input: &str) -> IResult<&str, String, VerboseError<&str>> {
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
    use rstest::rstest;

    #[rstest]
    #[case(r#"hello world!"#, "hello world!")]
    #[case(r#"hello "world!""#, "hello \"world!\"")]
    #[case(r#"\n"#, "\n")]
    #[case(r#"\\"#, "\\")]
    #[case(r#"\""#, "\"")]
    fn descriptor(#[case] input: &str, #[case] expected: &str) {
        let (rest, result) = super::descriptor(input).unwrap();

        assert_eq!(expected.to_string(), result);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }

    #[rstest]
    #[case(r#""hello world!""#, "hello world!")]
    #[case(r#""\n""#, "\n")]
    #[case(r#""\\""#, "\\")]
    #[case(r#""\"""#, "\"")]
    fn label(#[case] input: &str, #[case] expected: &str) {
        let (rest, result) = super::label(input).unwrap();

        assert_eq!(expected.to_string(), result);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }
}
