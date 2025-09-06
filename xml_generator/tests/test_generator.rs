#[cfg(test)]
mod tests {
    use gag::Gag;
    use std::any::Any;
    use std::fs::ReadDir;
    use std::path::PathBuf;
    use std::{fs, panic, path};
    use workspace_root::get_workspace_root;
    use xmlgenerator::XMLGenerator;
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

    fn check_invalid_result(result: Result<String, XMLGeneratorError>, expected_error: &String) {
        assert!(result.is_err());

        match result.unwrap_err() {
            XMLGeneratorError::DataTypeError(error) => {
                panic!("Data type error: {}", error)
            }
            XMLGeneratorError::XSDParserError(error) => panic!("Parse error: {}", error),
            XMLGeneratorError::DataTypesFormatError(error) => check_error(&error, &expected_error),
            XMLGeneratorError::XMLBuilderError(error) => {
                panic!("XML generation error: {}", error)
            }
            XMLGeneratorError::XSDValidatorError(error) => panic!("XSD Validator error: {}", error),
            XMLGeneratorError::InvalidXSDError(error) => check_error(&error, &expected_error),
        }
    }

    fn check_invalid_panic(error: Box<dyn Any>, expected_error: &String) {
        if let Some(s) = error.downcast_ref::<&str>() {
            assert_eq!(s, expected_error);
        } else if let Some(s) = error.downcast_ref::<String>() {
            assert_eq!(s, expected_error);
        } else {
            panic!("Unknown error");
        }
    }

    fn test_xml(generator: &XMLGenerator, filepath: &PathBuf, expected: &String) {
        let xml = panic::catch_unwind(|| generator.generate(filepath));
        match xml {
            Ok(result) => check_invalid_result(result, expected),
            Err(error) => check_invalid_panic(error, expected),
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

    // More comprehensive validation tests performed in Python
    fn check_result(result: &Result<String, XMLGeneratorError>) {
        match result {
            Ok(_) => {}
            Err(err) => panic!("XMLGenerator error: {:?}", err),
        }
    }

    fn check_error_string(string: &String) {
        if string.is_empty() {
            panic!("Unknown error");
        }

        if string.contains("not implemented") {
            eprintln!("Implementation error: {}", string);
            return;
        } else {
            panic!("Error: {}", string);
        }
    }

    fn check_panic(error: Box<dyn Any>) {
        if let Some(s) = error.downcast_ref::<&str>() {
            check_error_string(&s.to_string());
        } else if let Some(s) = error.downcast_ref::<String>() {
            check_error_string(s)
        } else {
            panic!("Unknown error");
        }
    }

    fn run_generator(
        generator: &XMLGenerator,
        filepath: &PathBuf,
    ) -> Result<Result<String, XMLGeneratorError>, Box<dyn Any + Send>> {
        let _err_gag = Gag::stderr().unwrap();

        panic::catch_unwind(|| generator.generate(&filepath))
    }

    #[test]
    fn test_examples() {
        let generator = XMLGenerator::new();
        let files = fetch_test_files("working");

        for file in files {
            let filepath = file.unwrap().path();

            let result = run_generator(&generator, &filepath);

            match result {
                Ok(result) => check_result(&result),
                Err(error) => check_panic(error),
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
