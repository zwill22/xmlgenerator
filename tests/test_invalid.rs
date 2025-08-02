#[cfg(test)]
mod tests {
    use std::fs::ReadDir;
    use std::{fs, path};
    use std::path::PathBuf;
    use xmlgenerator::{generate_xml, XMLGeneratorError};

    fn fetch_test_files() -> ReadDir {
        let example_dir = path::absolute("./invalid_examples").unwrap();
        let paths = fs::read_dir(example_dir).unwrap();

        paths
    }

    fn check_error(error_string: &String, expected_error: &String) {
        assert_eq!(error_string, expected_error);
    }

    fn test_xml(filepath: &PathBuf, expected: String) {
        let xml = generate_xml(filepath.clone().into_boxed_path());

        assert!(xml.is_err());
        match xml.unwrap_err() {
            XMLGeneratorError::FilepathError => panic!("Filepath error"),
            XMLGeneratorError::ParseError(error) => panic!("Parse error: {}", error),
            XMLGeneratorError::InvalidInputError(error) => check_error(&error, &expected),
            XMLGeneratorError::XMLGenerationError(error) => panic!("XML generation error: {}", error),
            XMLGeneratorError::StringConversionError(error) => panic!("String conversion error: {}", error),
        }
    }

    fn test_error(filename: &str, error: &str) {
        let files = fetch_test_files();

        for file in files {
            let filepath = file.unwrap().path();
            if filepath.ends_with(filename) {
                test_xml(&filepath, error.to_string());
            }
        }
    }

    #[test]
    fn test_invalid_file() {
        test_error("recursive.xsd", "No independent structs found");
        test_error("two_roots.xsd", "Multiple independent structs found!");
    }
}
