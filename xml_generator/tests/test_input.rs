#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use xmlgenerator::{XMLGenerator, XMLGeneratorError};

    fn check_error(error: XMLGeneratorError) {
        match error {
            XMLGeneratorError::InvalidPathError(_) => {}
            _ => panic!("Incorrect error type"),
        }
    }

    #[test]
    fn test_blank_path() {
        let generator = XMLGenerator::new();

        let path = PathBuf::new();

        match generator.validate(&path) {
            Ok(_) => panic!("No error thrown"),
            Err(e) => check_error(e),
        }

        match generator.generate(&path) {
            Ok(_) => panic!("No error thrown"),
            Err(e) => check_error(e),
        }
    }
}
