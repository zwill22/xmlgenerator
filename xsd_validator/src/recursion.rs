use crate::XSDValidationError;
use crate::tracker::Tracker;
use file_to_string::read_file;
use roxmltree::Document;
use std::path::PathBuf;

fn parse_document(document: &Document) -> Result<(), XSDValidationError> {
    let mut tracker = Tracker::new();
    let root = document.root();

    tracker.check_node(&root)
}

pub(crate) fn recursion_check(path: &PathBuf) -> Result<(), XSDValidationError> {
    let filedata = match read_file(path) {
        Ok(f) => f,
        Err(_) => return Err(XSDValidationError::ReadFileError),
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
