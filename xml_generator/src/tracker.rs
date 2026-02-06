use crate::element::Element;
use crate::error::XMLGeneratorError;
use std::collections::HashSet;
use std::string::String;

pub(crate) struct Tracker {
    values: HashSet<String>,
}

impl Tracker {
    pub(crate) fn new() -> Tracker {
        Tracker {
            values: [].iter().cloned().collect(),
        }
    }

    pub(crate) fn is_root(&self) -> bool {
        self.values.is_empty()
    }

    fn includes(&self, generator: &Element) -> bool {
        let id = generator.get_id();
        self.values.contains(&id)
    }

    pub(crate) fn add(&mut self, generator: &Element) -> Result<(), XMLGeneratorError> {
        if self.includes(generator) {
            return Err(XMLGeneratorError::InfiniteRecursionError);
        }

        let id = generator.get_id();
        self.values.insert(id);

        Ok(())
    }

    pub(crate) fn remove(&mut self, generator: &Element) {
        let id = generator.get_id();
        let result = self.values.remove(&id);
        if !result {
            panic!("Element not in hierarchy");
        }
    }
}
