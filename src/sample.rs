use crate::Label;

#[derive(Debug, PartialEq)]
pub struct Sample<'a> {
    name: &'a str,
    labels: Vec<Label<'a>>,
    number: f64,
}

impl<'a> Sample<'a> {
    pub fn new(name: &'a str, number: f64) -> Self {
        let labels = vec![];

        Self {
            name,
            labels,
            number,
        }
    }

    pub fn with_labels(name: &'a str, number: f64, labels: Vec<Label<'a>>) -> Self {
        Self {
            name,
            labels,
            number,
        }
    }

    pub fn add_label(mut self, name: &'a str, value: &str) -> Self {
        let label = Label {
            name,
            value: value.into(),
        };

        self.labels.push(label);

        self
    }
}
