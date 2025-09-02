pub use crate::error::XSDValidationError;
use crate::parser::Parser;
use crate::warning_handler::WarningHandler;
use libxml2_rs::{xmlCleanupParser, xmlInitParser};
use std::env::{current_dir, set_current_dir};
use std::path::Path;

mod error;
mod parser;
mod schema;
mod warning_handler;

pub struct XSDValidator {
    print_warnings: bool,
}

impl XSDValidator {
    pub fn new(print_warnings: bool) -> XSDValidator {
        unsafe { xmlInitParser() };

        XSDValidator { print_warnings }
    }

    fn validate_file(&self, path: &Path) -> Result<bool, XSDValidationError> {
        let mut errors: Vec<String> = Vec::new();

        let parser = Parser::new(path)?;
        parser.setup_error_handler(&mut errors);

        let mut warning_handler = WarningHandler::new();

        if !self.print_warnings {
            warning_handler.redirect()?;
        }

        let valid = parser.parse();

        warning_handler.restore();

        for error in &errors {
            if error.to_lowercase().contains("skipping") {
                if self.print_warnings {
                    eprintln!("{}", error);
                }
            } else {
                return Err(XSDValidationError::ParseError(error.clone()));
            }
        }

        Ok(valid)
    }

    pub fn validate(&self, path: &Path) -> Result<bool, XSDValidationError> {
        let wd = current_dir().expect("current_dir() failed");
        let file_dir = path.parent().expect("Filepath has no parent");

        set_current_dir(file_dir).expect("set_current_dir() failed");

        let result = self.validate_file(path);

        set_current_dir(wd).expect("set_current_dir() failed");

        result
    }
}

impl Drop for XSDValidator {
    fn drop(&mut self) {
        unsafe { xmlCleanupParser() };
    }
}
