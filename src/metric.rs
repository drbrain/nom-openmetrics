use crate::Label;

#[derive(Debug, PartialEq)]
pub struct Metric<'a> {
    name: &'a str,
    labels: Vec<Label<'a>>,
}

impl<'a> Metric<'a> {
    pub fn new(name: &'a str) -> Self {
        let labels = vec![];

        Self { name, labels }
    }

    pub fn with_labels(name: &'a str, labels: Vec<Label<'a>>) -> Self {
        Self { name, labels }
    }

    pub fn add_label(mut self, name: &'a str, value: &'a str) -> Self {
        let label = Label { name, value };

        self.labels.push(label);

        self
    }
}
