use std::cmp::{max, min};
use crate::XMLGeneratorError;
use crate::element_generator::ElementGenerator;
use crate::recursion_tracker::RecursionTracker;
use crate::type_generator::TypeGenerator;
use rand::Rng;
use xml_builder::XMLElement;

pub struct GroupGenerator {
    pub(crate) elements: Vec<ElementGenerator>,
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
}

impl GroupGenerator {
    pub(crate) fn new() -> GroupGenerator {
        GroupGenerator {
            elements: vec![],
            min: 0,
            max: None,
        }
    }

    fn get_occurrences(&self) -> usize {
        let mut rng = rand::rng();

        let max_range = 10;
        let max_val = match self.max {
            None => self.min + max_range,
            Some(m) => min(m, self.min + max_range),
        };

        if self.min == max_val {
            return max_val;
        }

        let min_val = max(1, self.min);

        rng.random_range(min_val..=max_val)
    }

    pub(crate) fn generate(
        &self,
        xml_element: &mut XMLElement,
        data_tracker: &mut RecursionTracker,
        data_types: &Vec<TypeGenerator>,
        elements: &Vec<ElementGenerator>,
    ) -> Result<(), XMLGeneratorError> {
        let n = self.get_occurrences();

        for _ in 0..n {
            for element in self.elements.iter() {
                let child = element.generate(data_tracker, data_types, elements)?;

                xml_element.add_child(child)?;
            }
        }

        Ok(())
    }
}

impl PartialEq for GroupGenerator {
    fn eq(&self, other: &Self) -> bool {
        if !self.elements.eq(&other.elements) {
            return false;
        }

        if self.min != other.min {
            return false;
        }

        if !self.max.eq(&other.max) {
            return false;
        }

        true
    }
}
