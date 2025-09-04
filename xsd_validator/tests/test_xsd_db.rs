#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};
    use workspace_root::get_workspace_root;
    use xsdtestdata::get_test_data;
    use xsdvalidator::{XSDValidationError, XSDValidator};

    use futures::executor::block_on;

    fn check_error(error: &XSDValidationError, path: &Path) {
        match error {
            XSDValidationError::PathError => panic!("Error resolving path: {:?}", path),
            XSDValidationError::StringError => panic!("Error converting path to string"),
            XSDValidationError::ReadFileError => panic!("Error reading file: {:?}", path),
            XSDValidationError::OutputRedirectError(e) => panic!("Error redirecting stderr: {}", e),
            XSDValidationError::GenerateContextError => {
                panic!("Error generating context for file: {:?}", path)
            }
            XSDValidationError::XSDRecursionError => {
                eprintln!("File includes recursive loop: {:?}", path);
            }
            XSDValidationError::ParseError(_) => {
                eprintln!("Invalid file listed as valid: {:?}", path)
            }
        }
    }

    fn test_valid_file(validator: &XSDValidator, path: &PathBuf) {
        let result = validator.validate(path);

        match result {
            Ok(value) => {
                if !value {
                    panic!("Invalid XSD: {:?}", path);
                }
            }
            Err(e) => check_error(&e, &path),
        }
    }

    fn test_invalid_file(validator: &XSDValidator, path: &PathBuf) {
        if let Ok(value) = validator.validate(path) {
            if value {
                eprintln!("Invalid XSD validated: {:?}", path);
            }
        }
    }

    fn validate(validator: &XSDValidator, files: &HashSet<(PathBuf, bool)>) {
        for (path, valid) in files {
            if *valid {
                test_valid_file(&validator, &path);
            } else {
                test_invalid_file(&validator, &path);
            }
        }
    }

    #[test]
    fn test_xsd() {
        let validator = XSDValidator::new(false);

        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let archive_path = root.join("xsd_tests.zip");

        let test_files = block_on(get_test_data(&db_root, &archive_path, true));

        validate(&validator, &test_files);
    }
}
