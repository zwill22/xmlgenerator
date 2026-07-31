use crate::XSDValidationError;
use roxmltree::Node;
use std::collections::HashSet;

/// Tracker struct used to track recursion in an [roxmltree] XSD/XML tree.
///
/// This struct holds unique values for all nodes previously visited by the parser.
/// If a node is encountered a second time in the same branch, then an error is thrown.
/// This is because a repeated cycle would cause infinite recursion in the tree.
///
/// [roxmltree]: https://crates.io/crates/roxmltree
/// 
/// # Fields
///
/// - `values` (`HashSet<u32>`) - Set of unique values for all parent nodes
///
/// # Example
///
/// ```rust
/// use xsdvalidator::tracker::Tracker;
/// use roxmltree::Document;
///
/// fn check_for_recursion(document: &Document) {
///     let mut tracker = Tracker::new();
///     let root = document.root();
///
///     if let Err(_) = tracker.check_node(&root) {
///         panic!("Infinite recursion detected!");
///     }
/// }
/// ```
pub struct Tracker {
    values: HashSet<u32>,
}

impl Default for Tracker {
    fn default() -> Self {
        Tracker {
            values: [].iter().cloned().collect(),
        }
    }
}

impl Tracker {
    /// Initialises an empty tracker with no nodes recorded
    ///
    /// # Returns
    ///
    /// - [Tracker] - An empty tracker object
    ///
    pub fn new() -> Self {
        Default::default()
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

    /// Recursively check a node and all its children for recursion
    ///
    /// This method adds a nodes unique ID to the tracker and recursively calls the method on all child nodes.
    /// If a node is added to the tracker that is already present, the method returns an [XSDValidationError::XSDRecursionError].
    /// Once all of a node's children have been checked, the current node is removed from the tracker and the function exits.
    ///
    /// # Arguments
    ///
    /// - `node` (`&Node`) - The root [roxmltree] node to check
    ///
    /// # Returns
    ///
    /// - `Result<(), XSDValidationError>` - Returns ok if no recursion detected, else [XSDValidationError::XSDRecursionError]
    ///
    /// # Errors
    ///
    /// - [XSDValidationError] - An [XSDValidationError::XSDRecursionError] is raised if recursion is detected
    ///
    /// # Example
    ///
    /// ```rust
    /// use xsdvalidator::tracker::Tracker;
    /// use roxmltree::Node;
    ///
    /// fn check_node(tracker: &Tracker, node: &Node) {
    ///     let result =  tracker.check_node(&root);
    ///
    ///     match result {
    ///         Ok(_) => {
    ///             println!("Node checked!")
    ///         },
    ///         Err(_) => {
    ///             eprintln!("Infinite recursion detected!");
    ///     }
    /// }
    /// ```
    ///
    pub fn check_node(&mut self, node: &Node) -> Result<(), XSDValidationError> {
        self.add(node)?;

        for child in node.children() {
            self.check_node(&child)?;
        }

        self.remove(node);

        Ok(())
    }
}
