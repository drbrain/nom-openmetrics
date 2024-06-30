use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::char,
    combinator::{map, value, verify},
    error::VerboseError,
    multi::fold_many0,
    sequence::preceded,
    IResult,
};

#[derive(Clone)]
enum Fragment<'a> {
    Normal(&'a str),
    Escaped(char),
}

pub fn string(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    fold_many0(alt((escaped, normal)), String::new, |mut body, fragment| {
        match fragment {
            Fragment::Normal(normal) => body.push_str(normal),
            Fragment::Escaped(escaped) => body.push(escaped),
        }

        body
    })(input)
}

fn escaped(input: &str) -> IResult<&str, Fragment, VerboseError<&str>> {
    preceded(
        char('\\'),
        alt((
            value(Fragment::Escaped('\n'), char('n')),
            value(Fragment::Escaped('\"'), char('"')),
            value(Fragment::Escaped('\\'), char('\\')),
        )),
    )(input)
}

fn normal(input: &str) -> IResult<&str, Fragment, VerboseError<&str>> {
    map(
        verify(is_not("\"\\\n"), |s: &str| !s.is_empty()),
        Fragment::Normal,
    )(input)
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    #[rstest]
    #[case(r#"hello world!"#, "hello world!")]
    #[case(r#"\n"#, "\n")]
    #[case(r#"\\"#, "\\")]
    #[case(r#"\""#, "\"")]
    fn string(#[case] input: &str, #[case] expected: &str) {
        let (rest, result) = super::string(input).unwrap();

        assert_eq!(expected.to_string(), result);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }
}
