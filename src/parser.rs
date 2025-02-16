mod label;
mod metric_descriptor;
mod metric_name;
mod number;
mod string;

use crate::{Family, Sample};
pub use label::labels;
use metric_descriptor::metric_descriptor;
pub use metric_name::metric_name;
use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{all_consuming, cut, eof, map, opt},
    error::context,
    multi::{many0, many1},
    sequence::{pair, preceded, terminated},
    IResult, Parser,
};
use nom_language::error::VerboseError;
pub use number::number;

fn eof_marker(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    context("eof", map((tag("# EOF"), opt(char('\n')), eof), |_| ())).parse(input)
}

/// Parses an OpenMetrics exposition
///
/// This must be terminated with `# EOF`.  See also [`set`]
pub fn openmetrics(input: &str) -> IResult<&str, Vec<Family>, VerboseError<&str>> {
    context("openmetrics", terminated(set, eof_marker)).parse(input)
}

pub fn family(input: &str) -> IResult<&str, Family, VerboseError<&str>> {
    context(
        "family",
        map(
            pair(many0(metric_descriptor), many1(sample)),
            |(descriptors, samples)| Family::new(descriptors, samples),
        ),
    )
    .parse(input)
}

/// Parse a set of metrics
///
/// This format is more likely to match prometheus scrape targets
pub fn prometheus(input: &str) -> IResult<&str, Vec<Family>, VerboseError<&str>> {
    context(
        "prometheus",
        all_consuming(terminated(many0(family), cut(eof))),
    )
    .parse(input)
}

pub fn sample(input: &str) -> IResult<&str, Sample, VerboseError<&str>> {
    context(
        "sample",
        map(
            terminated(
                (metric_name, opt(labels), preceded(char(' '), metric_value)),
                char('\n'),
            ),
            |(name, labels, number)| {
                if let Some(labels) = labels {
                    Sample::with_labels(name, number, labels)
                } else {
                    Sample::new(name, number)
                }
            },
        ),
    )
    .parse(input)
}

fn set(input: &str) -> IResult<&str, Vec<Family>, VerboseError<&str>> {
    context("set", many0(family)).parse(input)
}

/// Matches a metric value
fn metric_value(input: &str) -> IResult<&str, f64, VerboseError<&str>> {
    context("metric value", number).parse(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test::parse, MetricDescriptor, MetricType};
    use rstest::rstest;

    #[rstest]
    #[case("# EOF")]
    #[case("# EOF\n")]
    fn eof_marker(#[case] input: &str) {
        let (rest, _) = parse(super::eof_marker, input);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }

    #[test]
    fn openmetrics() {
        let input = "# HELP up up help text\nup{job=\"prometheus\"} 1\n# EOF\n";

        let (rest, openmetrics) = parse(super::openmetrics, input);

        assert_eq!(
            "up",
            openmetrics
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
        let input = "# TYPE up gauge\n# HELP up up help text\nup{job=\"prometheus\"} 1\nup{job=\"grafana\"} 0\n";

        let (rest, family) = parse(super::family, input);

        assert_eq!(
            MetricDescriptor::r#type("up", MetricType::Gauge),
            family.descriptors[0]
        );

        assert_eq!(
            MetricDescriptor::help("up", "up help text".into()),
            family.descriptors[1]
        );

        assert_eq!(
            Sample::new("up", 1.0).add_label("job", "prometheus"),
            family.samples[0]
        );

        assert_eq!(
            Sample::new("up", 0.0).add_label("job", "grafana"),
            family.samples[1]
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
        let input = "# HELP up up help text\nup{job=\"prometheus\"} 1\n";

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
