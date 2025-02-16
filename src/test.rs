use nom::Parser;
use nom_language::error::{convert_error, VerboseError};

#[track_caller]
pub fn parse<'a, O, F>(mut parser: F, input: &'a str) -> (&'a str, O)
where
    F: Parser<&'a str, Output = O, Error = VerboseError<&'a str>>,
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
