//! Crate that manages test data from the [xsdtests] database
//! 
//! [xsdtests]: https://github.com/w3c/xsdtests
//! 
//! The crate provides the following structs:
//!  
//! - [XSDTestData](struct@XSDTestData) - For fetching and managing the test data
//! - [XSDData](struct@XSDData) - For storing data on a particular test case (file)
//! 
//! # Examples
//! 
//! Obtaining test data:
//! 
//! ```rust
//! use xsdtestdata::XSDTestData;
//! 
//! fn fetch_xsd_test_data(root_directory: PathBuf) {
//!     let db_path = root_dir.join("xsdtests-master");
//!     let archive_path = root_dir.join("xsd_tests.zip");
//!     
//!     // Checks whether test data already exists either
//!     // already extracted at `db_path` or archived at `xsd_tests.zip`.
//!     // If neither of these locations already exist, then it
//!     // downloads `xsd_tests.zip`
//!     XSDTestData::new(&db_root, &archive_path)
//! }
//! ```
//! 
//! Running tests:
//! 
//! ```rust
//! use xsdtestdata::XSDTestData;
//! 
//! fn run_tests(xsd_test_data: &XSDTestData) {
//!     xsd_test_data.iter().for_each(|case| {
//!         let valid = case.is_valid();
//!         let file = case.get_path();
//! 
//!         if valid {
//!             assert!(file.exists());
//!         }
//!     });
//! }
//! ```
//! 

use file_to_string::read_file;
use reqwest::blocking;
use roxmltree::{Document, Node, ParsingOptions};
use std::collections::HashSet;
use std::fs::{File, canonicalize};
use std::io::Write;
use std::ops::AddAssign;
use std::path::{Path, PathBuf};
use zip::read::root_dir_common_filter;

/// Handles all errors that occur when using the test data
#[derive(Debug)]
pub enum XSDTestDataError {
    /// Error parsing test data file
    ParseError(String),
    /// Error reading archive (zip) file 
    ArchiveError,
    /// Error extracting (unzipping) archive (zip) file
    ArchiveExtractionError,
    /// Invalid file path 
    InvalidFileError(String),
    /// Unable to read file
    FileReadError(String),
}

/// Struct to hold data for a specific test case (file)
/// 
/// # Fields
/// 
/// - `key` (`String`) - Unique key in database describing the case
/// - `data_set` (`String`) - Name of dataset the file belongs to
/// - `path` (`PathBuf`) - Path to XSD file
/// - `valid` (`bool`) - Whether the test data lists the file as valid
/// 
/// # Examples
/// 
/// ```rust
/// use xsdtestdata::XSDData;
/// 
/// fn check_data(data: &XSDData) {
///     let data_set = data.get_data_set();
///     let valid = data.is_valid();
///     let path = data.get_path();
/// 
///     if data_set == "non-existent data-set" {
///         panic!("This set shouldn't exist");
///     }
/// 
///     if !path.exists() {
///         panic!("Path does not exist");
///     }
/// 
///     if valid {
///         println!("File is valid");
///     }
/// }
/// ```
/// 
#[derive(Clone)]
pub struct XSDData {
    key: String,
    data_set: String,
    path: PathBuf,
    valid: bool,
}

impl XSDData {
    /// Whether the test file is listed as valid
    /// 
    /// # Returns
    /// 
    /// - `bool` - Listed validity
    /// 
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the name of the data set the file belongs to
    /// 
    /// # Returns
    /// 
    /// - `&str` - Data set name
    ///
    pub fn get_data_set(&self) -> &str {
        &self.data_set
    }

    /// Get unique key for test case
    /// 
    /// # Returns
    /// 
    /// - `&str` - Test case key
    /// 
    pub fn get_key(&self) -> &str {
        &self.key
    }

    /// Get test file path
    /// 
    /// # Returns
    /// 
    /// - `&PathBuf` - Path to test file
    /// 
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
    db_root: &PathBuf,
) -> Option<XSDData> {
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

    let data = XSDData {
        key: path,
        data_set: group,
        path: filepath,
        valid: validity,
    };

    Some(data)
}

fn get_schema_info(node: &Node, path: &PathBuf, db_root: &PathBuf) -> Option<XSDData> {
    get_test_info(node, path, "schemaDocument".to_string(), db_root)
}

fn get_schema_test(
    results: &mut XSDTestData,
    node: &Node<'_, '_>,
    path: &PathBuf,
    db_root: &PathBuf,
) {
    if let Some(info) = get_schema_info(node, path, db_root) {
        results.push(info)
    }
}

fn get_instance_info(node: &Node, path: &PathBuf, db_root: &PathBuf) -> Option<XSDData> {
    get_test_info(node, path, "instanceDocument".to_string(), db_root)
}

fn get_test_group(test_group: &Node, path: &PathBuf, db_root: &PathBuf) -> XSDTestData {
    let mut data = XSDTestData::default();
    test_group.children().for_each(|child| {
        let tag_name = child.tag_name().name();

        if tag_name == "schemaTest" {
            get_schema_test(&mut data, &child, path, db_root);
        } else if tag_name == "instanceTest"
            && let Some(new_data) = get_instance_test(&child, path, db_root)
        {
            data += &new_data;
        }
    });

    data
}

fn read_test_set_file(filepath: &PathBuf, db_root: &PathBuf) -> XSDTestData {
    let filedata = read_file(filepath).expect("failed to read file");
    let mut data = XSDTestData::default();

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

fn get_instance_test(node: &Node, path: &PathBuf, db_root: &PathBuf) -> Option<XSDTestData> {
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

/// Struct to manage the XSD test files from [xsdtests]
/// 
/// The struct should be constructor [XSDTestData::new] which checks for the test data at the provided 
/// locations and downloads the necessary files if required.
/// 
/// # Fields
/// 
/// - `data` (`Vec<XSDData>`) - Data on all test files in a vector
/// 
/// # Example
/// 
/// ```rust
/// use std::path::PathBuf;
/// use xsdtestdata::XSDTestData;
/// 
/// fn fetch_test_data(root_dir: PathBuf) {
///     let db_root = root_dir.join("xsdtests-master");
///     
///     let archive_path = root_dir.join("xsdtests.zip");
/// 
///     let data = XSDTestData::new(&db_root, &archive_path);
/// 
///     data.print_stats();
/// }
/// ```
#[derive(Default, Clone)]
pub struct XSDTestData {
    data: Vec<XSDData>,
}

/// Enables use of `lsh += rhs`
impl AddAssign<&XSDTestData> for XSDTestData {
    fn add_assign(&mut self, rhs: &XSDTestData) {
        for data in rhs.data.iter() {
            self.push(data.clone());
        }
    }
}

impl XSDTestData {
    fn total(&self) -> usize {
        self.data.len()
    }

    fn contains(&self, data: &XSDData) -> bool {
        let key = &data.key;
        self.data.iter().any(|d| d.key == *key)
    }

    fn push(&mut self, data: XSDData) {
        if self.contains(&data) {
            return;
        }

        self.data.push(data);
    }

    fn add(&mut self, results: &XSDTestData, ignore: &[PathBuf]) {
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

    fn parse_test_data(test_suite: &PathBuf, db_path: &PathBuf, ignore: &[PathBuf]) -> XSDTestData {
        let filedata = read_file(test_suite).expect("failed to read file");
        let document = parse(&filedata).expect("failed to parse xml");

        let mut output = XSDTestData::default();
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

    /// Initialise a new [XSDTestData](struct@XSDTestData)  struct
    /// 
    /// This is the main constructor for the [XSDTestData](struct@XSDTestData) struct. 
    /// It takes two arguments `db_path` and `archive_path`.
    /// The constructor checks for the database at `db_path`,
    /// if it already exists, it builds the struct using these files.
    /// 
    /// If there is no directory at `db_path`, the method checks whether there
    /// is a compressed archive of the database at `archive_path`.
    /// If there is a zip file at `archive_path`, it unzips it to `db_path`
    /// and uses the data to construct the object.
    /// Otherwise, it downloads the [xsdtests] repo archive zip to `archive_path`
    /// and then extracts it to `db_path`.
    /// 
    /// # Arguments
    /// 
    /// - `db_path` (`&PathBuf`) - Path to the database root directory
    /// - `archive_path` (`&PathBuf`) - Path to the archive (zip) of the database
    /// 
    /// # Returns
    /// 
    /// - [XSDTestData](struct@XSDTestData) - A struct to manager the XSD test data
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use std::path::PathBuf;
    /// use xsdtestdata::XSDTestData;
    /// 
    /// fn get_test_data(root: &PathBuf) -> XSDTestData {
    ///     let db_path = root.join("xsdtests-master");
    ///     let archive = root.join("xsdtests.zip");
    ///     
    ///     XSDTestData::new(&db_path, &archive)
    /// }
    /// ```
    pub fn new(db_path: &PathBuf, archive_path: &PathBuf) -> XSDTestData {
        // TODO Remove ignores
        let ignore = vec![
            file_path(db_path, "msData/particles/particlesZ012.xsd"),
            file_path(db_path, "msData/particles/particlesZ015.xsd"),
            file_path(db_path, "msData/particles/particlesZ020.xsd"),
            file_path(db_path, "msData/regex/reG17.xsd"),
            file_path(db_path, "msData/regex/reJ25.xsd"),
            file_path(db_path, "saxonData/XmlVersions/xv009.xsd"),
        ];
        check_repo(db_path, archive_path);

        let suite = db_path.join("suite.xml");
        let mut data = XSDTestData::parse_test_data(&suite, db_path, &ignore);

        let extra_suite = db_path.join("extra-suite.xml");
        let extra_data = XSDTestData::parse_test_data(&extra_suite, db_path, &ignore);

        data += &extra_data;

        data
    }

    /// Print information about the structure of the test data
    /// 
    /// Prints stats about the database structure, including:
    /// 
    /// - Number of valid schemas
    /// - Number of invalid schemas
    /// - Total number of schemas
    /// - Table of these values for each data set
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use std::path::PathBuf;
    /// use xsdtestdata::XSDTestData;
    /// 
    /// fn print_data(db: &PathBuf, archive: &PathBuf) {
    ///     let data = XSDTestData::new(&db, &archive);
    /// 
    ///     data.print_stats();
    /// }
    /// ```
    pub fn print_stats(&self) {
        let total = self.total();
        let valid = self.data.iter().filter(|data| data.valid).count();
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

        println!(
            "\t{:36}{:8}{:8}{:8}",
            "Data set", "Valid", "Invalid", "Total"
        );

        for data_set in data_sets {
            let valid = self
                .data
                .iter()
                .filter(|data| data.is_set(data_set) && data.valid)
                .count();

            let invalid = self
                .data
                .iter()
                .filter(|data| data.is_set(data_set) && !data.valid)
                .count();

            println!(
                "\t{:36}{:8}{:8}{:8}",
                data_set,
                valid,
                invalid,
                valid + invalid
            );
        }
    }

    fn get_index(&self, index: usize) -> Option<&XSDData> {
        if index < self.total() {
            Some(&self.data[index])
        } else {
            None
        }
    }

    /// Get the test case with key `key`
    /// 
    /// Returns a reference to the [XSDData](struct@XSDData) for the test case with `key`. 
    /// The value is optional and the method returns `None` if the key is not in the data.
    /// 
    /// # Arguments
    /// 
    /// - `key` (`&str`) - Unique key for the desired test case
    /// 
    /// # Returns
    /// 
    /// - `Option<&XSDData>` - Reference to the test case with matching key, or `None` if the key is not present.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use xsdtestdata::XSDTestData;
    /// 
    /// fn check_key(data: &XSDTestData, key: &str) {
    ///     match data.get(key) {
    ///         Some(case) => {
    ///             println!("Found test data for key: {}", key);
    ///             println!("Test case in data set: {}", case.get_data_set());
    ///         },
    ///         None => {
    ///             println!("Key {} is not present in the XSD test data", key);
    ///         },
    ///     }
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<&XSDData> {
        self.data.iter().find(|data| data.key == key)
    }

    /// Convert [XSDTestData](struct@XSDTestData) into an iterator
    /// 
    /// Allows the test data to be iterated over so that each test case can be checked in turn
    /// 
    /// # Returns
    /// 
    /// - `XSDTestDataIterator<'_>` - Iterator over [XSDTestData](struct@XSDTestData)
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use xsdtestdata::XSDTestData;
    /// 
    /// fn check_cases(xsd_test_data: &XSDTestData) {
    ///     xsd_test_data.iter().for_each(|case| {
    ///         let path = case.get_path();
    /// 
    ///         assert!(path.exists());
    ///     });
    /// }
    /// ```
    /// 
    pub fn iter(&'_ self) -> XSDTestDataIterator<'_> {
        XSDTestDataIterator {
            test_data: self,
            index: 0,
        }
    }

}


/// Iterator for [XSDTestData](struct@XSDTestData)
pub struct XSDTestDataIterator<'a> {
    test_data: &'a XSDTestData,
    index: usize,
}

/// Iterate to next entry in the [XSDTestData](struct@XSDTestData) iterator  
impl<'a> Iterator for XSDTestDataIterator<'a> {
    type Item = &'a XSDData;
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

/// An `IntoIterator` for [XSDTestData](struct@XSDTestData)
pub struct XSDTestDataIntoIterator {
    test_data: XSDTestData,
}

/// Iterate `IntoIterator` for [XSDTestData](struct@XSDTestData)
impl Iterator for XSDTestDataIntoIterator {
    type Item = XSDData;

    fn next(&mut self) -> Option<Self::Item> {
        if self.test_data.data.is_empty() {
            return None;
        }

        let result = self.test_data.data.remove(0);

        Some(result)
    }
}

/// Create an `IntoIterator` for [XSDTestData](struct@XSDTestData)
impl IntoIterator for XSDTestData {
    type Item = XSDData;
    type IntoIter = XSDTestDataIntoIterator;

    fn into_iter(self) -> XSDTestDataIntoIterator {
        XSDTestDataIntoIterator { test_data: self }
    }
}

#[cfg(test)]
mod tests {
    use crate::XSDTestData;
    use workspace_root::get_workspace_root;

    #[test]
    fn get_suite() {
        let db = get_workspace_root().join("xsdtests-master");
        let archive = get_workspace_root().join("xsdtests.zip");

        let data = XSDTestData::new(&db, &archive);

        data.print_stats()
    }
}
