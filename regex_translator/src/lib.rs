extern crate alloc;

use core::fmt::Display;
use polars::prelude::*;

use regex::Regex;
use std::collections::HashMap;
use std::path;

mod test;

#[derive(Debug)]
pub enum RegexTranslationError {
    InvalidInput(String),
    RegexError(String),
    FileReadError(String),
    DataError(String),
    SurrogatesError,
}

impl From<regexml::Error> for RegexTranslationError {
    fn from(e: regexml::Error) -> Self {
        match e {
            regexml::Error::Internal => {
                RegexTranslationError::InvalidInput("Internal error".to_string())
            }
            regexml::Error::InvalidFlags(e) => RegexTranslationError::InvalidInput(e.to_string()),
            regexml::Error::Syntax(e) => RegexTranslationError::InvalidInput(e.to_string()),
            regexml::Error::MatchesEmptyString => {
                RegexTranslationError::InvalidInput("Empty string".to_string())
            }
            regexml::Error::InvalidReplacementString(e) => {
                RegexTranslationError::InvalidInput(e.to_string())
            }
        }
    }
}

impl From<regex::Error> for RegexTranslationError {
    fn from(e: regex::Error) -> Self {
        match e {
            regex::Error::Syntax(expr) => RegexTranslationError::RegexError(expr),
            regex::Error::CompiledTooBig(u) => {
                let str = format!("Regex string cannot be compiled, size: {}", u);

                RegexTranslationError::RegexError(str)
            }
            _ => RegexTranslationError::RegexError(e.to_string()),
        }
    }
}

impl Display for RegexTranslationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RegexTranslationError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
            RegexTranslationError::RegexError(e) => write!(f, "Regex error: {}", e),
            RegexTranslationError::FileReadError(e) => write!(f, "File read error: {}", e),
            RegexTranslationError::DataError(e) => write!(f, "Data error: {}", e),
            RegexTranslationError::SurrogatesError => write!(f, "Surrogate error"),
        }
    }
}
fn validate_input(input: &str) -> Result<(), RegexTranslationError> {
    match regexml::Regex::xsd(input, "") {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
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
                    lit(r"[\u{${1}}-\u{${2}}]"),
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
            Err(e) => {
                if !k.to_string().contains("Surrogates") {
                    return Err(RegexTranslationError::FileReadError(format!(
                        "Unknown blocks in datafile: {}",
                        e
                    )));
                }
            }
        }
    }

    // Additional mappings
    map.insert(
        "CombiningMarksforSymbols".to_string(),
        r"[\u20D0-\u20FF]".to_string(),
    );

    // Bopomofo includes extended character in Rust Regex crate
    map.insert("Bopomofo".to_string(), r"[\u3105-\u312f]".to_string());

    Ok(map)
}

fn get_unicode_mappings() -> Result<HashMap<String, String>, RegexTranslationError> {
    // TODO Replace with const
    let unicode_blocks = unicode_blocks()?;

    let mut output = HashMap::new();

    for (k, v) in unicode_blocks {
        // Unicode block (set)
        let block = format!(r"\p{{Is{}}}", k);

        output.insert(block, v.to_string());

        // Set negation
        let neg_block = format!(r"\P{{Is{}}}", k);
        let neg_set = format!(r"[^{}]", v);

        output.insert(neg_block, neg_set);
    }

    // Additional values
    output.insert(r"\p{Nd}".to_string(), r"[0-9]".to_string());

    Ok(output)
}

fn handle_surrogates(output: &str) -> Result<(), RegexTranslationError> {
    let surrogate_strings = vec![
        r"\p{IsHighSurrogates}",
        r"\p{IsHighPrivateUseSurrogates}",
        r"\p{IsLowSurrogates}",
        r"\P{IsHighSurrogates}",
        r"\P{IsHighPrivateUseSurrogates}",
        r"\P{IsLowSurrogates}",
    ];

    for string in surrogate_strings {
        if output.contains(string) {
            return Err(RegexTranslationError::SurrogatesError);
        }
    }

    Ok(())
}

fn get_xml_mappings() -> HashMap<String, String> {
    let mut mappings = HashMap::new();
    let i = r"\i".to_string();
    let i_set = r"[:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]";
    let i_ascii = r"[:A-Z_a-z]";

    let c = r"\c".to_string();
    let c_set = r"[-.0-9:A-Z_a-z\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u037D\u037F-\u1FFF\u200C-\u200D\u203F\u2040\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]";
    let c_ascii = r"[-.0-9:A-Z_a-z]";

    mappings.insert(i, i_ascii.to_string());
    mappings.insert(c, c_ascii.to_string());

    let neg_i = r"\I".to_string();
    let neg_i_set = format!(r"[^{}]", i_set);

    let neg_c = r"\C".to_string();
    let neg_c_set = format!(r"[^{}]", c_set);

    mappings.insert(neg_i, neg_i_set);
    mappings.insert(neg_c, neg_c_set);

    // Digit mapping is not recognised by Rust
    let d = r"\d".to_string();
    let d_set = r"[0-9]".to_string();

    mappings.insert(d, d_set);

    mappings
}

fn validate_output(output: &str) -> Result<(), RegexTranslationError> {
    match Regex::new(output) {
        Ok(_) => Ok(()),
        Err(e) => {
            handle_surrogates(output)?;

            Err(e.into())
        }
    }
}

fn parse_radix_string(input: &str, radix: u32) -> Result<u32, RegexTranslationError> {
    match u32::from_str_radix(input, radix) {
        Ok(v) => Ok(v),
        Err(_) => Err(RegexTranslationError::RegexError(
            "Error converting hex string to integer.".to_string(),
        )),
    }
}

fn replace_negation_patterns(input: &str) -> Result<String, RegexTranslationError> {
    const NEGATION: &str = r"\[(\S*?[\S--[-]])-(\[\S*?])]";
    let regex = Regex::new(NEGATION)?;

    let result = regex.replace_all(input, "[$1--$2]").to_string();

    Ok(result)
}

fn replace_character_reference(
    pattern: &str,
    input: &str,
    radix: u32,
) -> Result<String, RegexTranslationError> {
    let regex = Regex::new(pattern)?;

    let mut output = input.to_string();
    for captures in regex.captures_iter(input) {
        let full_match = captures.get(0).unwrap().as_str();
        let partial_match = captures.get(1).unwrap().as_str();

        let value = parse_radix_string(partial_match, radix)?;
        let hex_str = format!(r"\u{:>04x}", value);

        output = output.replace(full_match, &hex_str);
    }

    Ok(output)
}

fn replace_hex_character_reference(input: &str) -> Result<String, RegexTranslationError> {
    const HEX: u32 = 16;
    const PATTERN: &str = r"&#x(\w+.*?);";
    replace_character_reference(PATTERN, input, HEX)
}

fn replace_decimal_character_reference(input: &str) -> Result<String, RegexTranslationError> {
    const DEC: u32 = 10;
    const PATTERN: &str = r"&#(\w+.*?);";
    replace_character_reference(PATTERN, input, DEC)
}

pub struct RegexTranslator {
    mappings: HashMap<String, String>,
}

impl RegexTranslator {
    pub fn new() -> Result<Self, RegexTranslationError> {
        let mut mappings = get_unicode_mappings()?;

        let xml_mappings = get_xml_mappings();
        mappings.extend(xml_mappings);

        Ok(Self { mappings })
    }

    fn replace(&self, input: &str) -> Result<String, RegexTranslationError> {
        let mut output = input.to_string();

        for (k, v) in self.mappings.iter() {
            if input.contains(k) {
                output = output.as_str().replace(k, v.as_str());
            }
        }

        // Replace negation patterns
        output = replace_negation_patterns(output.as_str())?;

        // Replace hexidecimal character reference &#x{}; -> \u{}
        output = replace_hex_character_reference(output.as_str())?;

        // Replace decimal character reference &#{}; -> \u{}
        output = replace_decimal_character_reference(output.as_str())?;

        Ok(output)
    }

    pub fn translate(&self, input: &str) -> Result<String, RegexTranslationError> {
        validate_input(input)?;

        let output = self.replace(input)?;

        validate_output(&output)?;

        Ok(output)
    }
}
