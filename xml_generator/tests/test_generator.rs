#[cfg(test)]
mod tests {
    use std::fs::ReadDir;
    use std::path::PathBuf;
    use std::{fs, path};
    use xmlgenerator::generate_xml;
    use workspace_root::get_workspace_root;
    use xmlgenerator::error::XMLGeneratorError;

    fn fetch_test_files(directory: &str) -> ReadDir {
        let root: PathBuf = get_workspace_root();

        let mut example_dir = path::PathBuf::from(root);
        example_dir.push("examples");
        example_dir.push(directory);
        let paths = fs::read_dir(example_dir).unwrap();

        paths
    }

    fn check_error(error_string: &String, expected_error: &String) {
        assert_eq!(error_string, expected_error);
    }

    fn read_file(path: &PathBuf) -> String {
        let path_str = path.to_str().unwrap();
        let message = format!("Could not read file: {}", path_str);
        fs::read_to_string(path).expect(&message)
    }

    fn test_xml(filepath: &PathBuf, expected: String) {
        let contents = read_file(filepath);
        let xml = generate_xml(&contents);

        assert!(xml.is_err());
        match xml.unwrap_err() {
            XMLGeneratorError::DataTypeError(error) => {
                panic!("Data type error: {}", error)
            }
            XMLGeneratorError::XSDParserError(error) => panic!("Parse error: {}", error),
            XMLGeneratorError::DataTypesFormatError(error) => check_error(&error, &expected),
            XMLGeneratorError::XMLBuilderError(error) => {
                panic!("XML generation error: {}", error)
            }
        }
    }

    fn test_error(filename: &str, error: &str) {
        let files = fetch_test_files("invalid");

        for file in files {
            let filepath = file.unwrap().path();
            if filepath.ends_with(filename) {
                test_xml(&filepath, error.to_string());
            }
        }
    }

    // More comprehensive validation tests performed in Python
    fn check_result(result: String) {
        println!("{}", result);
    }

    #[test]
    fn test_examples() {
        let files = fetch_test_files("working");

        for file in files {
            let filepath = file.unwrap().path();
            println!("{}", filepath.display());
            let contents = read_file(&filepath);

            let xml = generate_xml(&contents);

            match xml {
                Ok(result) => check_result(result),
                Err(err) => panic!("{:?}", err),
            }
        }
    }

    #[test]
    fn test_invalid_file() {
        test_error("recursive.xsd", "No independent structs found");
        test_error("two_roots.xsd", "Multiple independent structs found!");
    }

    #[test]
    fn test_invalid_xml() {
        let empty_xml_string = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_string();
        let expected_error: String = "XML Error: Unexpected event: Eof!; position=0".to_string();

        let xml = generate_xml(&empty_xml_string);
        assert!(xml.is_err());
        match xml.unwrap_err() {
            XMLGeneratorError::DataTypeError(_) => panic!("Invalid data type"),
            XMLGeneratorError::XSDParserError(err) => check_error(&err, &expected_error),
            XMLGeneratorError::DataTypesFormatError(_) => panic!("Invalid data error"),
            XMLGeneratorError::XMLBuilderError(_) => panic!("XML generation error"),
        }
    }
}
