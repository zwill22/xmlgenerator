#[cfg(test)]
mod tests {
    use crate::{RegexTranslationError, RegexTranslator};
    use file_to_string::read_file;
    use roxmltree::Node;
    use roxmltree::{Attribute, Document};
    use std::collections::HashSet;

    use rstest::{fixture, rstest};
    use std::path::PathBuf;
    use workspace_root::get_workspace_root;
    use xsdtestdata::XsdTestData;

    fn parse_attribute(regex: &mut HashSet<String>, attribute: &Attribute) {
        if attribute.name().trim().to_lowercase() == "value" {
            regex.insert(attribute.value().to_string());
        }
    }

    fn parse_node(regex: &mut HashSet<String>, node: Node) {
        let name = node.tag_name().name().trim().to_string();

        if name == "pattern" {
            node.attributes()
                .for_each(|attr| parse_attribute(regex, &attr));
        }

        node.children().for_each(|child| parse_node(regex, child));
    }

    fn parse_file(regex: &mut HashSet<String>, filepath: &PathBuf) {
        let xml_string = match read_file(&filepath) {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        };

        let xml_tree = Document::parse(xml_string.as_str()).expect("Error parsing XML");

        let root = xml_tree.root();
        parse_node(regex, root);
    }


    fn handle_errors(error: RegexTranslationError, input: &str) {
        match error {
            RegexTranslationError::InvalidInput(e) => println!("{}", e),
            RegexTranslationError::RegexError(e) => panic!("{}", e),
            RegexTranslationError::FileReadError(e) => panic!("{}", e),
            RegexTranslationError::DataError(e) => panic!("{}", e),
            RegexTranslationError::UnicodeError(e) => panic!("{}", e),
            RegexTranslationError::SurrogatesError => println!("Surrogates in input: {}", input),
        }
    }

    fn test_pattern(translator: &RegexTranslator, pattern: &str) {
        match translator.translate(&pattern, true) {
            Ok(_) => return,
            Err(_) => match translator.translate(&pattern, false) {
                Ok(_) => return,
                Err(e) => handle_errors(e, &pattern),
            },
        }
    }

    fn patterns(test_data: &XsdTestData, data_set: &str) -> HashSet<String> {
        let mut values = HashSet::new();
        test_data.iter().for_each(|test_data| {
            if test_data.get_data_set() != data_set {
                return;
            }

            parse_file(&mut values, test_data.get_path());
        });

        values
    }

    #[fixture]
    #[once]
    fn translator() -> RegexTranslator {
        RegexTranslator::new().expect("Unable to initialise RegexTranslator")
    }

    #[fixture]
    #[once]
    fn test_data() -> XsdTestData {
        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let archive = root.as_path().join("xsdtests.zip");

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
    fn it_works(translator: &RegexTranslator, test_data: &XsdTestData, #[case] data_set: String) {
        patterns(test_data, &data_set).iter()
            .for_each(|pattern| test_pattern(translator, pattern));
    }

    fn handle_surrogate_string(translator: &RegexTranslator, surrogate: &str) {
        match translator.translate(surrogate, true) {
            Ok(_) => panic!("No error thrown"),
            Err(e) => match e {
                RegexTranslationError::SurrogatesError => {}
                _ => panic!("Invalid error thrown"),
            },
        }
    }

    #[rstest]
    fn check_surrogates(translator: &RegexTranslator) {
        let surrogates_strings = vec![
            r"\p{IsHighSurrogates}",
            r"\p{IsHighPrivateUseSurrogates}",
            r"\p{IsLowSurrogates}",
            r"\P{IsHighSurrogates}",
            r"\P{IsHighPrivateUseSurrogates}",
            r"\P{IsLowSurrogates}",
        ];

        surrogates_strings
            .iter()
            .for_each(|string| handle_surrogate_string(translator, string));
    }
}
