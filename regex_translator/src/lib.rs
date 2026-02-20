extern crate alloc;

use core::fmt::Display;

use regex::Regex;
use std::collections::HashMap;
use line_ending::LineEnding;

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

const fn get_file() -> &'static str {
    include_str!("../data/unicode_blocks.txt")
}

fn unicode_blocks() -> Result<HashMap<String, String>, RegexTranslationError> {
    const DATA: &'static str = get_file();

    let mut map = HashMap::new();
    for line in DATA.lines() {
        let values = line.split_whitespace().collect::<Vec<_>>();
        if values.len() != 4 {
            continue;
        }

        let min_char = values[0];
        let max_char = values[2];
        let name = values[3];

        if name.contains("Surrogates") {
            continue;
        }

        let range = format!(r"[\u{{{}}}-\u{{{}}}]", min_char, max_char);

        map.insert(name.to_string(), range.clone());
    }

    Ok(map)
}

fn get_unicode_mappings() -> Result<HashMap<String, String>, RegexTranslationError> {
    // TODO Replace with const
    let mut unicode_blocks = unicode_blocks()?;

    unicode_blocks.insert("Nd".to_string(), r"[0-9]".to_string());
    unicode_blocks.insert(r"L".to_string(), r"[[:alpha:]]".to_string());
    unicode_blocks.insert(r"Ll".to_string(), r"[[:lower:]]".to_string());
    unicode_blocks.insert(r"Lu".to_string(), r"[[:upper:]]".to_string());

    let mut output = HashMap::new();

    for (k, v) in unicode_blocks {
        // Unicode block (set)
        let block = format!(r"\p{{{}}}", k);

        output.insert(block, v.to_string());

        // Set negation
        let neg_block = format!(r"\P{{{}}}", k);
        let neg_set = format!(r"[^{}]", v);

        output.insert(neg_block, neg_set);
    }

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

fn apply_common_mappings(mappings: &mut HashMap<String, String>) {
    // Digit mapping is not recognised by Rust
    const D: &str = r"\d";
    const D_SET: &str = r"[0-9]";

    mappings.insert(D.to_string(), D_SET.to_string());

    // Whitespace
    const S: &str = r"\s";
    let s_set = match LineEnding::from_current_platform() {
        LineEnding::LF => r"[\t \n]",
        LineEnding::CRLF => r"[\t ]|(?:\r\n)",
        LineEnding::CR => r"[\t \r]",
    };

    mappings.insert(S.to_string(), s_set.to_string());
}

fn get_ascii_mappings() -> HashMap<String, String> {
    let mut mappings = HashMap::new();

    const I: &str = r"\i";
    const I_SET: &str = r"[:A-Z_a-z]";

    const C: &str = r"\c";
    const C_SET: &str = r"[-.0-9:A-Z_a-z]";

    mappings.insert(I.to_string(), I_SET.to_string());
    mappings.insert(C.to_string(), C_SET.to_string());

    const NEGATIVE_I: &str = r"\I";
    const NEGATIVE_I_SET: &str = r"[[\x{20}-\x{7E}]--[:A-Z_a-z]]";

    const NEGATIVE_C: &str = r"\C";
    const NEGATIVE_C_SET: &str = r"[[\x{20}-\x{7E}]--[-.0-9:A-Z_a-z]]";

    mappings.insert(NEGATIVE_I.to_string(), NEGATIVE_I_SET.to_string());
    mappings.insert(NEGATIVE_C.to_string(), NEGATIVE_C_SET.to_string());

    const W: &str = r"\w";
    const W_SET: &str = r"[a-zA-Z0-9_]";

    const NEGATIVE_W: &str = r"\W";
    const NEGATIVE_W_SET: &str = r"[[\x{20}-\x{7E}]--[a-zA-Z0-9_]]";

    mappings.insert(W.to_string(), W_SET.to_string());
    mappings.insert(NEGATIVE_W.to_string(), NEGATIVE_W_SET.to_string());

    apply_common_mappings(&mut mappings);

    mappings
}

fn get_full_mappings() -> HashMap<String, String> {
    let mut mappings = HashMap::new();

    const I: &str = r"\i";
    const I_SET: &str = r"[:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]";

    const C: &str = r"\c";
    const C_SET: &str = r"[-.0-9:A-Z_a-z\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u037D\u037F-\u1FFF\u200C-\u200D\u203F\u2040\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]";

    mappings.insert(I.to_string(), I_SET.to_string());
    mappings.insert(C.to_string(), C_SET.to_string());

    const NEGATIVE_I: &str = r"\I";
    const NEGATIVE_I_SET: &str = r"[^[:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]]";

    const NEGATIVE_C: &str = r"\C";
    const NEGATIVE_C_SET: &str = r"[^[-.0-9:A-Z_a-z\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u037D\u037F-\u1FFF\u200C-\u200D\u203F\u2040\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]]";

    mappings.insert(NEGATIVE_I.to_string(), NEGATIVE_I_SET.to_string());
    mappings.insert(NEGATIVE_C.to_string(), NEGATIVE_C_SET.to_string());

    const W: &str = r"\w";
    const W_SET: &str = r"[[[:alpha:]][0-9]_]";

    const NEGATIVE_W: &str = r"\W";
    const NEGATIVE_W_SET: &str = r"[^[[:alnum:]]]";

    mappings.insert(W.to_string(), W_SET.to_string());
    mappings.insert(NEGATIVE_W.to_string(), NEGATIVE_W_SET.to_string());

    apply_common_mappings(&mut mappings);

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

#[derive(Default)]
pub struct RegexTranslator {
    unicode_mappings: HashMap<String, String>,
    ascii_mappings: HashMap<String, String>,
    full_mappings: HashMap<String, String>,
}

impl RegexTranslator {
    pub fn new() -> Result<Self, RegexTranslationError> {
        let translator = Self {
            unicode_mappings: get_unicode_mappings()?,
            ascii_mappings: get_ascii_mappings(),
            full_mappings: get_full_mappings(),
        };

        Ok(translator)
    }

    fn replace(&self, input: &str, ascii: bool) -> Result<String, RegexTranslationError> {
        let mut output = input.to_string();

        for (k, v) in self.unicode_mappings.iter() {
            if input.contains(k) {
                output = output.as_str().replace(k, v.as_str());
            }
        }

        if ascii {
            for (k, v) in self.ascii_mappings.iter() {
                if input.contains(k) {
                    output = output.as_str().replace(k, v.as_str());
                }
            }
        } else {
            for (k, v) in self.full_mappings.iter() {
                if input.contains(k) {
                    output = output.as_str().replace(k, v.as_str());
                }
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

    pub fn translate(&self, input: &str, ascii: bool) -> Result<String, RegexTranslationError> {
        validate_input(input)?;

        let output = self.replace(input, ascii)?;

        validate_output(&output)?;

        Ok(output)
    }
}
