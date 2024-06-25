#[derive(Debug, PartialEq)]
pub struct Label<'a> {
    pub name: &'a str,
    pub value: String,
}

impl<'a> Label<'a> {
    pub fn new(name: &'a str, value: String) -> Self {
        Self { name, value }
    }
}
