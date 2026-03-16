#[cfg(test)]
mod tests {
    use crate::{RegexTranslationError, RegexTranslator};
    use file_to_string::read_file;
    use roxmltree::Node;
    use roxmltree::{Attribute, Document};
    use std::collections::HashSet;

    use std::path::PathBuf;
    use rstest::{fixture, rstest};
    use workspace_root::get_workspace_root;
    use xsdtestdata::get_test_data;

    fn parse_attribute(regex: &mut HashSet<String>, attribute: &Attribute) {
        if attribute.name().trim().to_lowercase() == "value" {
            regex.insert(attribute.value().to_string());
        }
    }

    fn parse_node(regex: &mut HashSet<String>, node: Node) {
        let name = node.tag_name().name().trim().to_string();

        if name == "pattern" {
            node.attributes().for_each(|attr| parse_attribute(regex, &attr));
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

    fn get_regex_patterns(root: &PathBuf, archive: &PathBuf) -> HashSet<String> {
        let mut regex = HashSet::new();

        let test_data = get_test_data(root, archive, true);

        for (filepath, listed_as_valid) in test_data {
            if listed_as_valid {
                parse_file(&mut regex, &filepath);
            }
        }

        regex
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

    #[fixture]
    #[once]
    fn translator() -> RegexTranslator {
        RegexTranslator::new().expect("Unable to initialise RegexTranslator")
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

    #[rstest]
    fn it_works(
        translator: &RegexTranslator,
    ) {
        let root = get_workspace_root();
        let db_root = root.as_path().join("xsdtests-master");
        let archive = root.as_path().join("xsdtests.zip");

        let patterns = get_regex_patterns(&db_root, &archive);

        patterns.iter().for_each(|pattern| test_pattern(translator, pattern));
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

        surrogates_strings.iter().for_each(|string| handle_surrogate_string(translator, string));
    }
}
