#[cfg(test)]
mod tests {
    use rstest::{ fixture, rstest };
    use std::path::{ Path, PathBuf };
    use workspace_root::get_workspace_root;
    use xsdtestdata::XsdTestData;
    use xsdvalidator::{ XSDValidationError, XSDValidator };

    fn check_error(error: &XSDValidationError, path: &Path) -> String {
        match error {
            XSDValidationError::PathError => panic!("Error resolving path: {:?}", path),
            XSDValidationError::StringError => panic!("Error converting path to string"),
            XSDValidationError::ReadFileError => panic!("Error reading file: {:?}", path),
            XSDValidationError::OutputRedirectError(e) => panic!("Error redirecting stderr: {}", e),
            XSDValidationError::GenerateContextError => {
                panic!("Error generating context for file: {:?}", path)
            }
            XSDValidationError::XSDRecursionError => {
                format!("File includes recursive loop: {:?}", path)
            }
            XSDValidationError::ParseError(_) => {
                format!("Invalid file listed as valid: {:?}", path)
            }
        }
    }

    fn test_valid_file(validator: &XSDValidator, path: &PathBuf) -> String {
        match validator.validate(path) {
            Ok(value) => {
                if !value {
                    panic!("Invalid XSD: {:?}", path);
                }

                "".to_string()
            }
            Err(e) => check_error(&e, path),
        }
    }

    fn test_invalid_file(validator: &XSDValidator, path: &PathBuf) -> String {
        if let Ok(value) = validator.validate(path) && value {
            return format!("Invalid XSD validated: {:?}", path);
        }

        "".to_string()
    }

    #[fixture]
    #[once]
    fn validator() -> XSDValidator {
        XSDValidator::new(false)
    }

    #[fixture]
    #[once]
    fn test_data() -> XsdTestData {
        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let archive_path = root.join("xsd_tests.zip");

        XsdTestData::new(&db_root, &archive_path)
    }

    #[rstest]
    #[case::oracle_data("oracleData")]
    #[case::wg_data("wgData")]
    #[case::ibm_data("ibmData")]
    #[case::ms_data("msData")]
    #[case::saxon_data("saxonData")]
    #[case::sun_data("sunData")]
    #[case::nist_data("nistData")]
    #[case::boeing_data("boeingData")]
    #[case::common("common")]
    fn test_database_validation(
        validator: &XSDValidator,
        test_data: &XsdTestData,
        #[case] data_set: String
    ) {
        for data in test_data.iter() {
            if data.get_data_set() != data_set.as_str() {
                return;
            }
            let file = data.get_path();
            if data.is_valid() {
                test_valid_file(validator, file);
            } else {
                test_invalid_file(validator, file);
            }
        }
    }
}
