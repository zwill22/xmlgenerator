use reqwest::get;
use roxmltree::{Document, Node, ParsingOptions};
use std::fs::{File, canonicalize, read_to_string};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use file_to_string::read_file;

#[derive(Debug)]
pub enum XSDTestDataError {
    ParseError(String),
    ArchiveError,
    ArchiveExtractionError,
    InvalidFileError(String),
    FileReadError(String),
}

struct Schema {
    path: PathBuf,
    valid: bool,
}

impl Schema {
    fn new(schema_path: &PathBuf, validity: bool) -> Self {
        Schema {
            path: schema_path.clone(),
            valid: validity,
        }
    }
}

async fn fetch_repo(archive_path: &PathBuf) -> File {
    let url = "https://github.com/w3c/xsdtests/archive/refs/heads/master.zip".to_string();

    let response = get(url).await.expect("failed to send request");
    let content = response.bytes().await.expect("failed to get bytes");

    let mut file = File::create(archive_path).expect("failed to create file");
    file.write_all(&content).expect("failed to write to file");

    file
}

fn extract_repo(db_root: &PathBuf, archive: &File) -> Result<(), XSDTestDataError> {
    let mut archive = match zip::ZipArchive::new(archive) {
        Ok(archive) => archive,
        Err(_) => return Err(XSDTestDataError::ArchiveError),
    };

    match archive.extract(db_root) {
        Ok(_) => Ok(()),
        Err(_) => Err(XSDTestDataError::ArchiveExtractionError),
    }
}

async fn get_archive_file(archive_path: &PathBuf) -> File {
    if !archive_path.exists() {
        return fetch_repo(archive_path).await;
    }

    File::open(archive_path).expect("failed to open file")
}

async fn check_repo(db_root: &PathBuf, archive_path: &PathBuf) {
    if db_root.exists() {
        return;
    }

    let archive = get_archive_file(archive_path).await;

    match extract_repo(db_root, &archive) {
        Ok(_) => {}
        Err(e) => panic!("failed to extract archive file: {:?}", e),
    }
}

fn parse_with_dtd(contents: &'_ String) -> Result<Document<'_>, XSDTestDataError> {
    let mut options = ParsingOptions::default();
    options.allow_dtd = true;

    match Document::parse_with_options(contents, options) {
        Ok(document) => Ok(document),
        Err(e) => Err(XSDTestDataError::ParseError(e.to_string())),
    }
}

fn parse(contents: &'_ String) -> Result<Document<'_>, XSDTestDataError> {
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
    if valid_str == "valid" {
        Some(true)
    } else if valid_str == "invalid" {
        return Some(false);
    } else if valid_str == "indeterminate" {
        return None;
    } else {
        panic!("Unknown validity: {}", valid_str);
    }
}

fn get_test_info(node: &Node, filepath: &PathBuf, tag_name: String) -> Option<Schema> {
    let mut schema_path = None;
    let mut valid = None;

    let path = filepath.parent().unwrap();

    for child in node.children() {
        let tag = child.tag_name().name();
        if tag == tag_name {
            let schema = match get_href_path(&child, path) {
                Ok(path) => path,
                Err(_) => return None,
            };
            if schema == *filepath {
                return None;
            }
            schema_path = Some(schema);
        } else if tag == "expected" {
            let validity = get_validity(&child);
            if validity.is_none() {
                return None;
            }
            valid = validity
        }
    }

    if schema_path.is_none() || valid.is_none() {
        return None;
    }

    let schema = Schema::new(&schema_path.unwrap(), valid.unwrap());

    Some(schema)
}

fn get_schema_info(node: &Node, path: &PathBuf) -> Option<Schema> {
    get_test_info(node, path, "schemaDocument".to_string())
}

fn get_schema_test(results: &mut Vec<Schema>, node: &Node<'_, '_>, path: &PathBuf) {
    match get_schema_info(node, path) {
        Some(info) => results.push(info),
        None => return,
    }
}

fn get_instance_info(node: &Node, path: &PathBuf) -> Option<Schema> {
    get_test_info(node, path, "instanceDocument".to_string())
}

fn get_test_group(results: &mut Vec<Schema>, test_group: &Node, path: &PathBuf) {
    for child in test_group.children() {
        let tag_name = child.tag_name().name();

        if tag_name == "schemaTest" {
            get_schema_test(results, &child, path);
        } else if tag_name == "instanceTest" {
            get_instance_test(results, &child, path);
        }
    }
}

fn read_test_set_file(schemas: &mut Vec<Schema>, filepath: &PathBuf) {
    let filedata = match read_file(&filepath) {
        Ok(s) => s,
        Err(e) => panic!("failed to read file: {:?}", e),
    };

    let document = match parse(&filedata) {
        Ok(d) => d,
        Err(_) => return,
    };

    let root = document.root_element();
    for child in root.children() {
        let tag_name = child.tag_name().name();

        if tag_name == "testGroup" {
            get_test_group(schemas, &child, &filepath)
        }
    }
}

fn get_instance_test(schemas: &mut Vec<Schema>, node: &Node, path: &PathBuf) {
    let instance = match get_instance_info(node, path) {
        Some(info) => info,
        None => return,
    };

    if instance.path == *path {
        return;
    }

    if instance.valid {
        read_test_set_file(schemas, &instance.path);
    }
}

async fn read_test_set(root_path: &PathBuf, extension: &String) -> Vec<Schema> {
    let filepath = root_path.join(extension);
    let mut schemas = Vec::new();
    read_test_set_file(&mut schemas, &filepath);

    schemas
}

fn get_attribute(node: &Node, name: String) -> String {
    for attribute in node.attributes() {
        if attribute.name() == name {
            return attribute.value().to_string();
        }
    }

    panic!("attribute not found");
}

async fn join(output: &mut Vec<(PathBuf, bool)>, new_results: &Vec<Schema>) {
    for result in new_results {
        output.push((result.path.clone(), result.valid));
    }
}

async fn parse_test_data(filepath: &PathBuf, db_root_path: &PathBuf) -> Vec<(PathBuf, bool)> {
    let filedata = read_to_string(&filepath).expect("failed to read file");
    let document = parse(&filedata).expect("failed to parse xml");

    let mut output = vec![];
    let root = document.root_element();
    for child in root.children() {
        let tag = child.tag_name().name();
        if tag == "testSetRef" {
            let test_path = get_attribute(&child, "href".to_string());
            let result = read_test_set(db_root_path, &test_path).await;
            join(&mut output, &result).await;
        }
    }

    output
}

pub async fn get_test_data(
    db_path: &PathBuf,
    archive_path: &PathBuf,
    extra: bool,
) -> Vec<(PathBuf, bool)> {
    check_repo(&db_path, &archive_path).await;

    let suite = db_path.join("suite.xml");
    let mut data = parse_test_data(&suite, &db_path).await;

    if !extra {
        return data;
    }

    let extra_suite = db_path.join("extra-suite.xml");
    let extra_data = parse_test_data(&extra_suite, &db_path).await;

    data.extend(extra_data);

    data
}

#[cfg(test)]
mod tests {
    use crate::get_test_data;
    use futures::executor::block_on;
    use workspace_root::get_workspace_root;

    #[test]
    fn get_suite() {
        let db = get_workspace_root().join("xsdtests-master");
        let archive = get_workspace_root().join("xsdtests.zip");
        let extra = true;
        let data = block_on(get_test_data(&db, &archive, extra));

        let mut valid = 0;
        let mut invalid = 0;
        for (_path, validity) in data {
            if validity {
                valid += 1;
            } else {
                invalid += 1;
            }
        }

        if extra {
            println!("\tXSD Test suite (extended)");
        } else {
            println!("\tXSD Test suite");
        }
        println!("\t{:24}{:6}", "Valid schemas", valid);
        println!("\t{:24}{:6}", "Invalid schemas", invalid);
        println!("\t{:24}{:6}", "Total schemas", valid + invalid);
    }
}
