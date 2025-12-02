use crate::element_generator::ElementGenerator;

pub struct GroupGenerator {
    pub(crate) elements: Vec<ElementGenerator>,
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
}

impl GroupGenerator {
    pub fn new() -> GroupGenerator {
        GroupGenerator {
            elements: vec![],
            min: 0,
            max: None,
        }
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
