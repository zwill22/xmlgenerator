#[cfg(test)]
mod tests {
    use reqwest::get;
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;
    use workspace_root::get_workspace_root;
    use xsdvalidator::{XSDValidationError, XSDValidator};

    use futures::executor::block_on;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Error, Write};

    async fn fetch_repo(archive_path: &PathBuf) -> File {
        let url = "https://github.com/w3c/xsdtests/archive/refs/heads/master.zip".to_string();

        let response = get(url).await.expect("failed to send request");
        let content = response.bytes().await.expect("failed to get bytes");

        let mut file = File::create(archive_path).expect("failed to create file");
        file.write_all(&content).expect("failed to write to file");

        file
    }

    fn extract_repo(db_root: &PathBuf, archive: &File) -> Result<(), Box<dyn std::error::Error>> {
        let mut archive = zip::ZipArchive::new(archive)?;

        archive.extract(db_root)?;

        Ok(())
    }

    fn get_archive_file(archive_path: &PathBuf) -> File {
        if !archive_path.exists() {
            let future = fetch_repo(archive_path);
            return block_on(future);
        }

        File::open(archive_path).expect("failed to open file")
    }

    fn check_repo(db_root: &PathBuf, archive_path: &PathBuf) {
        if db_root.exists() {
            return;
        }

        let archive = get_archive_file(archive_path);

        match extract_repo(db_root, &archive) {
            Ok(_) => {}
            Err(e) => panic!("failed to extract archive file: {:?}", e),
        }
    }

    fn check_error(error: &XSDValidationError, path: &Path) {
        match error {
            XSDValidationError::StringError => panic!("Error converting path to string"),
            XSDValidationError::PathError => panic!("Error resolving path: {:?}", path),
            XSDValidationError::ReadFileError => panic!("Error reading file: {:?}", path),
            XSDValidationError::GenerateContextError => {
                panic!("Error generating context for file: {:?}", path)
            }
            XSDValidationError::ParseError(e) => panic!("Parse error: {}, file: {:?}", e, path),
        }
    }

    fn test_file(validator: &XSDValidator, path: &Path) {
        let result = validator.validate(path);

        match result {
            Ok(value) => {
                if !value {
                    panic!("Invalid XSD: {:?}", path);
                }
            }
            Err(e) => check_error(&e, &path),
        }
    }

    fn lines_from_file(filename: impl AsRef<Path>) -> Result<Vec<String>, Error> {
        BufReader::new(File::open(filename)?).lines().collect()
    }

    fn get_file_list_from_file<'a>(filepath: &PathBuf, db_root: &PathBuf) -> Vec<PathBuf> {
        let list = lines_from_file(filepath).expect("failed to read file");

        list.iter().map(|path| db_root.join(path)).collect()
    }

    fn get_file_list<'a>(filepath: &PathBuf, db_root: &PathBuf) -> Vec<PathBuf> {
        if filepath.is_file() {
            return get_file_list_from_file(filepath, db_root);
        }

        let mut result = Vec::new();
        for file in WalkDir::new(filepath) {
            if let Ok(entry) = file {
                let path: PathBuf = entry.into_path();
                if path.ends_with(".xsd") {
                    result.push(path);
                }
            }
        }

        result
    }

    #[test]
    fn test_xsd() {
        let validator = XSDValidator::new();

        let root = get_workspace_root();
        let db_root = root.join("xsdtests-master");
        let archive_path = root.join("xsd_tests.zip");
        let filepath = root.join("valid.txt");

        check_repo(&db_root, &archive_path);

        let file_list = get_file_list(&filepath, &db_root);

        for path in file_list {
            test_file(&validator, &path);
        }
    }
}
