use crate::{
    parser::{metric_name, string},
    MetricDescriptor, MetricType,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until1, take_while},
    character::complete::char,
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
                char('\n'),
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
            preceded(char(' '), string::descriptor),
        )),
        |(metric, help)| MetricDescriptor::help(metric, help),
    )(input)
}

fn type_descriptor(input: &str) -> IResult<&str, MetricDescriptor, VerboseError<&str>> {
    map(
        tuple((
            preceded(tag("TYPE "), metric_name),
            preceded(char(' '), metric_type),
        )),
        |(metric, r#type)| MetricDescriptor::r#type(metric, r#type),
    )(input)
}

fn unit_descriptor(input: &str) -> IResult<&str, MetricDescriptor, VerboseError<&str>> {
    map(
        tuple((
            preceded(tag("UNIT "), metric_name),
            preceded(char(' '), take_while(is_metric_name_char)),
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
        let input = "HELP adsb_aircraft_mlat_recent Number of aircraft observed with a position determined by multilateration in the last minute";

        let (rest, descriptor) = super::help_descriptor(input).unwrap();

        let expected =
            MetricDescriptor::help(
            "adsb_aircraft_mlat_recent", 
            "Number of aircraft observed with a position determined by multilateration in the last minute".into()
        );

        assert_eq!(expected, descriptor);

        assert!(rest.is_empty());
    }

    #[rstest]
    #[case(
        "# HELP up Job is scrapeable\n",
        MetricDescriptor::help("up", "Job is scrapeable".into())
    )]
    #[case(
        "# HELP adsb_aircraft_mlat_recent Number of aircraft observed with a position determined by multilateration in the last minute\n",
        MetricDescriptor::help("adsb_aircraft_mlat_recent", "Number of aircraft observed with a position determined by multilateration in the last minute".into())
    )]
    fn metric_descriptor_help(#[case] input: &str, #[case] expected: MetricDescriptor) {
        let (rest, descriptor) = parse(super::metric_descriptor, input);

        assert_eq!(expected, descriptor);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }

    #[rstest]
    #[case("counter", MetricType::Counter)]
    #[case("gauge", MetricType::Gauge)]
    #[case("gaugehistogram", MetricType::Gaugehistogram)]
    #[case("junk", MetricType::Unknown("junk"))]
    fn metric_descriptor_type(#[case] input_type: &str, #[case] expected: MetricType) {
        let expected = MetricDescriptor::r#type("metric", expected);

        let input = format!("# TYPE metric {input_type}\n");

        let (rest, descriptor) = parse(super::metric_descriptor, &input);

        assert_eq!(expected, descriptor);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }

    #[test]
    fn metric_descriptor_unit() {
        let expected = MetricDescriptor::unit("metric", "unit");
        let input = "# UNIT metric unit\n";

        let (rest, descriptor) = parse(super::metric_descriptor, input);

        assert_eq!(expected, descriptor);

        assert!(rest.is_empty(), "leftover: {rest:?}");
    }
}
