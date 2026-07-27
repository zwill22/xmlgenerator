#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use xsdvalidator::{ XSDValidationError, XSDValidator };

    fn check_error(error: XSDValidationError) {
        match error {
            XSDValidationError::PathError => {}
            _ => panic!("Incorrect error type"),
        }
    }

    #[test]
    fn test_blank_path() {
        let validator = XSDValidator::new(false);

        let path = PathBuf::new();

        match validator.validate(&path) {
            Ok(_) => panic!("No error thrown"),
            Err(e) => check_error(e),
        }
    }
}
