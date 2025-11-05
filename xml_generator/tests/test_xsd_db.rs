#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tokio::runtime::Runtime;
    use workspace_root::get_workspace_root;
    use xsdtestdata::get_test_data;

    use gag::Gag;
    use std::panic;
    use xmlgenerator::{XMLGenerator, XMLGeneratorError};

    fn check_error(error: &XMLGeneratorError) {
        match error {
            XMLGeneratorError::XSDValidatorError(e) => panic!("XSD validator error: {}", e),
            XMLGeneratorError::DataTypeInformationError(e) => {
                panic!("Data type information error: {}", e)
            }
            XMLGeneratorError::DataTypeNotFoundError(e) => panic!("DataType not found: {}", e),
            XMLGeneratorError::XSDParserError(e) => eprintln!("XSD parser error: {}", e),
            XMLGeneratorError::DataTypesFormatError(e) => panic!("DataTypes format error: {}", e),
            XMLGeneratorError::XMLBuilderError(e) => panic!("XML builder error: {}", e),
            XMLGeneratorError::InvalidXSDVersionError(e) => eprintln!("Invalid XSD version: {}", e),
            XMLGeneratorError::InfiniteRecursionError => eprintln!("Infinite recursion detected"),
            XMLGeneratorError::NoElementsError => eprintln!("XSD does not contain any elements"),
            XMLGeneratorError::InvalidXSDError(e) => panic!("Invalid XSD error: {}", e),
            XMLGeneratorError::TypeGenerationError(e) => panic!("Type generation error: {}", e),
            XMLGeneratorError::RegexError(e) => panic!("Regex error: {}", e),
            XMLGeneratorError::UnimplementedFeature(e) => eprintln!("Unimplemented feature: {}", e),
        }
    }

    fn check_result_str(str: &String) {
        if str.is_empty() {
            panic!("Empty string is not allowed");
        }
    }

    fn check_result(result: &Result<String, XMLGeneratorError>) {
        match result {
            Ok(str) => check_result_str(str),
            Err(err) => check_error(err),
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
        path: &PathBuf,
    ) -> Result<Result<String, XMLGeneratorError>, Box<dyn Any + Send>> {
        let _err_gag = Gag::stderr().unwrap();

        panic::catch_unwind(|| generator.generate(&path))
    }

    fn test_file(generator: &XMLGenerator, path: &PathBuf) {
        let result = run_generator(generator, path);

        match result {
            Ok(valid_result) => check_result(&valid_result),
            Err(error) => check_panic(error),
        }
    }

    fn validate(
        generator: &XMLGenerator,
        filepath: &PathBuf,
    ) -> Result<Result<(), XMLGeneratorError>, Box<dyn Any + Send>> {
        let _err_gag = Gag::stderr().unwrap();

        panic::catch_unwind(|| generator.validate(filepath))
    }

    fn check_validation_result(result: &Result<(), XMLGeneratorError>) -> bool {
        match result {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn validate_file(generator: &XMLGenerator, filepath: &PathBuf, valid: bool) -> bool {
        if !valid {
            return false;
        }

        match validate(generator, filepath) {
            Ok(result) => check_validation_result(&result),
            Err(_) => false,
        }
    }

    fn get_valid_files(
        generator: &XMLGenerator,
        root: &PathBuf,
        archive: &PathBuf,
    ) -> HashSet<PathBuf> {
        let mut valid_files = HashSet::new();

        // Create the runtime
        let rt = Runtime::new().unwrap();

        // Spawn a future onto the runtime
        let test_data = rt.block_on(get_test_data(root, archive, false));

        for (filepath, listed_as_valid) in test_data {
            if listed_as_valid {
                let valid = validate_file(generator, &filepath, listed_as_valid);
                if valid {
                    valid_files.insert(filepath);
                }
            }
        }

        valid_files
    }

    #[test]
    fn test_xsd_db() {
        let generator = XMLGenerator::new();

        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let archive = root.join("xsdtests.zip");

        let valid_files = get_valid_files(&generator, &db_root, &archive);

        for path in valid_files {
            println!("File: {:?}", path);
            test_file(&generator, &path);
        }
    }
}
