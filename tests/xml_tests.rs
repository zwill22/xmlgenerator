#[cfg(test)]
mod tests {
    use std::{fs, path};
    use std::fs::ReadDir;
    use xmlgenerator::generate_xml;

    fn fetch_test_files() -> ReadDir {
        let example_dir = path::absolute("./examples").unwrap();
        let paths = fs::read_dir(example_dir).unwrap();

        paths
    }

    fn check_result(result: String) {
        println!("{}", result);
    }

    #[test]
    fn test_xml() {
        let files = fetch_test_files();

        for file in files {
            let filepath = file.unwrap().path();
            println!("{}", filepath.display());
            let xml = generate_xml(filepath.into_boxed_path());

            match xml {
                Ok(result) => check_result(result),
                Err(err) => panic!("{:?}", err),
            }
        }
    }
}
