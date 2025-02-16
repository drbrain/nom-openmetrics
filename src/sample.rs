use crate::Label;

/// A metric sample
#[derive(Debug, PartialEq)]
pub struct Sample<'a> {
    name: &'a str,
    labels: Vec<Label<'a>>,
    number: f64,
}

impl<'a> Sample<'a> {
    /// Create a `Sample` without labels
    pub fn new(name: &'a str, number: f64) -> Self {
        let labels = vec![];

        Self {
            name,
            labels,
            number,
        }
    }

    /// Create a `Sample` with labels
    pub fn with_labels(name: &'a str, number: f64, labels: Vec<Label<'a>>) -> Self {
        Self {
            name,
            labels,
            number,
        }
    }

    /// Add a label to a `Sample`
    pub fn add_label(mut self, name: &'a str, value: &str) -> Self {
        let label = Label {
            name,
            value: value.into(),
        };

        self.labels.push(label);

        self
    }

    /// [`Label`]s for a `Sample`
    pub fn labels(&self) -> &[Label<'a>] {
        &self.labels
    }

    /// The metric name
    pub fn name(&self) -> &str {
        self.name
    }

    /// The metric value
    pub fn number(&self) -> f64 {
        self.number
    }
}
