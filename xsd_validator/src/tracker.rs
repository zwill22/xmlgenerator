use crate::XSDValidationError;
use roxmltree::Node;
use std::collections::HashSet;

pub struct Tracker {
    values: HashSet<u32>,
}

impl Tracker {
    pub fn new() -> Tracker {
        Tracker {
            values: [].iter().cloned().collect(),
        }
    }

    fn includes(&self, node: &Node) -> bool {
        let id = node.id().get();
        self.values.contains(&id)
    }

    fn add(&mut self, node: &Node) -> Result<(), XSDValidationError> {
        if self.includes(node) {
            return Err(XSDValidationError::XSDRecursionError);
        }

        let id = node.id().get();
        self.values.insert(id);

        Ok(())
    }

    fn remove(&mut self, node: &Node) {
        let id = node.id().get();
        let result = self.values.remove(&id);
        if !result {
            panic!("Element not in hierarchy");
        }
    }

    pub fn check_node(&mut self, node: &Node) -> Result<(), XSDValidationError> {
        self.add(&node)?;

        for child in node.children() {
            self.check_node(&child)?;
        }

        self.remove(&node);

        Ok(())
    }
}
