use crate::{MetricDescriptor, Sample};

/// A set of metric `Sample`s
#[derive(Debug, PartialEq)]
pub struct Family<'a> {
    pub descriptors: Vec<MetricDescriptor<'a>>,
    pub samples: Vec<Sample<'a>>,
}

impl<'a> Family<'a> {
    /// Create a `Family`
    pub fn new(descriptors: Vec<MetricDescriptor<'a>>, samples: Vec<Sample<'a>>) -> Self {
        Self {
            descriptors,
            samples,
        }
    }
}
