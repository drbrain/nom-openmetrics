mod label;
mod metric_descriptor;
mod metric_name;
mod number;

use crate::{Family, Sample};
pub use label::labels;
pub use metric_name::metric_name;
use nom::{
    bytes::complete::{tag, take_till},
    combinator::{map, opt},
    error::{context, VerboseError},
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
pub use number::number;

use self::metric_descriptor::metric_descriptor;

/// Parses an OpenMetrics exposition
///
/// This must be terminated with `# EOF`.  See also [`set`]
pub fn exposition(input: &str) -> IResult<&str, Vec<Family>, VerboseError<&str>> {
    context(
        "exposition",
        terminated(set, tuple((tag("# EOF"), opt(tag("\n"))))),
    )(input)
}

pub fn family(input: &str) -> IResult<&str, Family, VerboseError<&str>> {
    context(
        "family",
        map(
            tuple((many0(metric_descriptor), many0(sample))),
            |(descriptors, samples)| Family::new(descriptors, samples),
        ),
    )(input)
}

pub fn sample(input: &str) -> IResult<&str, Sample, VerboseError<&str>> {
    context(
        "sample",
        map(
            terminated(
                tuple((metric_name, opt(labels), preceded(tag(" "), metric_value))),
                tag("\n"),
            ),
            |(name, labels, number)| {
                if let Some(labels) = labels {
                    Sample::with_labels(name, number, labels)
                } else {
                    Sample::new(name, number)
                }
            },
        ),
    )(input)
}

/// Parse a set of metrics
///
/// This format is more likely to match prometheus scrape targets
pub fn set(input: &str) -> IResult<&str, Vec<Family>, VerboseError<&str>> {
    context("set", many0(family))(input)
}

/// Matches a metric value
fn metric_value(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    context("metric value", number)(input)
}

// FIX: Implement escaped-string
pub fn string(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(tag("\""), take_till(|c| c == '"'), tag("\""))(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test::parse, MetricDescriptor};
    use rstest::rstest;

    #[rstest]
    #[case("up 1\n", Sample::new("up", 1.0))]
    #[case("up{job=\"prometheus\"} 2\n", Sample::new("up", 2.0).add_label("job", "prometheus"))]
    #[case("up{job=\"☃\"} 1\n", Sample::new("up", 1.0).add_label("job", "☃"))]
    fn sample(#[case] input: &str, #[case] expected: Sample<'_>) {
        let (rest, metric) = parse(super::sample, input);

        assert_eq!(expected, metric, "input: {input} metric: {metric:?}");
        assert!(rest.is_empty());
    }

    #[test]
    fn family() {
        let input = "# HELP up \"up help text\"\nup{job=\"prometheus\"} 1\n";

        let (rest, family) = parse(super::family, input);

        assert_eq!(
            &MetricDescriptor::help("up", "up help text"),
            family.descriptors.first().expect("parsed one descriptor")
        );

        assert_eq!(
            &Sample::new("up", 1.0).add_label("job", "prometheus"),
            family.samples.first().expect("parsed one sample")
        );

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }
}
