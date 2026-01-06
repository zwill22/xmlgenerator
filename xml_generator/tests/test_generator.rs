#[cfg(test)]
mod tests {
    use gag::Gag;
    use std::fs::ReadDir;
    use std::path::PathBuf;
    use std::{fs, panic};
    use workspace_root::get_workspace_root;
    use xmlgenerator::XMLGenerator;
    use xmlgenerator::error::XMLGeneratorError;

    fn fetch_test_files(directory: &str) -> ReadDir {
        let root: PathBuf = get_workspace_root();

        let mut example_dir = PathBuf::from(root);
        example_dir.push("examples");
        example_dir.push(directory);
        let paths = fs::read_dir(example_dir).unwrap();

        paths
    }

    fn check_error(error: &XMLGeneratorError, expected_error: &String) {
        match error {
            XMLGeneratorError::InvalidPathError(e) => panic!("Invalid path error: {}", e),
            XMLGeneratorError::XSDValidatorError(e) => panic!("XSD validator error: {}", e),
            XMLGeneratorError::DataTypeInformationError(e) => {
                panic!("Data type information error: {}", e)
            }
            XMLGeneratorError::DataTypeNotFoundError(e) => panic!("Data type not found: {}", e),
            XMLGeneratorError::XSDParserError(e) => panic!("XSD parser error: {}", e),
            XMLGeneratorError::DataTypesFormatError(e) => panic!("DataTypes format error: {}", e),
            XMLGeneratorError::XMLBuilderError(e) => panic!("XML builder error: {}", e),
            XMLGeneratorError::InvalidXSDVersionError(e) => panic!("Invalid XSD version: {}", e),
            XMLGeneratorError::InfiniteRecursionError => {
                assert_eq!("Recursion detected", expected_error)
            }
            XMLGeneratorError::NoElementsError => panic!("No elements included in XSD"),
            XMLGeneratorError::InvalidXSDError(e) => panic!("Invalid XSD error: {}", e),
            XMLGeneratorError::NoIndependentElementsError => {},
            XMLGeneratorError::MultipleRootsError => {},
            XMLGeneratorError::TypeGenerationError(e) => panic!("Type generation error: {}", e),
            XMLGeneratorError::RegexError(e) => panic!("Regex error: {}", e),
            XMLGeneratorError::UnimplementedFeature(e) => panic!("Unimplemented feature: {}", e),
        }
    }

    fn test_xml(generator: &XMLGenerator, filepath: &PathBuf, expected: &String) {
        match generator.generate(filepath) {
            Ok(_) => panic!("No error thrown for invalid result"),
            Err(error) => check_error(&error, expected),
        }
    }

    fn test_error(generator: &XMLGenerator, filename: &str, error: &str) {
        let files = fetch_test_files("invalid");

        for file in files {
            let filepath = file.unwrap().path();
            if filepath.ends_with(filename) {
                test_xml(generator, &filepath, &error.to_string());
            }
        }
    }

    fn run_generator(
        generator: &XMLGenerator,
        filepath: &PathBuf,
    ) -> Result<String, XMLGeneratorError> {
        let _err_gag = Gag::stderr().unwrap();

        generator.generate(&filepath)
    }

    #[test]
    fn test_examples() {
        let generator = XMLGenerator::new();
        let files = fetch_test_files("working");

        for file in files {
            let filepath = file.unwrap().path();

            match run_generator(&generator, &filepath) {
                Ok(_) => {}
                Err(err) => panic!("XMLGenerator error: {:?}", err),
            }
        }

        test_error(&generator, "recursive.xsd", "Recursion detected");
        test_error(
            &generator,
            "two_roots.xsd",
            "not implemented: Multiple independent (root) elements",
        );
    }
}
