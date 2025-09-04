use crate::XSDValidationError;
use crate::tracker::Tracker;
use roxmltree::{Document, Node};
use std::fs::read_to_string;
use std::path::Path;

fn parse_document(document: &Document) -> Result<(), XSDValidationError> {
    let mut tracker = Tracker::new();
    let root = document.root();

    tracker.check_node(&root)
}

pub(crate) fn recursion_check(path: &Path) -> Result<(), XSDValidationError> {
    let filedata = match read_to_string(path) {
        Ok(f) => f,
        Err(e) => {
            return Err(XSDValidationError::ReadFileError);
        }
    };
    let doc = match Document::parse(&filedata) {
        Ok(d) => d,
        Err(_) => {
            return Err(XSDValidationError::ParseError(
                path.to_str().unwrap().to_string(),
            ));
        }
    };

    parse_document(&doc)
}
