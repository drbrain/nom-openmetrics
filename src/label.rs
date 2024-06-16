#[derive(Debug, PartialEq)]
pub struct Label<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> Label<'a> {
    pub fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }
}
