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
        let path_str = path.to_str().unwrap();
        let message = format!("Could not read file: {}", path_str);
        fs::read_to_string(path).expect(&message)
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
    ) where T: Write {
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

    fn check_panic(error: Box<dyn Any>) {
        if let Some(s) = error.downcast_ref::<&str>() {
            println!("Implementation error: {}", s);
        } else if let Some(s) = error.downcast_ref::<String>() {
            println!("panic error: {}", s);
        } else {
            println!("unknown error");
        }
    }

    fn test_xml<T: Write>(writer: &mut BufWriter<T>, filepath: &PathBuf) {
        let contents = read_file(filepath);
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
            println!("{}", path.display());
            test_xml(output_writer, &path);
            println!("Finished: {}", path.display());
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
