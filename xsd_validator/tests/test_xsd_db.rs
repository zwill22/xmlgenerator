#[cfg(test)]
mod tests {
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
            XSDValidationError::ParseError(e) => {
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

    fn validate(validator: &XSDValidator, file_list: &Vec<(PathBuf, bool)>, ignore_list: &Vec<PathBuf>) {
        for (path, valid) in file_list {
            if ignore_list.contains(path) {
                continue;
            }
            println!("Validating {:?}", path);
            if *valid {
                test_valid_file(&validator, &path);
            } else {
                test_invalid_file(&validator, &path);
            }
        }
    }

    fn file_path(db_root: &PathBuf, path_string: &str) -> PathBuf {
        db_root.join(path_string)
    }

    #[test]
    fn test_xsd() {
        let validator = XSDValidator::new(true);

        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let archive_path = root.join("xsd_tests.zip");

        let file_list = block_on(get_test_data(&db_root, &archive_path, false));
        let ignore_list = vec![
            file_path(&db_root, "msData/particles/particlesZ012.xsd"),
            file_path(&db_root, "msData/particles/particlesZ015.xsd"),
            file_path(&db_root, "msData/particles/particlesZ020.xsd"),
        ];
        validate(&validator, &file_list, &ignore_list);
    }
}
