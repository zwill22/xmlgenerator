use crate::element_generator::ElementGenerator;
use crate::error::XMLGeneratorError;
use std::collections::HashSet;
use std::string::String;

pub(crate) struct RecursionTracker {
    values: HashSet<String>,
}

impl RecursionTracker {
    pub(crate) fn new() -> RecursionTracker {
        RecursionTracker {
            values: [].iter().cloned().collect(),
        }
    }

    fn includes(&self, generator: &ElementGenerator) -> bool {
        let id = generator.get_id();
        self.values.contains(&id)
    }

    pub(crate) fn add(&mut self, generator: &ElementGenerator) -> Result<(), XMLGeneratorError> {
        if self.includes(generator) {
            return Err(XMLGeneratorError::InfiniteRecursionError);
        }

        let id = generator.get_id();
        self.values.insert(id);

        Ok(())
    }

    pub(crate) fn remove(&mut self, generator: &ElementGenerator) {
        let id = generator.get_id();
        let result = self.values.remove(&id);
        if !result {
            panic!("Element not in hierarchy");
        }
    }
}
