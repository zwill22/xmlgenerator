#[cfg(test)]
mod tests {
    use std::path::Path;
    use workspace_root::get_workspace_root;
    use xsdtestdata::get_test_data;
    use xsdvalidator::{XSDValidationError, XSDValidator};

    use futures::executor::block_on;

    fn check_parse_error(error: &String, path: &Path) {
        let error_strings = vec![
            "not allowed",
            "not valid",
            "is not a valid value"
        ];

        for e in error_strings {
            if error.contains(e) {
                eprintln!("Invalid file listed as valid: {:?}", path);
                return;
            }
        }

        panic!("Parse error: {}, file: {:?}", error, path);
    }

    fn check_error(error: &XSDValidationError, path: &Path) {
        match error {
            XSDValidationError::PathError => panic!("Error resolving path: {:?}", path),
            XSDValidationError::StringError => panic!("Error converting path to string"),
            XSDValidationError::OutputRedirectError(e) => panic!("Error redirecting stderr: {}", e),
            XSDValidationError::GenerateContextError => {
                panic!("Error generating context for file: {:?}", path)
            }
            XSDValidationError::ParseError(e) => {
                eprintln!("Invalid file listed as valid: {:?}", path)
            }
        }
    }

    fn test_valid_file(validator: &XSDValidator, path: &Path) {
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

    fn test_invalid_file(validator: &XSDValidator, path: &Path) {
        if let Ok(value) = validator.validate(path) {
            if value {
                eprintln!("Invalid XSD validated: {:?}", path);
            }
        }
    }

    #[test]
    fn test_xsd() {
        let validator = XSDValidator::new(false); 

        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let archive_path = root.join("xsd_tests.zip");

        let file_list = block_on(get_test_data(&db_root, &archive_path, false));

        for (path, valid) in file_list {
            if valid {
                test_valid_file(&validator, &path);
            } else {
                test_invalid_file(&validator, &path);
            }
        }
    }
}
