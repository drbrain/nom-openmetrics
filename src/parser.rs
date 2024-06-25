mod label;
mod metric_descriptor;
mod metric_name;
mod number;

use crate::{Family, Sample};
pub use label::labels;
pub use metric_name::metric_name;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    combinator::{all_consuming, eof, map, opt},
    error::{context, VerboseError},
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
pub use number::number;

use self::metric_descriptor::metric_descriptor;

fn eof_marker(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    context(
        "eof",
        map(tuple((tag("# EOF"), opt(tag("\n")), eof)), |_| ()),
    )(input)
}

/// Parses an OpenMetrics exposition
///
/// This must be terminated with `# EOF`.  See also [`set`]
pub fn exposition(input: &str) -> IResult<&str, Vec<Family>, VerboseError<&str>> {
    context("exposition", terminated(set, eof_marker))(input)
}

pub fn family(input: &str) -> IResult<&str, Family, VerboseError<&str>> {
    context(
        "family",
        map(
            alt((
                tuple((many0(metric_descriptor), many1(sample))),
                tuple((many1(metric_descriptor), many0(sample))),
            )),
            |(descriptors, samples)| {
                eprintln!("descriptors: {descriptors:?}");
                eprintln!("samples: {samples:?}");
                Family::new(descriptors, samples)
            },
        ),
    )(input)
}

/// Parse a set of metrics
///
/// This format is more likely to match prometheus scrape targets
pub fn prometheus(input: &str) -> IResult<&str, Vec<Family>, VerboseError<&str>> {
    context("prometheus", all_consuming(terminated(many0(family), eof)))(input)
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

fn set(input: &str) -> IResult<&str, Vec<Family>, VerboseError<&str>> {
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
    #[case("# EOF")]
    #[case("# EOF\n")]
    fn eof_marker(#[case] input: &str) {
        let (rest, _) = parse(super::eof_marker, input);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }

    #[test]
    fn exposition() {
        let input = "# HELP up \"up help text\"\nup{job=\"prometheus\"} 1\n# EOF\n";

        let (rest, exposition) = parse(super::exposition, input);

        assert_eq!(
            "up",
            exposition
                .first()
                .expect("parsed one family")
                .descriptors
                .first()
                .unwrap()
                .metric()
        );

        assert!(rest.is_empty(), "leftover: {rest:?}");
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
    fn prometheus() {
        let input = "# HELP up \"up help text\"\nup{job=\"prometheus\"} 1\n";

        let (rest, prometheus) = parse(super::prometheus, input);

        assert!(rest.is_empty(), "leftover: {rest:?}");

        assert_eq!(
            "up",
            prometheus
                .first()
                .expect("parsed one family")
                .descriptors
                .first()
                .unwrap()
                .metric()
        );
    }
}
