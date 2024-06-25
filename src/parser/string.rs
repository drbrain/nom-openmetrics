use nom::{
    bytes::complete::{tag, take_till},
    error::VerboseError,
    sequence::delimited,
    IResult,
};

// FIX: Implement escaped-string
pub fn string(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(tag("\""), take_till(|c| c == '"'), tag("\""))(input)
}
