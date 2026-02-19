#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use workspace_root::get_workspace_root;
    use xsdtestdata::get_test_data;

    use gag::Gag;
    use std::panic;
    use xmlgenerator::{XMLGenerator, XMLGeneratorError};

    fn increment(stats: &mut HashMap<String, Stat>, name: &str, value: &str) {
        match stats.get_mut(name) {
            None => {
                let mut stat = Stat::new();
                stat.add(value);

                stats.insert(name.to_string(), stat);
            }
            Some(stat) => stat.add(value),
        }
    }

    fn check_error(error: &XMLGeneratorError, stats: &mut HashMap<String, Stat>) {
        match error {
            XMLGeneratorError::InvalidPathError(e) => panic!("Invalid path: {}", e),
            XMLGeneratorError::XSDValidatorError(e) => panic!("XSD validator error: {}", e),
            XMLGeneratorError::DataTypeInformationError(e) => {
                panic!("Data type information error: {}", e)
            }
            XMLGeneratorError::DataTypeNotFoundError(e) => panic!("DataType not found: {}", e),
            XMLGeneratorError::XSDParserError(e) => {
                increment(stats, "XSD Parser Errors", e);
            }
            XMLGeneratorError::DataTypesFormatError(e) => {
                increment(stats, "Data Types Format Error", e);
            }
            XMLGeneratorError::XMLBuilderError(e) => panic!("XML builder error: {}", e),
            XMLGeneratorError::InvalidXSDVersionError(e) => {
                increment(stats, "Invalid XSD Version", e);
            }
            XMLGeneratorError::InfiniteRecursionError => {
                increment(stats, "Infinite Recursion", "");
            }
            XMLGeneratorError::NoElementsError => {
                increment(stats, "No Elements", "");
            }
            XMLGeneratorError::InvalidXSDError(e) => panic!("Invalid XSD error: {}", e),
            XMLGeneratorError::InvalidXSDNameError(e) => {
                increment(stats, "Invalid XSD element name", e);
            }
            XMLGeneratorError::NoIndependentElementsError => {
                increment(stats, "No Independent Elements", "");
            }
            XMLGeneratorError::MultipleRootsError => {
                increment(stats, "Multiple Roots", "");
            }
            XMLGeneratorError::TypeGenerationError(e) => panic!("Type generation error: {}", e),
            XMLGeneratorError::RegexError(e) => panic!("Regex error: {}", e),
            XMLGeneratorError::LineEndingsError(e) => {
                increment(stats, "Invalid line ending", e);
            }
            XMLGeneratorError::UnimplementedFeature(e) => {
                increment(stats, "Unimplemented Features", e);
            }
        }
    }

    fn check_result_str(str: &String) {
        if str.is_empty() {
            panic!("Empty string is not allowed");
        }
    }

    fn run_generator(
        generator: &XMLGenerator,
        path: &PathBuf,
    ) -> Result<String, XMLGeneratorError> {
        let _err_gag = Gag::stderr().unwrap();

        generator.generate(path)
    }

    fn test_file(generator: &XMLGenerator, path: &PathBuf, stats: &mut HashMap<String, Stat>) {
        match run_generator(generator, path) {
            Ok(str) => check_result_str(&str),
            Err(err) => check_error(&err, stats),
        }
    }

    fn validate(generator: &XMLGenerator, filepath: &PathBuf) -> Result<(), XMLGeneratorError> {
        let _err_gag = Gag::stderr().unwrap();

        generator.validate(filepath)
    }

    fn validate_file(generator: &XMLGenerator, filepath: &PathBuf, valid: bool) -> bool {
        if !valid {
            return false;
        }

        validate(generator, filepath).is_ok()
    }

    fn get_valid_files(
        generator: &XMLGenerator,
        root: &PathBuf,
        archive: &PathBuf,
    ) -> HashSet<PathBuf> {
        let mut valid_files = HashSet::new();

        let test_data = get_test_data(root, archive, false);

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

    #[derive(Default)]
    struct Stat {
        count: usize,
        types: HashSet<String>,
    }

    impl Stat {
        fn new() -> Self {
            Default::default()
        }

        fn add(&mut self, s: &str) {
            self.count += 1;
            self.types.insert(s.to_string());
        }
    }

    // TODO Invalid XSD input error should be caught by XSDValidator
    fn print_stats(stats: &HashMap<String, Stat>) {
        println!();
        println!("{:<32} \tTotal\tUnique", "Error type");
        println!("------------------------------------------------------");
        let mut total = 0;
        let mut unique = 0;
        for (name, val) in stats.iter() {
            let u = val.types.len();

            total += val.count;
            unique += u;
            println!("{:<32}:\t{:5}\t{:6}", name, val.count, u);
        }
        println!("======================================================");
        println!("{:<32} \t{:5}\t{:6}", "Total", total, unique);
        println!();
    }

    #[test]
    fn test_xsd_db() {
        let generator = XMLGenerator::new();

        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let archive = root.join("xsdtests.zip");

        let valid_files = get_valid_files(&generator, &db_root, &archive);

        let mut stats = HashMap::new();

        for path in valid_files {
            test_file(&generator, &path, &mut stats);
        }

        if stats.is_empty() {
            return;
        }

        print_stats(&stats);
    }

    #[allow(unused)]
    fn test_single_file(filepath: &str) {
        // TODO Change into CLI
        let generator = XMLGenerator::new();

        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let path = db_root.join(filepath);

        let mut stats = HashMap::new();
        test_file(&generator, &path, &mut stats);
        if !stats.is_empty() {
            for (name, stat) in stats.iter() {
                eprintln!("Error:\t{}", name);
                for error_type in &stat.types {
                    if error_type.is_empty() {
                        continue;
                    }
                    eprintln!("Type:\t{}", error_type);
                }
            }

            eprintln!("File:\t{}", path.display());
        }
    }
}
