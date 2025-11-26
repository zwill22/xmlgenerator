extern crate alloc;

use polars::prelude::*;
use regex;
use regexml;

use std::collections::HashMap;
use std::path;

#[derive(Debug)]
pub enum RegexTranslationError {
    InvalidInput(String),
    RegexError(String),
    FileReadError(String),
    DataError(String),
}

impl From<regexml::Error> for RegexTranslationError {
    fn from(value: regexml::Error) -> Self {
        match value {
            regexml::Error::Internal => {
                RegexTranslationError::InvalidInput(String::from("Internal error"))
            }
            regexml::Error::InvalidFlags(e) => RegexTranslationError::InvalidInput(e.to_string()),
            regexml::Error::Syntax(e) => RegexTranslationError::InvalidInput(e.to_string()),
            regexml::Error::MatchesEmptyString => {
                RegexTranslationError::InvalidInput(String::from("Empty string"))
            }
            regexml::Error::InvalidReplacementString(e) => {
                RegexTranslationError::InvalidInput(e.to_string())
            }
        }
    }
}

impl From<regex::Error> for RegexTranslationError {
    fn from(value: regex::Error) -> Self {
        match value {
            regex::Error::Syntax(e) => RegexTranslationError::RegexError(e.to_string()),
            regex::Error::CompiledTooBig(e) => {
                let str = format!("Regex string cannot be compiled, size: {}", e);
                RegexTranslationError::RegexError(str)
            }
            _ => RegexTranslationError::RegexError("Unknown error".to_string()),
        }
    }
}

fn validate_input(input: &str) -> Result<(), RegexTranslationError> {
    match regexml::Regex::xsd(input, "") {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Invalid input: {}", input);
            Err(e.into())
        }
    }
}

fn get_file_path() -> PlPath {
    const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
    const SEPARATOR: char = path::MAIN_SEPARATOR;
    const RELATIVE_DIR: &str = "data/unicode_blocks.txt";

    let full_path = MANIFEST_DIR.to_string() + &*SEPARATOR.to_string() + RELATIVE_DIR;

    PlPath::new(full_path.as_str())
}

fn get_string(
    data_frame: &DataFrame,
    column: &str,
    index: usize,
) -> Result<String, RegexTranslationError> {
    match data_frame.column(column).unwrap().get(index).unwrap() {
        AnyValue::String(string) => Ok(string.to_owned()),
        _ => Err(RegexTranslationError::DataError(
            "Value is not a string".to_string(),
        )),
    }
}

fn unicode_blocks() -> Result<HashMap<String, String>, RegexTranslationError> {
    let path = get_file_path();

    let lf = LazyCsvReader::new(path)
        .with_has_header(true)
        .with_separator(b'\t')
        .with_infer_schema_length(None)
        .finish()
        .expect("Unable to read file");

    let df = match lf
        .select([
            col("Block range")
                .str()
                .replace_all(
                    lit(r#""?U\+(.*?)..U\+(.*?)$"?"#),
                    lit(r"[\u${1}-\u${2}]"),
                    false,
                )
                .str()
                .replace_all(lit(r#"""#), lit(r""), false)
                .alias("Ranges"),
            col("Block name")
                .str()
                .replace_all(lit(r"\[\w+\]"), lit(r""), false)
                .str()
                .replace_all(lit(r"\s"), lit(r""), false)
                .str()
                .replace_all(lit(r"-"), lit(""), false)
                .str()
                .replace_all(lit(r#"""#), lit(r""), false)
                .alias("Name"),
        ])
        .collect()
    {
        Ok(d) => d,
        Err(e) => return Err(RegexTranslationError::FileReadError(e.to_string())),
    };

    let mut map = HashMap::new();
    for i in 0..df.height() {
        let k = get_string(&df, "Name", i)?;
        let v = get_string(&df, "Ranges", i)?;

        match validate_output(v.as_str()) {
            Ok(_) => {
                map.insert(k, v);
            }
            Err(_) => {
                if !k.to_string().contains("Surrogates") {
                    return Err(RegexTranslationError::FileReadError(
                        "Unknown blocks in datafile".to_string(),
                    ));
                }
            }
        }
    }

    Ok(map)
}

fn replace(input: &str) -> Result<String, RegexTranslationError> {
    let map = unicode_blocks()?; // TODO Replace with const

    let mut output = input.to_string();

    for (k, v) in map {
        let string = format!(r"\p{{Is{}}}", k);

        if input.contains(&string) {
            output = input.replace(&string, &v);
        }
    }

    Ok(output)
}

fn validate_output(output: &str) -> Result<(), RegexTranslationError> {
    match regex::Regex::new(output) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn translate(input: &str) -> Result<String, RegexTranslationError> {
    validate_input(input)?;

    let output = replace(input)?;
    validate_output(&output)?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use file_to_string::read_file;
    use roxmltree::{Attribute, Document};
    use roxmltree::Node;
    use crate::translate;

    use std::path::PathBuf;
    use tokio::runtime::Runtime;
    use xsdtestdata::get_test_data;
    use workspace_root::get_workspace_root;


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

        // Create the runtime
        let rt = Runtime::new().unwrap();

        // Spawn a future onto the runtime
        let test_data = rt.block_on(get_test_data(root, archive, true));

        for (filepath, listed_as_valid) in test_data {
            if listed_as_valid {
                parse_file(&mut regex, &filepath);
            }

        }

        regex
    }

    #[test]
    fn it_works() {
        let root = get_workspace_root();
        let db_root = root.as_path().join("xsdtests-master");
        let archive = root.as_path().join("xsdtests.zip");

        let regex = get_regex_strings(&db_root, &archive);

        for input_regex in regex {
            match translate(input_regex.as_str()) {
                Ok(out) => {
                    if input_regex != out {
                        print!("Translated:\t{}\t->\t{}", input_regex, out);
                    }
                },
                Err(e) => panic!("Error translating: {}", input_regex)
            }
        }
    }
}
