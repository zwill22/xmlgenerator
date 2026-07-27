use file_to_string::read_file;
use reqwest::blocking;
use roxmltree::{ Document, Node, ParsingOptions };
use std::collections::HashSet;
use std::fs::{ File, canonicalize };
use std::io::Write;
use std::ops::AddAssign;
use std::path::{ Path, PathBuf };
use zip::read::root_dir_common_filter;

#[derive(Debug)]
pub enum XSDTestDataError {
    ParseError(String),
    ArchiveError,
    ArchiveExtractionError,
    InvalidFileError(String),
    FileReadError(String),
}

#[derive(Clone)]
pub struct XsdData {
    key: String,
    data_set: String,
    path: PathBuf,
    valid: bool,
}

impl XsdData {
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_data_set(&self) -> &str {
        &self.data_set
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    fn is_set(&self, name: &str) -> bool {
        self.get_data_set() == name
    }
}

fn fetch_repo(archive_path: &PathBuf) {
    let url = "https://github.com/w3c/xsdtests/archive/refs/heads/master.zip".to_string();

    let response = blocking::get(url).expect("failed to send request");
    let content = response.bytes().expect("failed to get bytes");

    let mut file = File::create(archive_path).expect("failed to create file");
    file.write_all(&content).expect("failed to write to file");
}

fn extract_repo(db_root: &PathBuf, archive: &File) -> Result<(), XSDTestDataError> {
    let mut archive = match zip::ZipArchive::new(archive) {
        Ok(archive) => archive,
        Err(_) => {
            return Err(XSDTestDataError::ArchiveError);
        }
    };

    match archive.extract_unwrapped_root_dir(db_root, root_dir_common_filter) {
        Ok(_) => Ok(()),
        Err(_) => Err(XSDTestDataError::ArchiveExtractionError),
    }
}

fn get_archive_file(archive_path: &PathBuf) -> File {
    if !archive_path.exists() {
        fetch_repo(archive_path);
    }

    File::open(archive_path).expect("failed to open file")
}

fn check_repo(db_root: &PathBuf, archive_path: &PathBuf) {
    if db_root.exists() {
        return;
    }

    let archive = get_archive_file(archive_path);

    extract_repo(db_root, &archive).expect("failed to extract repo archive");
}

fn parse_with_dtd(contents: &str) -> Result<Document<'_>, XSDTestDataError> {
    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };

    match Document::parse_with_options(contents, options) {
        Ok(document) => Ok(document),
        Err(e) => Err(XSDTestDataError::ParseError(e.to_string())),
    }
}

fn parse(contents: &str) -> Result<Document<'_>, XSDTestDataError> {
    match Document::parse(contents) {
        Ok(document) => Ok(document),
        Err(_) => parse_with_dtd(contents),
    }
}

fn get_href_path(node: &Node, path: &Path) -> Result<PathBuf, XSDTestDataError> {
    if node.has_children() {
        panic!("Base node has children");
    }

    let xsd_path = get_attribute(node, "href".to_string());

    let full_path = path.join(xsd_path);
    match canonicalize(full_path) {
        Ok(path) => Ok(path),
        Err(e) => Err(XSDTestDataError::InvalidFileError(e.to_string())),
    }
}

fn get_validity(node: &Node) -> Option<bool> {
    if node.has_children() {
        panic!("Validity node must have children");
    }

    let valid_str = get_attribute(node, "validity".to_string());
    match valid_str.as_str() {
        "valid" => Some(true),
        "invalid" => Some(false),
        "indeterminate" => None,
        _ => panic!("Unknown validity:: {}", valid_str),
    }
}

fn get_key_and_group(root: &PathBuf, full_path: &Path) -> (String, String) {
    let path = full_path.strip_prefix(root).unwrap();

    let mut group = "other".to_string();
    if let Some(component) = path.components().next() {
        group = component.as_os_str().to_str().unwrap().to_string();
    }

    (path.to_string_lossy().to_string(), group)
}

fn get_test_info(
    node: &Node,
    filepath: &PathBuf,
    tag_name: String,
    db_root: &PathBuf
) -> Option<XsdData> {
    let mut schema_path = None;
    let mut valid = None;

    let path = filepath.parent().unwrap();

    for child in node.children() {
        let tag = child.tag_name().name();
        if tag == tag_name {
            let schema = match get_href_path(&child, path) {
                Ok(path) => path,
                Err(_) => {
                    return None;
                }
            };
            if schema == *filepath {
                return None;
            }
            schema_path = Some(schema);
        } else if tag == "expected" {
            let validity = get_validity(&child);
            validity?;
            valid = validity;
        }
    }

    if schema_path.is_none() || valid.is_none() {
        return None;
    }

    let filepath = schema_path.unwrap();
    let validity = valid.unwrap();

    let (path, group) = get_key_and_group(db_root, &filepath);

    let data = XsdData {
        key: path,
        data_set: group,
        path: filepath,
        valid: validity,
    };

    Some(data)
}

fn get_schema_info(node: &Node, path: &PathBuf, db_root: &PathBuf) -> Option<XsdData> {
    get_test_info(node, path, "schemaDocument".to_string(), db_root)
}

fn get_schema_test(
    results: &mut XsdTestData,
    node: &Node<'_, '_>,
    path: &PathBuf,
    db_root: &PathBuf
) {
    if let Some(info) = get_schema_info(node, path, db_root) {
        results.push(info)
    }
}

fn get_instance_info(node: &Node, path: &PathBuf, db_root: &PathBuf) -> Option<XsdData> {
    get_test_info(node, path, "instanceDocument".to_string(), db_root)
}

fn get_test_group(test_group: &Node, path: &PathBuf, db_root: &PathBuf) -> XsdTestData {
    let mut data = XsdTestData::default();
    test_group.children().for_each(|child| {
        let tag_name = child.tag_name().name();

        if tag_name == "schemaTest" {
            get_schema_test(&mut data, &child, path, db_root);
        } else if
            tag_name == "instanceTest" &&
            let Some(new_data) = get_instance_test(&child, path, db_root)
        {
            data += &new_data;
        }
    });

    data
}

fn read_test_set_file(filepath: &PathBuf, db_root: &PathBuf) -> XsdTestData {
    let filedata = read_file(filepath).expect("failed to read file");
    let mut data = XsdTestData::default();

    let document = match parse(&filedata) {
        Ok(d) => d,
        Err(_) => {
            return data;
        }
    };

    let root = document.root_element();
    root.children().for_each(|child| {
        let tag_name = child.tag_name().name();

        if tag_name == "testGroup" {
            data += &get_test_group(&child, filepath, db_root);
        }
    });

    data
}

fn get_instance_test(node: &Node, path: &PathBuf, db_root: &PathBuf) -> Option<XsdTestData> {
    let instance = get_instance_info(node, path, db_root)?;

    if instance.path == *path {
        return None;
    }

    if instance.valid {
        let data = read_test_set_file(&instance.path, db_root);
        return Some(data);
    }

    None
}

fn get_attribute(node: &Node, name: String) -> String {
    for attribute in node.attributes() {
        if attribute.name() == name {
            return attribute.value().to_string();
        }
    }

    panic!("attribute not found");
}

fn file_path(db_root: &Path, path_string: &str) -> PathBuf {
    db_root.join(path_string)
}

#[derive(Default, Clone)]
pub struct XsdTestData {
    data: Vec<XsdData>,
}

impl XsdTestData {
    fn total(&self) -> usize {
        self.data.len()
    }

    fn contains(&self, data: &XsdData) -> bool {
        let key = &data.key;
        self.data.iter().any(|d| d.key == *key)
    }

    fn push(&mut self, data: XsdData) {
        if self.contains(&data) {
            return;
        }

        self.data.push(data);
    }
}

impl AddAssign<&XsdTestData> for XsdTestData {
    fn add_assign(&mut self, rhs: &XsdTestData) {
        for data in rhs.data.iter() {
            self.push(data.clone());
        }
    }
}

impl XsdTestData {
    fn add(&mut self, results: &XsdTestData, ignore: &[PathBuf]) {
        results.data.iter().for_each(|result| {
            if ignore.contains(&result.path) {
                return;
            }

            self.push(result.clone())
        })
    }

    fn add_test_set(root_path: &PathBuf, extension: &String) -> Self {
        let filepath = root_path.join(extension);

        read_test_set_file(&filepath, root_path)
    }

    fn parse_test_data(test_suite: &PathBuf, db_path: &PathBuf, ignore: &[PathBuf]) -> XsdTestData {
        let filedata = read_file(test_suite).expect("failed to read file");
        let document = parse(&filedata).expect("failed to parse xml");

        let mut output = XsdTestData::default();
        let root = document.root_element();
        root.children().for_each(|child| {
            let tag = child.tag_name().name();
            if tag == "testSetRef" {
                let test_path = get_attribute(&child, "href".to_string());
                let result = Self::add_test_set(db_path, &test_path);
                output.add(&result, ignore);
            }
        });

        output
    }

    pub fn new(db_path: &PathBuf, archive_path: &PathBuf) -> XsdTestData {
        // TODO Remove ignores
        let ignore = vec![
            file_path(db_path, "msData/particles/particlesZ012.xsd"),
            file_path(db_path, "msData/particles/particlesZ015.xsd"),
            file_path(db_path, "msData/particles/particlesZ020.xsd"),
            file_path(db_path, "msData/regex/reG17.xsd"),
            file_path(db_path, "msData/regex/reJ25.xsd"),
            file_path(db_path, "saxonData/XmlVersions/xv009.xsd")
        ];
        check_repo(db_path, archive_path);

        let suite = db_path.join("suite.xml");
        let mut data = XsdTestData::parse_test_data(&suite, db_path, &ignore);

        let extra_suite = db_path.join("extra-suite.xml");
        let extra_data = XsdTestData::parse_test_data(&extra_suite, db_path, &ignore);

        data += &extra_data;

        data
    }

    pub fn print_stats(&self) {
        let total = self.total();
        let valid = self.data
            .iter()
            .filter(|data| data.valid)
            .count();
        let invalid = total - valid;

        println!("Test data\n");

        println!("\t{:24}{:8}", "Valid schemas", valid);
        println!("\t{:24}{:8}", "Invalid schemas", invalid);
        println!("\t{:24}{:8}", "Total schemas", total);

        let mut data_sets = HashSet::new();
        self.data.iter().for_each(|data| {
            let data_set = &data.data_set;
            data_sets.insert(data_set);
        });
        println!("\t{:24}{:8}", "Data sets", data_sets.len());

        println!();

        println!("\t{:36}{:8}{:8}{:8}", "Data set", "Valid", "Invalid", "Total");

        for data_set in data_sets {
            let valid = self.data
                .iter()
                .filter(|data| data.is_set(data_set) && data.valid)
                .count();

            let invalid = self.data
                .iter()
                .filter(|data| data.is_set(data_set) && !data.valid)
                .count();

            println!("\t{:36}{:8}{:8}{:8}", data_set, valid, invalid, valid + invalid);
        }
    }

    fn get_index(&self, index: usize) -> Option<&XsdData> {
        if index < self.total() { Some(&self.data[index]) } else { None }
    }

    pub fn get(&self, key: &str) -> Option<&XsdData> {
        self.data.iter().find(|data| data.key == key)
    }
}

pub struct XsdTestDataIterator<'a> {
    test_data: &'a XsdTestData,
    index: usize,
}

impl<'a> Iterator for XsdTestDataIterator<'a> {
    type Item = &'a XsdData;
    fn next(&mut self) -> Option<Self::Item> {
        match self.test_data.get_index(self.index) {
            Some(data) => {
                self.index += 1;
                Some(data)
            }
            None => None,
        }
    }
}

impl XsdTestData {
    pub fn iter(&'_ self) -> XsdTestDataIterator<'_> {
        XsdTestDataIterator {
            test_data: self,
            index: 0,
        }
    }
}

pub struct XsdTestDataIntoIterator {
    test_data: XsdTestData,
}

impl Iterator for XsdTestDataIntoIterator {
    type Item = XsdData;

    fn next(&mut self) -> Option<Self::Item> {
        if self.test_data.data.is_empty() {
            return None;
        }

        let result = self.test_data.data.remove(0);

        Some(result)
    }
}

impl IntoIterator for XsdTestData {
    type Item = XsdData;
    type IntoIter = XsdTestDataIntoIterator;

    fn into_iter(self) -> XsdTestDataIntoIterator {
        XsdTestDataIntoIterator { test_data: self }
    }
}

#[cfg(test)]
mod tests {
    use crate::XsdTestData;
    use workspace_root::get_workspace_root;

    #[test]
    fn get_suite() {
        let db = get_workspace_root().join("xsdtests-master");
        let archive = get_workspace_root().join("xsdtests.zip");

        let data = XsdTestData::new(&db, &archive);

        data.print_stats()
    }
}
