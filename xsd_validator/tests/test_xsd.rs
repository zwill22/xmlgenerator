#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::ReadDir;
    use workspace_root::get_workspace_root;
    use xsdvalidator::XSDValidator;

    fn fetch_test_files(directory: &str) -> ReadDir {
        let root = get_workspace_root();

        let example_dir = root.join("examples").join(directory);
        let paths = fs::read_dir(example_dir).unwrap();

        paths
    }

    fn validate_files(validator: &XSDValidator, files: ReadDir) {
        for file in files {
            let file = file.unwrap().path();
            let result = validator.validate(&file);

            assert!(result.is_ok());
            assert!(result.unwrap());
        }
    }

    #[test]
    fn test_xsd() {
        let validator = XSDValidator::new(true);

        let files = fetch_test_files("working");

        validate_files(&validator, files);
    }
}
