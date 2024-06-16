mod label;
mod metric_name;
mod number;

use crate::Metric;
pub use label::labels;
pub use metric_name::metric_name;
use nom::{
    combinator::{map, opt},
    error::{context, VerboseError},
    sequence::tuple,
    IResult,
};
pub use number::number;

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
    #[case("up", Metric::new("up"))]
    #[case(r#"up{job="prometheus"}"#, Metric::new("up").add_label("job", "prometheus"))]
    #[case(r#"up{job="☃"}"#, Metric::new("up").add_label("job", "☃"))]
    fn metric(#[case] input: &str, #[case] expected: Metric<'_>) {
        let (rest, metric) = parse(super::metric, input);

        assert_eq!(expected, metric, "input: {input} metric: {metric:?}");
        assert!(rest.is_empty());
    }
}
