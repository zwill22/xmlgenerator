#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::io::{BufReader, BufWriter, prelude::*};
    use std::path::PathBuf;
    use std::{fs, panic};
    use workspace_root::get_workspace_root;
    use xmlgenerator::error::XMLGeneratorError;
    use xmlgenerator::generate_xml;

    fn get_file_path(path: &str) -> PathBuf {
        let root: PathBuf = get_workspace_root();

        root.join(path)
    }

    fn read_file(path: &PathBuf) -> String {
        fs::read_to_string(path).unwrap_or_else(|_| "".to_string())
    }

    fn process_error(error: &XMLGeneratorError) {
        match error {
            XMLGeneratorError::DataTypeError(e) => println!("Data Type Error: {}", e),
            XMLGeneratorError::XSDParserError(e) => println!("XSD Parser Error: {}", e),
            XMLGeneratorError::DataTypesFormatError(e) => {
                println!("XSD DataTypes Format Error: {}", e)
            }
            XMLGeneratorError::XMLBuilderError(e) => println!("XML Builder Error: {}", e),
        }
    }

    fn check_result<T>(
        writer: &mut BufWriter<T>,
        result: &Result<String, XMLGeneratorError>,
        filepath: &PathBuf,
    ) where
        T: Write,
    {
        match result {
            Ok(_) => {
                let out = writeln!(writer, "{}", filepath.to_str().unwrap());
                if let Err(e) = out {
                    println!("Error writing file: {}", e);
                }
            }
            Err(error) => process_error(error),
        }
    }

    fn check_error_string(string: &String) {
        if string.is_empty() {
            panic!("Unknown error");
        }

        if string.contains("not implemented") {
            println!("Implementation error: {}", string);
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

    fn check_invalid_error(error: &XMLGeneratorError) {
        match error {
            XMLGeneratorError::DataTypeError(_) => {}
            XMLGeneratorError::XSDParserError(_) => {}
            XMLGeneratorError::DataTypesFormatError(_) => {}
            XMLGeneratorError::XMLBuilderError(_) => panic!("XML builder called on invalid input"),
        }
    }

    fn check_invalid_result(result: &Result<String, XMLGeneratorError>) {
        match result {
            Ok(_) => panic!("Invalid input, result should return an error"),
            Err(e) => check_invalid_error(e),
        }
    }

    fn test_invalid_xml(filepath: &PathBuf) {
        let contents = read_file(filepath);
        if contents.is_empty() {
            return;
        }

        match panic::catch_unwind(|| generate_xml(&contents)) {
            Ok(result) => check_invalid_result(&result),
            Err(error) => check_panic(error),
        }
    }

    fn test_xml<T: Write>(writer: &mut BufWriter<T>, filepath: &PathBuf) {
        let contents = read_file(filepath);
        if contents.is_empty() {
            return;
        }

        let result = panic::catch_unwind(|| generate_xml(&contents));

        match result {
            Ok(xml) => check_result(writer, &xml, filepath),
            Err(error) => check_panic(error),
        }
    }

    fn test_files(output_writer: &mut BufWriter<fs::File>, filename: &PathBuf) {
        let root = get_workspace_root();
        let file = fs::File::open(filename).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            let path = root.join("xsdtests-master").join(&line);

            if line.contains("invalid") {
                test_invalid_xml(&path);
            } else {
                test_xml(output_writer, &path);
            }
        }
    }

    #[test]
    fn test_xsd() {
        let valid_xsd = get_file_path("valid_xsd.txt");

        let output_file = fs::File::create(valid_xsd).expect("Unable to create file");
        let mut output_writer = BufWriter::new(output_file);

        let xsd_list = get_file_path("xsd_list.txt");
        if xsd_list.exists() {
            test_files(&mut output_writer, &xsd_list);
        } else {
            panic!("File does not exist");
        }
    }
}
