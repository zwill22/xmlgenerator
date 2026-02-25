#[cfg(test)]
mod tests {
    use crate::{RegexTranslationError, RegexTranslator};
    use file_to_string::read_file;
    use roxmltree::Node;
    use roxmltree::{Attribute, Document};
    use std::collections::HashSet;

    use std::path::PathBuf;
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
            for attribute in node.attributes() {
                parse_attribute(regex, &attribute);
            }
        }

        for child in node.children() {
            parse_node(regex, child);
        }
    }

    fn parse_file(regex: &mut HashSet<String>, filepath: &PathBuf) {
        let xml_string = match read_file(&filepath) {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        };

        let xml_tree = match Document::parse(xml_string.as_str()) {
            Ok(t) => t,
            Err(e) => panic!("{}", e),
        };

        let root = xml_tree.root();
        parse_node(regex, root);
    }

    fn get_regex_strings(root: &PathBuf, archive: &PathBuf) -> HashSet<String> {
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
            RegexTranslationError::SurrogatesError => println!("Surrogates in input: {}", input),
        }
    }

    #[test]
    fn it_works() {
        let root = get_workspace_root();
        let db_root = root.as_path().join("xsdtests-master");
        let archive = root.as_path().join("xsdtests.zip");

        let regex = get_regex_strings(&db_root, &archive);
        let translator = match RegexTranslator::new() {
            Ok(t) => t,
            Err(e) => panic!("Unable to initialise RegexTranslator: {}", e),
        };

        for input_regex in regex {
            match translator.translate(&input_regex, true) {
                Ok(_) => {}
                Err(_) => match translator.translate(&input_regex, false) {
                    Ok(_) => {}
                    Err(e) => handle_errors(e, &input_regex),
                },
            }
        }
    }

    #[test]
    fn check_surrogates() {
        let translator = RegexTranslator::new().unwrap();

        let surrogates_strings = vec![
            r"\p{IsHighSurrogates}",
            r"\p{IsHighPrivateUseSurrogates}",
            r"\p{IsLowSurrogates}",
            r"\P{IsHighSurrogates}",
            r"\P{IsHighPrivateUseSurrogates}",
            r"\P{IsLowSurrogates}",
        ];

        for string in surrogates_strings {
            match translator.translate(string, true) {
                Ok(_) => panic!("No error thrown"),
                Err(e) => match e {
                    RegexTranslationError::SurrogatesError => {}
                    _ => panic!("Invalid error thrown"),
                },
            }
        }
    }
}
