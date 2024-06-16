mod label;
mod metric_name;
mod number;

use crate::Sample;
pub use label::labels;
pub use metric_name::metric_name;
use nom::{
    combinator::{map, opt},
    error::{context, VerboseError},
    sequence::tuple,
    IResult,
};
pub use number::number;

fn sample(input: &str) -> IResult<&str, Sample, VerboseError<&str>> {
    context(
        "metric",
        map(tuple((metric_name, opt(labels))), |(name, labels)| {
            if let Some(labels) = labels {
                Sample::with_labels(name, labels)
            } else {
                Sample::new(name)
            }
        }),
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
    use rstest::rstest;

    #[rstest]
    #[case("up", Sample::new("up"))]
    #[case(r#"up{job="prometheus"}"#, Sample::new("up").add_label("job", "prometheus"))]
    #[case(r#"up{job="☃"}"#, Sample::new("up").add_label("job", "☃"))]
    fn sample(#[case] input: &str, #[case] expected: Sample<'_>) {
        let (rest, metric) = parse(super::sample, input);

        assert_eq!(expected, metric, "input: {input} metric: {metric:?}");
        assert!(rest.is_empty());
    }
}
