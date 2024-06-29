#[derive(Debug, PartialEq)]
pub enum MetricDescriptor<'a> {
    Type {
        metric: &'a str,
        r#type: MetricType<'a>,
    },
    Help {
        metric: &'a str,
        help: String,
    },
    Unit {
        metric: &'a str,
        unit: &'a str,
    },
}

impl<'a> MetricDescriptor<'a> {
    pub fn help(metric: &'a str, help: String) -> Self {
        Self::Help { metric, help }
    }

    pub fn r#type(metric: &'a str, r#type: MetricType<'a>) -> Self {
        Self::Type { metric, r#type }
    }

    pub fn unit(metric: &'a str, unit: &'a str) -> Self {
        Self::Unit { metric, unit }
    }

    pub fn metric(&self) -> &'a str {
        match self {
            MetricDescriptor::Type { metric, .. }
            | MetricDescriptor::Help { metric, .. }
            | MetricDescriptor::Unit { metric, .. } => metric,
        }
    }
}

#[derive(Debug, PartialEq, strum::Display)]
pub enum MetricType<'a> {
    Counter,
    Gauge,
    Gaugehistogram,
    Histogram,
    Info,
    Stateset,
    Summary,
    #[strum(to_string = "{0}")]
    Unknown(&'a str),
}
