use crate::{MetricDescriptor, Sample};

#[derive(Debug, PartialEq)]
pub struct Family<'a> {
    pub descriptors: Vec<MetricDescriptor<'a>>,
    pub samples: Vec<Sample<'a>>,
}

impl<'a> Family<'a> {
    pub fn new(descriptors: Vec<MetricDescriptor<'a>>, samples: Vec<Sample<'a>>) -> Self {
        Self {
            descriptors,
            samples,
        }
    }
}
