#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::ReadDir;
    use workspace_root::get_workspace_root;
    use xsdvalidator::XSDValidator;

    fn fetch_test_files(directory: &str) -> ReadDir {
        let root = get_workspace_root();

        let example_dir = root.join("examples").join(directory);
        let paths = fs::read_dir(example_dir).expect("Unable to read examples directory");

        paths
    }

    fn validate_files(validator: &XSDValidator, files: ReadDir) {
        files.for_each(|file| {
            let file = file.expect("File not found").path();
            validator.validate(&file).expect("File is invalid");
        });
    }

    #[test]
    fn test_xsd() {
        let validator = XSDValidator::new(false);

        let files = fetch_test_files("working");

        validate_files(&validator, files);
    }
}
