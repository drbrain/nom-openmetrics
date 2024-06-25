use crate::{
    parser::{metric_name, string},
    MetricDescriptor, MetricType,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until1, take_while},
    combinator::map,
    error::{context, VerboseError},
    sequence::{preceded, terminated, tuple},
    IResult,
};

use super::metric_name::is_metric_name_char;

pub fn metric_descriptor(input: &str) -> IResult<&str, MetricDescriptor, VerboseError<&str>> {
    context(
        "metric decriptor",
        preceded(
            tag("# "),
            terminated(
                alt((help_descriptor, type_descriptor, unit_descriptor)),
                tag("\n"),
            ),
        ),
    )(input)
}

fn metric_type(input: &str) -> IResult<&str, MetricType, VerboseError<&str>> {
    context(
        "metric type",
        alt((
            map(tag("counter"), |_| MetricType::Counter),
            map(tag("gaugehistogram"), |_| MetricType::Gaugehistogram),
            map(tag("gauge"), |_| MetricType::Gauge),
            map(tag("info"), |_| MetricType::Info),
            map(tag("stateset"), |_| MetricType::Stateset),
            map(tag("summary"), |_| MetricType::Summary),
            map(take_until1("\n"), MetricType::Unknown),
        )),
    )(input)
}

fn help_descriptor(input: &str) -> IResult<&str, MetricDescriptor, VerboseError<&str>> {
    map(
        tuple((
            preceded(tag("HELP "), metric_name),
            preceded(tag(" "), string),
        )),
        |(metric, help)| MetricDescriptor::help(metric, help),
    )(input)
}

fn type_descriptor(input: &str) -> IResult<&str, MetricDescriptor, VerboseError<&str>> {
    map(
        tuple((
            preceded(tag("TYPE "), metric_name),
            preceded(tag(" "), metric_type),
        )),
        |(metric, r#type)| MetricDescriptor::r#type(metric, r#type),
    )(input)
}

fn unit_descriptor(input: &str) -> IResult<&str, MetricDescriptor, VerboseError<&str>> {
    map(
        tuple((
            preceded(tag("UNIT "), metric_name),
            preceded(tag(" "), take_while(is_metric_name_char)),
        )),
        |(metric, unit)| MetricDescriptor::unit(metric, unit),
    )(input)
}

#[cfg(test)]
mod test {
    use crate::{test::parse, MetricDescriptor, MetricType};
    use rstest::rstest;

    #[test]
    fn help_descriptor() {
        let expected = MetricDescriptor::help("metric", "help text here".into());
        let input = "# HELP metric \"help text here\"\n";

        let (rest, descriptor) = parse(super::metric_descriptor, input);

        assert_eq!(expected, descriptor);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }

    #[rstest]
    #[case("counter", MetricType::Counter)]
    #[case("gauge", MetricType::Gauge)]
    #[case("gaugehistogram", MetricType::Gaugehistogram)]
    #[case("junk", MetricType::Unknown("junk"))]
    fn type_descriptor(#[case] input_type: &str, #[case] expected: MetricType) {
        let expected = MetricDescriptor::r#type("metric", expected);

        let input = format!("# TYPE metric {input_type}\n");

        let (rest, descriptor) = parse(super::metric_descriptor, &input);

        assert_eq!(expected, descriptor);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }

    #[test]
    fn unit_descriptor() {
        let expected = MetricDescriptor::unit("metric", "unit");
        let input = "# UNIT metric unit\n";

        let (rest, descriptor) = parse(super::metric_descriptor, input);

        assert_eq!(expected, descriptor);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }
}
