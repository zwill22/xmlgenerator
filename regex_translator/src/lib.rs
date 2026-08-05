//! Crate for translating an `xs:pattern` to a Rust Regex pattern
//!
//! The `xs:pattern` specification is a form of regular expression.
//! However, the syntax differs from the one used in Rust's [regex] crate in several ways.
//! This crate provides methods to translate from the XSD pattern to a Rust-style pattern.
//!
//! [regex]: https://crates.io/crates/regex

extern crate alloc;

use core::fmt::Display;

use line_ending::LineEnding;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::num::ParseIntError;
use unic_char_basics::{is_noncharacter, is_private_use};
use unic_ucd::CharAge;
use unic_ucd_block::BlockIter;
use unic_ucd_category::GeneralCategory;

mod test;

/// Enum for handling different errors in the [RegexTranslator]
#[derive(Debug)]
pub enum RegexTranslationError {
    /// The input is not a valid `xs:pattern`
    InvalidInput(String),
    /// Error occurs during translation
    RegexError(String),
    /// Error parsing character data.
    DataError(String),
    /// Error converting unicode character to int
    UnicodeError(String),
    /// Pattern contains surrogates
    SurrogatesError,
}

/// Convert an error from [regexml] to a [RegexTranslationError::InvalidInput] error
/// 
/// [regexml]: https://crates.io/crates/regexml
///
/// # Arguments
///
/// - `e` (`regexml::Error`) - A [regexml] error
///
/// # Returns
///
/// - [RegexTranslationError] - A [RegexTranslationError::InvalidInput] error
///
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

/// Converts a `regex::Error` into a [RegexTranslationError::RegexError]
///
/// # Arguments
///
/// - `e` (`regex::Error``) - A [regex] error
///
/// # Returns
///
/// - [RegexTranslationError] - A [RegexTranslationError::RegexError]
/// 
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

/// Converts a `ParseIntError` into a [RegexTranslationError::UnicodeError]
///
/// # Arguments
///
/// - `e` (`ParseIntError``) - A `ParseIntError` from `std::num`
///
/// # Returns
///
/// - [RegexTranslationError] - A [RegexTranslationError::UnicodeError]
///
impl From<ParseIntError> for RegexTranslationError {
    fn from(e: ParseIntError) -> Self {
        RegexTranslationError::UnicodeError(e.to_string())
    }
}

impl Display for RegexTranslationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RegexTranslationError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
            RegexTranslationError::RegexError(e) => write!(f, "Regex error: {}", e),
            RegexTranslationError::DataError(e) => write!(f, "Data error: {}", e),
            RegexTranslationError::SurrogatesError => write!(f, "Surrogate error"),
            RegexTranslationError::UnicodeError(e) => write!(f, "Unicode error: {}", e),
        }
    }
}

fn is_surrogate(character: char) -> bool {
    const MIN: u32 = 55296; // 0xD800
    const MAX: u32 = 55296; // 0xDFFF

    let z = character as u32;

    MIN <= z && z <= MAX
}

fn is_supported(character: char) -> bool {
    if character.is_control() {
        return false;
    }

    if is_surrogate(character) {
        return false;
    }

    if is_noncharacter(character) {
        return false;
    }

    if is_private_use(character) {
        return false;
    }

    if character.age().is_none() {
        return false;
    }

    true
}

fn get_unicode_categories() -> HashMap<String, Vec<char>> {
    let mut lists: HashMap<String, Vec<char>> = HashMap::new();

    for block in BlockIter::new() {
        for character in block.range.iter() {
            if !is_supported(character) {
                continue;
            }
            let category = (match GeneralCategory::of(character) {
                GeneralCategory::UppercaseLetter => "Lu",
                GeneralCategory::LowercaseLetter => "Ll",
                GeneralCategory::TitlecaseLetter => "Lt",
                GeneralCategory::ModifierLetter => "Lm",
                GeneralCategory::OtherLetter => "Lo",
                GeneralCategory::NonspacingMark => "Mn",
                GeneralCategory::SpacingMark => "Mc",
                GeneralCategory::EnclosingMark => "Me",
                GeneralCategory::DecimalNumber => "Nd",
                GeneralCategory::LetterNumber => "Nl",
                GeneralCategory::OtherNumber => "No",
                GeneralCategory::ConnectorPunctuation => "Pc",
                GeneralCategory::DashPunctuation => "Pd",
                GeneralCategory::OpenPunctuation => "Ps",
                GeneralCategory::ClosePunctuation => "Pe",
                GeneralCategory::InitialPunctuation => "Pi",
                GeneralCategory::FinalPunctuation => "Pf",
                GeneralCategory::OtherPunctuation => "Po",
                GeneralCategory::MathSymbol => "Sm",
                GeneralCategory::CurrencySymbol => "Sc",
                GeneralCategory::ModifierSymbol => "Sk",
                GeneralCategory::OtherSymbol => "So",
                GeneralCategory::SpaceSeparator => "Zs",
                GeneralCategory::LineSeparator => "Zl",
                GeneralCategory::ParagraphSeparator => "Zp",
                GeneralCategory::Control => "Cc",
                GeneralCategory::Format => "Cf",
                GeneralCategory::Surrogate => "Cs",
                GeneralCategory::PrivateUse => "Co",
                GeneralCategory::Unassigned => "Cn",
            })
            .to_string();

            let supergroup = category.chars().next().unwrap().to_string();

            for item in [category, supergroup].iter() {
                match lists.get_mut(item) {
                    Some(list) => {
                        list.push(character);
                    }
                    None => {
                        lists.insert(item.to_string(), vec![character]);
                    }
                }
            }
        }
    }

    lists
}

fn add(map: &mut HashMap<String, String>, key: &str, value: &str) {
    map.insert(key.to_string(), value.to_string());

    let mut alternative_names = HashMap::new();
    alternative_names.insert(
        "IsCombiningDiacriticalMarksforSymbols".to_string(),
        "IsCombiningMarksforSymbols".to_string(),
    );

    alternative_names.insert("IsGreekandCoptic".to_string(), "IsGreek".to_string());
    alternative_names.insert("IsPrivateUseArea".to_string(), "IsPrivateUse".to_string());

    if alternative_names.contains_key(key) {
        let alt_key = &alternative_names[key];
        map.insert(alt_key.to_string(), value.to_string());
    }
}

fn unicode_blocks() -> HashMap<String, String> {
    let mut map = HashMap::new();

    for block in BlockIter::new() {
        let name = block.name.split_whitespace().collect::<Vec<_>>().join("");

        let range = block.range;

        let min_char = range.low;
        let max_char = range.high;

        let key = format!("Is{}", name);
        let range = format!(r"[{}-{}]", min_char, max_char);

        add(&mut map, &key, &range);
    }

    map
}

fn unicode_categories(ascii: bool) -> HashMap<String, String> {
    let data = get_unicode_categories();

    let mut map = HashMap::new();
    for (key, values) in data {
        let mut string = r"[".to_owned();
        for value in values {
            if ascii && (value as u32) > 128 {
                continue;
            }
            string.push_str(value.escape_unicode().to_string().as_str());
        }
        string.push(']');

        if string == "[]" {
            continue;
        }

        map.insert(key.to_string(), string);
    }

    map
}

fn unicode_definitions(ascii: bool) -> HashMap<String, String> {
    let mut blocks = if ascii {
        HashMap::new()
    } else {
        unicode_blocks()
    };
    let sets = unicode_categories(ascii);

    blocks.extend(sets);

    blocks
}

fn get_comp_set(set: &str, ascii: bool) -> String {
    if ascii {
        return format!(r"[[\x{{0}}-\x{{7F}}]--{}]", set);
    }

    format!(r"[^{}]", set)
}

fn get_unicode_mappings(ascii: bool) -> HashMap<String, String> {
    let unicode_blocks = unicode_definitions(ascii);

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

    output
}

fn into_sets<Str: Display>(input: Vec<Str>) -> HashSet<String> {
    let mut output = HashSet::new();

    for k in input {
        output.insert(format!(r"\p{{{}}}", k));
        output.insert(format!(r"\P{{{}}}", k));
    }

    output
}

fn get_unsupported() -> HashSet<String> {
    let mut out = vec![];

    for block in BlockIter::new() {
        let count = block.range.iter().filter(|c| is_supported(*c)).count();

        if count > 0 {
            continue;
        }

        let name = block.name.split_whitespace().collect::<Vec<_>>().join("");
        let key = format!("Is{}", name);

        out.push(key);
    }

    into_sets(out)
}

fn get_surrogates() -> HashSet<String> {
    let surrogate_strings = vec![
        "IsHighSurrogates",
        "IsHighPrivateUseSurrogates",
        "IsLowSurrogates",
    ];

    into_sets(surrogate_strings)
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
    let mut mappings = get_unicode_mappings(true);

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
    const W_SET: &str = r"[a-zA-Z0-9]";

    const NEGATIVE_W: &str = r"\W";
    const NEGATIVE_W_SET: &str = r"[[\x{20}-\x{7E}]--[a-zA-Z0-9_]]";

    mappings.insert(W.to_string(), W_SET.to_string());
    mappings.insert(NEGATIVE_W.to_string(), NEGATIVE_W_SET.to_string());

    apply_common_mappings(&mut mappings);

    mappings
}

fn get_full_mappings() -> HashMap<String, String> {
    let mut mappings = get_unicode_mappings(false);

    const I: &str = r"\i";
    const I_SET: &str = r"[:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]";

    const C: &str = r"\c";
    const C_SET: &str = r"[-\.0-9:A-Z_a-z\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u037D\u037F-\u1FFF\u200C-\u200D\u203F\u2040\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]";

    mappings.insert(I.to_string(), I_SET.to_string());
    mappings.insert(C.to_string(), C_SET.to_string());

    const NEGATIVE_I: &str = r"\I";
    const NEGATIVE_I_SET: &str = r"[^[:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]]";

    const NEGATIVE_C: &str = r"\C";
    const NEGATIVE_C_SET: &str = r"[^[-\.0-9:A-Z_a-z\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u037D\u037F-\u1FFF\u200C-\u200D\u203F\u2040\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]]";

    mappings.insert(NEGATIVE_I.to_string(), NEGATIVE_I_SET.to_string());
    mappings.insert(NEGATIVE_C.to_string(), NEGATIVE_C_SET.to_string());

    const W: &str = r"\w";
    const W_SET: &str = r"[[[:alpha:]][0-9]]";

    const NEGATIVE_W: &str = r"\W";
    const NEGATIVE_W_SET: &str = r"[^[[:alnum:]]]";

    mappings.insert(W.to_string(), W_SET.to_string());
    mappings.insert(NEGATIVE_W.to_string(), NEGATIVE_W_SET.to_string());

    apply_common_mappings(&mut mappings);

    mappings
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

        let value = u32::from_str_radix(partial_match, radix)?;
        let character = match char::from_u32(value) {
            Some(c) => c,
            None => {
                return Err(RegexTranslationError::DataError(
                    "Unable to parse character reference.".to_string(),
                ));
            }
        };

        output = output.replace(full_match, character.escape_unicode().to_string().as_str());
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

/// Regex translator struct manages translation between Regex types
///
/// The struct manages methods for converting from `xs:pattern` to [regex].
/// This includes mappings for ASCII only translations (`ascii_mappings`),
/// and mappings for full translations (`unicode_mappings`).
/// It also includes two sets of unsupported patterns, including `surrogates` and `unsupported`.
///
/// A new struct should be initialised using [RegexTranslator::new].
/// Translations can then be processed using [RegexTranslator::translate].
/// Another method [RegexTranslator::requires_unicode] is provided to determine whether the input pattern contains any patterns that require non-ASCII translation.
///
/// # Fields
///
/// - `ascii_mappings` (`HashMap<String, String>`) - A map of translations (ASCII only)
/// - `unicode_mappings` (`HashMap<String, String>`) - A full map of translations
/// - `unsupported` (`HashSet<String>`) - Set of unsupported patterns
/// - `surrogates` (`HashSet<String>`) - Set of surrogate patterns
///
/// # Example
///
/// ```rust
/// use regextranslator::RegexTranslator;
///
/// fn translate_xsd_to_rust_regex(input: &str) -> String {
///     let translator = RegexTranslator::new();
///
///     match translator.translate(input, false) {
///         Ok(output) => {
///             return output;
///         },
///         Err(e) => {
///             panic!("Error translating input: {}", e);
///         },
///     }
/// }
/// ```
///
#[derive(Default)]
pub struct RegexTranslator {
    ascii_mappings: HashMap<String, String>,
    unicode_mappings: HashMap<String, String>,
    unsupported: HashSet<String>,
    surrogates: HashSet<String>,
}

impl RegexTranslator {
    /// Constructor for the [RegexTranslator], should always be used to initialise an instance of the struct.
    ///
    /// This method collects all the data about the differences between the Regex flavours.
    /// It creates the mappings and sets necessary to perform the translations and check for unsupported patterns.
    /// It is recommended to only initialise one instance, since multiple instances may consume resources.
    ///
    /// # Returns
    ///
    /// - `Self` - Returns a [RegexTranslator] instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use regextranslator::RegexTranslator;
    ///
    /// let _ = RegexTranslator::new();
    /// ```
    pub fn new() -> Self {
        Self {
            ascii_mappings: get_ascii_mappings(),
            unicode_mappings: get_full_mappings(),
            unsupported: get_unsupported(),
            surrogates: get_surrogates(),
        }
    }

    fn handle_surrogates(&self, input: &str) -> Result<(), RegexTranslationError> {
        for pattern in self.surrogates.iter() {
            if input.contains(pattern) {
                return Err(RegexTranslationError::SurrogatesError);
            }
        }

        Ok(())
    }

    fn handle_unsupported_patterns(&self, input: &str) -> Result<(), RegexTranslationError> {
        for pattern in self.unsupported.iter() {
            if input.contains(pattern) {
                return Err(RegexTranslationError::InvalidInput(pattern.to_string()));
            }
        }

        Ok(())
    }

    fn validate_input(&self, input: &str) -> Result<(), RegexTranslationError> {
        regexml::Regex::xsd(input, "")?;

        self.handle_surrogates(input)?;
        self.handle_unsupported_patterns(input)?;

        Ok(())
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

        // Replace hexadecimal character reference &#x{}; -> \u{}
        output = replace_hex_character_reference(output.as_str())?;

        // Replace decimal character reference &#{}; -> \u{}
        output = replace_decimal_character_reference(output.as_str())?;

        // Replace OR sequence [a|b] with [ab]
        output = replace_or_patterns(output.as_str())?;

        Ok(output)
    }

    /// Check if an input pattern requires full Unicode mappings
    ///
    /// # Arguments
    ///
    /// - `pattern` (`&str`) - Input XSD-style regex pattern
    ///
    /// # Returns
    ///
    /// - `bool` - Whether a full unicode translation is required
    ///
    /// # Example
    ///
    /// ```rust
    /// use regextranslator::{RegexTranslator, RegexTranslationError};
    ///
    /// fn run_translator(input: &str) -> Result<String, RegexTranslationError> {
    ///     let translator = RegexTranslator::new();
    ///
    ///     let unicode = translator.requires_unicode(input);
    ///
    ///     translator.translate(input, !unicode)
    /// }
    /// ```
    ///
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

    fn validate_output(&self, output: &str) -> Result<(), RegexTranslationError> {
        match Regex::new(output) {
            Ok(_) => Ok(()),
            Err(_) => self.handle_surrogates(output),
        }
    }

    /// Translates `input` pattern to Rust [regex] compatible version
    ///
    /// The function takes the `input` pattern, and checks whether it is a valid `xs:pattern` using the [regexml] crate.
    /// It then translates the pattern into one that is compatible with Rust's [regex] crate.
    /// If `ascii = true`, the translator uses `ascii_mappings` to translate patterns.
    /// If `ascii = false`, the translator uses the full `unicode_mappings` to translate patterns.
    /// Any input that contains an unsupported pattern will return an error.
    /// Once the pattern is translated, it is verified using `regex::Regex`.
    ///
    /// # Arguments
    ///
    /// - `input` (`&str`) - Input pattern (`xs:pattern`)
    /// - `ascii` (`bool`) - Only use ASCII translations
    ///
    /// # Returns
    ///
    /// - `Result<String, RegexTranslationError>` - Returns a translated regex pattern on success or a [RegexTranslationError] on failure
    ///
    /// # Errors
    ///
    /// - [RegexTranslationError::InvalidInput] - The input pattern is not a valid `xs:pattern` 
    /// - [RegexTranslationError::RegexError] - Error during translation
    /// - [RegexTranslationError::DataError] - Error parsing character data
    /// - [RegexTranslationError::UnicodeError] - Error converting character to int
    /// - [RegexTranslationError::SurrogatesError] - pattern contains surrogates
    /// 
    ///
    /// # Examples
    ///
    /// ```
    /// use regextranslator::RegexTranslator;
    /// 
    /// fn translate_pattern(pattern: &str) -> String {
    ///     let translator = RegexTranslator::new();
    /// 
    ///     match translator.translate(pattern, false) {
    ///         Ok(output) => {
    ///             return output;
    ///         },
    ///         Err(e) => {
    ///             panic!("Unable to translate pattern");
    ///         }
    ///     }
    /// }
    /// ```
    /// 
    pub fn translate(&self, input: &str, ascii: bool) -> Result<String, RegexTranslationError> {
        self.validate_input(input)?;

        let output = self.replace(input, ascii)?;

        self.validate_output(&output)?;

        Ok(output)
    }
}
