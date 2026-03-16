extern crate alloc;

use core::fmt::Display;

use line_ending::LineEnding;
use regex::Regex;
use std::collections::HashMap;

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

fn is_surrogate(code: &str) -> bool {
    let z = u32::from_str_radix(code, 16).unwrap();

    const MIN_STR: &str = "D800";
    const MAX_STR: &str = "DFFF";
    const RADIX: u32 = 16;

    let min = u32::from_str_radix(MIN_STR, RADIX).unwrap();
    let max = u32::from_str_radix(MAX_STR, RADIX).unwrap();

    min <= z && z <= max
}

const fn get_file() -> &'static str {
    include_str!("../data/unicode_blocks.txt")
}

const fn get_data() -> &'static str {
    include_str!("../data/unicode_data.txt")
}

fn get_unicode_categories() -> Result<HashMap<String, Vec<String>>, RegexTranslationError> {
    const DATA: &str = get_data();

    let mut lists: HashMap<String, Vec<String>> = HashMap::new();
    for line in DATA.lines() {
        let values = line.split(";").collect::<Vec<_>>();
        if values.len() != 15 {
            return Err(RegexTranslationError::DataError("Invalid line".to_string()));
        }

        let code = values.first().unwrap().to_string();
        if is_surrogate(&code) {
            continue;
        }

        let group = values.get(2).unwrap().to_string();
        if group.is_empty() {
            continue;
        }

        let supergroup = group.chars().next().unwrap().to_string();

        for item in [group, supergroup].iter() {
            match lists.get_mut(item) {
                Some(list) => {
                    list.push(code.clone());
                }
                None => {
                    lists.insert(item.to_string(), vec![code.clone()]);
                }
            }
        }
    }

    Ok(lists)
}

fn unicode_blocks(ascii: bool) -> Result<HashMap<String, String>, RegexTranslationError> {
    const DATA: &str = get_file();

    let mut map = HashMap::new();
    for line in DATA.lines() {
        if line.starts_with("#") {
            continue;
        }

        if line.is_empty() {
            continue;
        }
        let values = line
            .split(";")
            .map(|c| c.split_whitespace().collect::<Vec<_>>().join(""))
            .collect::<Vec<_>>();

        if values.len() != 2 {
            continue;
        }

        let range: Vec<_> = values.first().unwrap().split(".").collect();

        let min_char = range.first().unwrap().to_string();
        let max_char = range.last().unwrap().to_string();
        let name = values.last().unwrap().to_string();

        if ascii {
            let min_val = u32::from_str_radix(&min_char, 16).unwrap();
            let max_val = u32::from_str_radix(&max_char, 16).unwrap();
            if min_val > 128 || max_val > 128 {
                break;
            }
        }

        if name.contains("Surrogates") {
            continue;
        }

        let key = format!("Is{}", name);
        let range = format!(r"[\u{{{}}}-\u{{{}}}]", min_char, max_char);

        map.insert(key, range);
    }

    Ok(map)
}

fn unicode_categories(ascii: bool) -> Result<HashMap<String, String>, RegexTranslationError> {
    let data = get_unicode_categories()?;

    let mut map = HashMap::new();
    for (key, values) in data {
        let mut string = r"[".to_owned();
        for value in values {
            if ascii {
                let val = u32::from_str_radix(&value, 16).unwrap();
                if val > 128 {
                    continue;
                }
            }
            string.push_str(&format!(r"\x{{{}}}", value));
        }
        string.push(']');

        if string == "[]" {
            continue;
        }

        map.insert(key.to_string(), string);
    }

    Ok(map)
}

fn unicode_definitions(ascii: bool) -> Result<HashMap<String, String>, RegexTranslationError> {
    let mut blocks = unicode_blocks(ascii)?;
    let sets = unicode_categories(ascii)?;

    for (k, v) in sets {
        blocks.insert(k, v);
    }
    Ok(blocks)
}

fn get_comp_set(set: &str, ascii: bool) -> String {
    if ascii {
        return format!(r"[[\x{{0}}-\x{{7F}}]--{}]", set);
    }

    format!(r"[^{}]", set)
}

fn get_unicode_mappings(ascii: bool) -> Result<HashMap<String, String>, RegexTranslationError> {
    // TODO Replace with const
    let unicode_blocks = unicode_definitions(ascii)?;

    let mut output = HashMap::new();

    for (k, v) in unicode_blocks {
        // Unicode block (set)
        let block = format!(r"\p{{{}}}", k);

        output.insert(block, v.to_string());

        // Set negation
        let comp_block = format!(r"\P{{{}}}", k);
        let comp_set = get_comp_set(&v, ascii);

        output.insert(comp_block, comp_set);
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

fn get_ascii_mappings() -> Result<HashMap<String, String>, RegexTranslationError> {
    let mut mappings = get_unicode_mappings(true)?;

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

    Ok(mappings)
}

fn get_full_mappings() -> Result<HashMap<String, String>, RegexTranslationError> {
    let mut mappings = get_unicode_mappings(false)?;

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

    Ok(mappings)
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

fn replace_or_patterns(input: &str) -> Result<String, RegexTranslationError> {
    const OR: &str = r"\[(\w)\|(\w)]";
    let regex = Regex::new(OR)?;

    let result = regex.replace_all(input, "[$1$2]").to_string();

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

fn dot_replace(input: &str) -> Result<String, RegexTranslationError> {
    // Remove groups
    const DOT_GROUP: &str = r"\[([\S+?--\\])\.(\S+?)\]";
    let regex = Regex::new(DOT_GROUP)?;
    let tmp = regex.replace_all(input, r"[$1\.$2]").to_string();

    const DOT: &str = r"([^\\]|^)\.";
    let regex = Regex::new(DOT)?;

    let result = regex.replace_all(&tmp, r"$1[\x{20}-\x{7E}]").to_string();

    Ok(result.to_string())
}

#[derive(Default)]
pub struct RegexTranslator {
    ascii_mappings: HashMap<String, String>,
    unicode_mappings: HashMap<String, String>,
}

impl RegexTranslator {
    pub fn new() -> Result<Self, RegexTranslationError> {
        let translator = Self {
            ascii_mappings: get_ascii_mappings()?,
            unicode_mappings: get_full_mappings()?,
        };

        Ok(translator)
    }

    fn replace(&self, input: &str, ascii: bool) -> Result<String, RegexTranslationError> {
        let mut output = input.to_string();

        if ascii {
            for (k, v) in self.ascii_mappings.iter() {
                if input.contains(k) {
                    output = output.replace(k, v);
                }
            }
            output = dot_replace(&output)?;
        } else {
            for (k, v) in self.unicode_mappings.iter() {
                if input.contains(k) {
                    output = output.replace(k, v);
                }
            }
        }

        // Replace negation patterns
        output = replace_negation_patterns(output.as_str())?;

        // Replace hexidecimal character reference &#x{}; -> \u{}
        output = replace_hex_character_reference(output.as_str())?;

        // Replace decimal character reference &#{}; -> \u{}
        output = replace_decimal_character_reference(output.as_str())?;

        // Replace OR sequence [a|b] with [ab]
        output = replace_or_patterns(output.as_str())?;

        Ok(output)
    }

    pub fn requires_unicode(&self, pattern: &str) -> bool {
        for def in self.unicode_mappings.keys() {
            if self.ascii_mappings.contains_key(def) {
                continue;
            }

            if pattern.contains(def) {
                return true;
            }
        }

        false
    }

    pub fn translate(&self, input: &str, ascii: bool) -> Result<String, RegexTranslationError> {
        validate_input(input)?;

        let output = self.replace(input, ascii)?;

        validate_output(&output)?;

        Ok(output)
    }
}
