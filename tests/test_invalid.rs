use std::io::Write;
use std::process::{Command, Output, Stdio};

#[cfg(test)]
mod tests {
    use std::{fs, path};
    use std::fs::ReadDir;
    use xmlgenerator::generate_xml;

    fn fetch_test_files() -> ReadDir {
        let example_dir = path::absolute("./invalid_examples").unwrap();
        let paths = fs::read_dir(example_dir).unwrap();

        paths
    }

    #[test]
    fn test_xml() {
        let files = fetch_test_files();

        for file in files {
            let filepath = file.unwrap().path();
            println!("{}", filepath.display());
            let xml = generate_xml(filepath.into_boxed_path());

            assert!(xml.is_err());
        }
    }
}
