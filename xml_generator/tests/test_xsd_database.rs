#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;
    use workspace_root::get_workspace_root;

    use rstest::*;
    use std::panic;
    use xmlgenerator::{XMLGenerator, XMLGeneratorError};
    use xsdtestdata::XsdTestData;

    fn check_error(error: &XMLGeneratorError) {
        match error {
            XMLGeneratorError::InvalidPathError(e) => panic!("Invalid path: {}", e),
            XMLGeneratorError::XSDValidatorError(e) => panic!("XSD validator error: {}", e),
            XMLGeneratorError::DataTypeInformationError(e) => {
                panic!("Data type information error: {}", e)
            }
            XMLGeneratorError::DataTypeNotFoundError(e) => panic!("DataType not found: {}", e),
            XMLGeneratorError::XSDParserError(e) => {
                eprintln!("XSD Parser Error: {}", e);
            }
            XMLGeneratorError::DataTypesFormatError(e) => {
                eprintln!("Data Types Format Error: {}", e);
            }
            XMLGeneratorError::XMLBuilderError(e) => panic!("XML builder error: {}", e),
            XMLGeneratorError::InvalidXSDVersionError(e) => {
                eprintln!("Invalid XSD Version: {}", e);
            }
            XMLGeneratorError::InfiniteRecursionError => {
                eprintln!("Infinite Recursion detected");
            }
            XMLGeneratorError::NoElementsError => {
                eprintln!("No Elements in XSD");
            }
            XMLGeneratorError::InvalidXSDError(e) => panic!("Invalid XSD error: {}", e),
            XMLGeneratorError::InvalidXSDNameError(e) => {
                eprintln!("Invalid XSD element name: {}", e);
            }
            XMLGeneratorError::NoIndependentElementsError => {
                eprintln!("No Independent Elements found in XSD");
            }
            XMLGeneratorError::MultipleRootsError => {
                eprintln!("Multiple Roots found in XSD");
            }
            XMLGeneratorError::TypeGenerationError(e) => panic!("Type generation error: {}", e),
            XMLGeneratorError::RegexError(e) => panic!("Regex error: {}", e),
            XMLGeneratorError::RegexMismatchError(p1, p2) => {
                eprintln!("Regex mismatch error: {} <-> {}", p1, p2);
            }
            XMLGeneratorError::EncodingError => {
                eprintln!("Encoding error");
            }
            XMLGeneratorError::LineEndingsError(e) => {
                eprintln!("Invalid line ending error: {}", e);
            }
            XMLGeneratorError::UnimplementedFeature(_) => {}
        }
    }

    fn check_result_str(str: &str) {
        if str.is_empty() {
            panic!("Empty string is not allowed");
        }

        //println!("{}", str.replace(">", ">\n").replace("</", "\n</"));
        //println!("{}", str);
    }

    fn test_file(generator: &XMLGenerator, test_data: &XsdTestData, path: &str) {
        let data = match test_data.get(path) {
            Some(data) => data,
            None => return,
        };

        let path = data.get_path();
        match generator.generate(path) {
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
        test_data: &'a XsdTestData,
        data_set: String,
    ) -> HashSet<&'a str> {
        let mut valid_files = HashSet::new();

        test_data.iter().for_each(|data| {
            if data.get_data_set() != data_set {
                return;
            }
            let valid = data.is_valid();
            if valid {
                if validate_file(generator, data.get_path(), valid) {
                    valid_files.insert(data.get_key());
                }
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
    fn test_data() -> XsdTestData {
        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let archive = root.join("xsdtests.zip");

        XsdTestData::new(&db_root, &archive)
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
        test_data: &XsdTestData,
        #[case] data_set: String,
    ) {
        for filepath in get_valid_files(generator, test_data, data_set) {
            test_file(generator, test_data, filepath);
        }
    }

    #[rstest]
    #[case("saxonData/XmlVersions/xv008.xsd")]
    fn test_files(generator: &XMLGenerator, test_data: &XsdTestData, #[case] file: &str) {
        println!("{}", file);

        test_file(generator, test_data, file);
    }
}
