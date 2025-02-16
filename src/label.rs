/// A label for a metric
#[derive(Debug, PartialEq)]
pub struct Label<'a> {
    pub name: &'a str,
    pub value: String,
}

impl<'a> Label<'a> {
    /// Create a `Label`
    pub fn new(name: &'a str, value: String) -> Self {
        Self { name, value }
    }
}
