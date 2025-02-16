/// The HELP, TYPE, and UNIT of a metric
#[derive(Debug, PartialEq)]
pub enum MetricDescriptor<'a> {
    /// The `MetricType`
    Type {
        metric: &'a str,
        r#type: MetricType<'a>,
    },
    /// The metric description
    Help { metric: &'a str, help: String },
    /// The metric unit
    Unit { metric: &'a str, unit: &'a str },
}

impl<'a> MetricDescriptor<'a> {
    /// Crate a HELP descriptor
    pub fn help(metric: &'a str, help: String) -> Self {
        Self::Help { metric, help }
    }

    /// Crate a TYPE descriptor
    pub fn r#type(metric: &'a str, r#type: MetricType<'a>) -> Self {
        Self::Type { metric, r#type }
    }

    /// Crate a UNIT descriptor
    pub fn unit(metric: &'a str, unit: &'a str) -> Self {
        Self::Unit { metric, unit }
    }

    /// The metric name
    pub fn metric(&self) -> &'a str {
        match self {
            MetricDescriptor::Type { metric, .. }
            | MetricDescriptor::Help { metric, .. }
            | MetricDescriptor::Unit { metric, .. } => metric,
        }
    }
}

/// The type of the metric
#[derive(Debug, PartialEq, strum::Display)]
pub enum MetricType<'a> {
    /// A counter measures discrete events
    Counter,
    /// A guage measures a current value
    Gauge,
    /// A gauge histogram measures current distributions
    Gaugehistogram,
    /// A distribution of discrete events
    Histogram,
    /// Exposes textual information through labels
    Info,
    /// A series of related boolean values
    Stateset,
    /// A quantile summary of discrete events
    Summary,
    /// Unknown type
    #[strum(to_string = "{0}")]
    Unknown(&'a str),
}
