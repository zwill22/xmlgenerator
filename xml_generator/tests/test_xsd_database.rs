#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;
    use workspace_root::get_workspace_root;

    use rstest::*;
    use std::panic;
    use xmlgenerator::{ XMLGenerator, XMLGeneratorError };
    use xsdtestdata::XSDTestData;

    fn check_error(error: &XMLGeneratorError) -> String {
        match error {
            XMLGeneratorError::InvalidPathError(e) => panic!("Invalid path: {}", e),
            XMLGeneratorError::XSDValidatorError(e) => panic!("XSD validator error: {}", e),
            XMLGeneratorError::DataTypeInformationError(e) => {
                panic!("Data type information error: {}", e)
            }
            XMLGeneratorError::DataTypeNotFoundError(e) => panic!("DataType not found: {}", e),
            XMLGeneratorError::XSDParserError(e) => { format!("XSD Parser Error: {}", e) }
            XMLGeneratorError::DataTypesFormatError(e) => {
                format!("Data Types Format Error: {}", e)
            }
            XMLGeneratorError::XMLBuilderError(e) => panic!("XML builder error: {}", e),
            XMLGeneratorError::InvalidXSDVersionError(e) => {
                format!("Invalid XSD Version: {}", e)
            }
            XMLGeneratorError::InfiniteRecursionError => "Infinite Recursion detected".to_string(),
            XMLGeneratorError::NoElementsError => "No Elements in XSD".to_string(),
            XMLGeneratorError::InvalidXSDError(e) => panic!("Invalid XSD error: {}", e),
            XMLGeneratorError::InvalidXSDNameError(e) => {
                format!("Invalid XSD element name: {}", e)
            }
            XMLGeneratorError::NoIndependentElementsError => {
                "No Independent Elements found in XSD".to_string()
            }
            XMLGeneratorError::MultipleRootsError => "Multiple Roots found in XSD".to_string(),
            XMLGeneratorError::TypeGenerationError(e) => panic!("Type generation error: {}", e),
            XMLGeneratorError::RegexError(e) => panic!("Regex error: {}", e),
            XMLGeneratorError::RegexMismatchError(p1, p2) => {
                format!("Regex mismatch error: {} <-> {}", p1, p2)
            }
            XMLGeneratorError::EncodingError => "Encoding error".to_string(),
            XMLGeneratorError::LineEndingsError(e) => {
                format!("Invalid line ending error: {}", e)
            }
            XMLGeneratorError::UnimplementedFeature(e) => {
                format!("Unimplemented feature: {}", e)
            }
        }
    }

    fn check_result_str(str: &str) -> String {
        if str.is_empty() {
            panic!("Empty string is not allowed");
        }

        str.to_string()
    }

    fn test_file(generator: &XMLGenerator, test_data: &XSDTestData, path: &str) -> String {
        let data = match test_data.get(path) {
            Some(data) => data,
            None => {
                return "".to_string();
            }
        };

        let path = data.get_path();
        match generator.generate(path, Some(57)) {
            Ok(str) => check_result_str(&str),
            Err(err) => check_error(&err),
        }
    }

    fn validate_file(generator: &XMLGenerator, filepath: &PathBuf, valid: bool) -> bool {
        if !valid {
            return false;
        }

        generator.validate(filepath).is_ok()
    }

    fn get_valid_files<'a>(
        generator: &XMLGenerator,
        test_data: &'a XSDTestData,
        data_set: String
    ) -> HashSet<&'a str> {
        let mut valid_files = HashSet::new();

        test_data.iter().for_each(|data| {
            if data.get_data_set() != data_set {
                return;
            }
            let valid = data.is_valid();
            if valid && validate_file(generator, data.get_path(), valid) {
                valid_files.insert(data.get_key());
            }
        });

        valid_files
    }

    #[fixture]
    #[once]
    fn generator() -> XMLGenerator {
        XMLGenerator::new()
    }

    #[fixture]
    #[once]
    fn test_data() -> XSDTestData {
        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let archive = root.join("xsdtests.zip");

        XSDTestData::new(&db_root, &archive)
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
    fn test_xsd_database(
        generator: &XMLGenerator,
        test_data: &XSDTestData,
        #[case] data_set: String
    ) {
        for filepath in get_valid_files(generator, test_data, data_set) {
            test_file(generator, test_data, filepath);
        }
    }

    // #[rstest]
    // fn test_single_file(generator: &XMLGenerator, test_data: &XSDTestData) {
    //     let file = "saxonData/XmlVersions/xv008.xsd";
    //     println!("{}", file);
    //
    //     let output = test_file(generator, test_data, file);
    //     println!("{}", output);
    // }
}
