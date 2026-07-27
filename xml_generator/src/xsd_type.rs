use crate::XMLGeneratorError;
use crate::datetime::Datetime;
use crate::error::unimplemented;
use crate::pattern::Pattern;
use crate::whitespace::WhiteSpace;
use const_format::formatcp;
use http::Uri;
use language_tags::LanguageTag;
use regex::Regex;
use std::fmt::Display;

#[derive(Default, PartialEq)]
pub(crate) enum XSDType {
    // Integer types
    Byte, // A signed 8-bit integer
    Short, // A signed 16-bit integer
    Int, // A signed 32-bit integer
    Long, // A signed 64-bit integer
    UnsignedByte, // An unsigned 8-bit integer
    UnsignedShort, // An unsigned 16-bit integer
    UnsignedInt, // An unsigned 32-bit integer
    UnsignedLong, // An unsigned 64-bit integer

    // Floating point numbers
    Float, // A 32-bit floating point number
    Double, // A 64-bit floating point number

    // Miscellaneous types
    Uri, // Any URI value

    // TODO Implement these types properly
    // Base64Binary, // A base64 value
    // HexBinary,    // A hexadecimal binary value
    Duration, // A duration of time
    Language, // An RFC 1766 language string

    DateTime(Datetime),
    String(Pattern), // A string type with a pattern

    #[default]
    None, // No recognised type
}

fn validate(input_str: &str, pattern: &str) -> Result<bool, XMLGeneratorError> {
    let regex = match Regex::new(pattern) {
        Ok(re) => re,
        Err(_) => {
            return Err(XMLGeneratorError::RegexError(input_str.to_string()));
        }
    };

    let result = regex.is_match(input_str);

    Ok(result)
}

fn validate_duration(input: &str) -> bool {
    // PnYnMnDTnHnMnS
    const DURATION: &str =
        r"[Pp](?:[0-9]+[Yy])?(?:[0-9]+[Mm])?(?:[0-9]+[Dd])?T?(?:[0-9]+[Hh])?(?:[0-9]+[Mm])?(?:[0-9]+[Ss])?";

    validate(input, DURATION).unwrap_or(false)
}

fn validate_language(input: &str) -> bool {
    match LanguageTag::parse(input) {
        Ok(tag) => tag.is_valid(),
        Err(_) => false,
    }
}

fn validate_pattern(pattern: &Pattern, input: &str) -> bool {
    let re = pattern.get_pattern(true);
    let is_valid = match validate(input, re) {
        Ok(valid) => valid,
        Err(_) => {
            return false;
        }
    };

    if is_valid {
        return true;
    }

    let extended = pattern.get_pattern(false);
    validate(input, extended).unwrap_or(false)
}

impl XSDType {
    pub(crate) fn string(string: &str, whitespace: &WhiteSpace) -> Result<Self, XMLGeneratorError> {
        let pattern = Pattern::from_string(string, whitespace)?;
        let out = Self::String(pattern);
        Ok(out)
    }

    fn from_date(string: &str) -> Result<Self, XMLGeneratorError> {
        let datetime = Datetime::from(string);

        let out = Self::DateTime(datetime);
        Ok(out)
    }

    pub(crate) fn validate(&self, input: &str) -> bool {
        match self {
            XSDType::Byte => input.parse::<i8>().is_ok(),
            XSDType::Short => input.parse::<i16>().is_ok(),
            XSDType::Int => input.parse::<i32>().is_ok(),
            XSDType::Long => input.parse::<i64>().is_ok(),
            XSDType::UnsignedByte => input.parse::<u8>().is_ok(),
            XSDType::UnsignedShort => input.parse::<u16>().is_ok(),
            XSDType::UnsignedInt => input.parse::<u32>().is_ok(),
            XSDType::UnsignedLong => input.parse::<u64>().is_ok(),
            XSDType::Float => input.parse::<f32>().is_ok(),
            XSDType::Double => input.parse::<f64>().is_ok(),
            XSDType::Uri => input.parse::<Uri>().is_ok(),
            // XSDType::Base64Binary => unimplemented!("Base64Binary"),
            // XSDType::HexBinary => unimplemented!("HexBinary"),
            XSDType::Duration => validate_duration(input),
            XSDType::Language => validate_language(input),
            XSDType::DateTime(_) => unimplemented!("Datetime validation"),
            XSDType::String(pattern) => validate_pattern(pattern, input),
            XSDType::None => false,
        }
    }

    pub(crate) fn whitespace(&self) -> WhiteSpace {
        match self {
            XSDType::Byte => WhiteSpace::Preserve,
            XSDType::Short => WhiteSpace::Preserve,
            XSDType::Int => WhiteSpace::Preserve,
            XSDType::Long => WhiteSpace::Collapse,
            XSDType::UnsignedByte => WhiteSpace::Preserve,
            XSDType::UnsignedShort => WhiteSpace::Preserve,
            XSDType::UnsignedInt => WhiteSpace::Preserve,
            XSDType::UnsignedLong => WhiteSpace::Preserve,
            XSDType::Float => WhiteSpace::Collapse,
            XSDType::Double => WhiteSpace::Collapse,
            XSDType::Uri => WhiteSpace::Collapse,
            // XSDType::Base64Binary => WhiteSpace::Collapse,
            // XSDType::HexBinary => WhiteSpace::Collapse,
            XSDType::Duration => WhiteSpace::Collapse,
            XSDType::Language => WhiteSpace::Collapse,
            XSDType::DateTime(_) => WhiteSpace::Collapse,
            XSDType::String(pattern) => pattern.get_whitespace(),
            XSDType::None => WhiteSpace::Preserve,
        }
    }

    pub(crate) fn get_pattern(&self) -> Option<&Pattern> {
        match self {
            XSDType::DateTime(datetime) => Some(datetime.get_pattern()),
            XSDType::String(pattern) => {
                if pattern.is_empty() {
                    return None;
                }

                Some(pattern)
            }
            _ => None,
        }
    }

    pub(crate) fn from_string(s: &str) -> Result<Self, XMLGeneratorError> {
        const NAME: &str = r"\i\c*";
        const NC_NAME: &str = r"[\i--[:]][\c--[:]]*";
        const NM_TOKEN: &str = r"\c+";
        const NORMAL: &str = r"[^\r\n\t]*";
        const TOKEN: &str = r"[^\s]*(?: [^\s]*)*";
        const NC_NAMES: &str = formatcp!(r"{0}(?:\s+{0})*", NC_NAME);
        const BOOLEAN: &str = r"(?:true|false|0|1)";
        const NM_TOKENS: &str = formatcp!(r"{0}(?:\s+{0})*", NM_TOKEN);
        const NULL: &str = "";

        const DATE: &str = "%Y-%m-%d";
        const DATETIME: &str = "%Y-%m-%dT%H:%M:%S";
        const G_DAY: &str = "---%d";
        const G_MONTH: &str = "--%m";
        const G_MONTH_DAY: &str = "--%m-%d";
        const G_YEAR: &str = "%Y";
        const G_YEAR_MONTH: &str = "%Y-%m";
        const TIME: &str = "%H:%M:%S";

        const COLLAPSE: WhiteSpace = WhiteSpace::Collapse;
        const REPLACE: WhiteSpace = WhiteSpace::Replace;
        const PRESERVE: WhiteSpace = WhiteSpace::Preserve;

        match s {
            // Numeric Data Types
            "byte" => Ok(XSDType::Byte),
            "decimal" => XSDType::string(r"(\+|-)?([0-9]+(\.[0-9]*)?|\.[0-9]+)", &COLLAPSE),
            "short" => Ok(XSDType::Short),
            "int" => Ok(XSDType::Int),
            "long" => Ok(XSDType::Long),
            "integer" => XSDType::string(r"[+-]?[0-9]+", &COLLAPSE),
            "negativeInteger" => XSDType::string(r"-[1-9][0-9]+", &COLLAPSE),
            "nonNegativeInteger" => XSDType::string(r"0|(?:\+?[1-9][0-9]*)", &COLLAPSE),
            "nonPositiveInteger" => XSDType::string(r"0|(?:-[1-9][0-9]*)", &COLLAPSE),
            "positiveInteger" => XSDType::string(r"\+?[1-9][0-9]*", &COLLAPSE),
            "unsignedLong" => Ok(XSDType::UnsignedLong),
            "unsignedInt" => Ok(XSDType::UnsignedInt),
            "unsignedShort" => Ok(XSDType::UnsignedShort),
            "unsignedByte" => Ok(XSDType::UnsignedByte),

            // String data types
            "ENTITY" => XSDType::string(NC_NAME, &COLLAPSE),
            "ID" => XSDType::string(NC_NAME, &COLLAPSE),
            "IDREF" => unimplemented("IDREF"), // Requires cross-referencing
            "language" => Ok(XSDType::Language),
            "Name" => XSDType::string(NAME, &COLLAPSE),
            "NCName" => XSDType::string(NC_NAME, &COLLAPSE),
            "NMTOKEN" => XSDType::string(NM_TOKEN, &COLLAPSE),
            "normalizedString" => XSDType::string(NORMAL, &REPLACE),
            "QName" => unimplemented("QName"), // Requires cross-referencing
            "string" => XSDType::string(NULL, &PRESERVE),
            "token" => XSDType::string(TOKEN, &COLLAPSE),

            // Date time data types
            "date" => XSDType::from_date(DATE),
            "dateTime" => XSDType::from_date(DATETIME),
            "gDay" => XSDType::from_date(G_DAY),
            "gMonth" => XSDType::from_date(G_MONTH),
            "gMonthDay" => XSDType::from_date(G_MONTH_DAY),
            "gYear" => XSDType::from_date(G_YEAR),
            "gYearMonth" => XSDType::from_date(G_YEAR_MONTH),
            "time" => XSDType::from_date(TIME),

            // Miscellaneous data types
            "duration" => Ok(XSDType::Duration),
            "anyURI" => Ok(XSDType::Uri),
            "base64Binary" => unimplemented("base64Binary"),
            "boolean" => XSDType::string(BOOLEAN, &COLLAPSE),
            "float" => Ok(XSDType::Float),
            "double" => Ok(XSDType::Double),
            "hexBinary" => unimplemented("hexBinary"),
            "NOTATION" => XSDType::string(NULL, &COLLAPSE),

            // List types
            "ENTITIES" => XSDType::string(NC_NAMES, &PRESERVE),
            "NMTOKENS" => XSDType::string(NM_TOKENS, &PRESERVE),
            "IDREFS" => unimplemented("IDREFS"), // Requires cross-referencing

            // Just use a string for any type
            "anyType" => XSDType::string(NULL, &PRESERVE),
            "anySimpleType" => XSDType::string(NULL, &PRESERVE),

            _ => Ok(XSDType::None),
        }
    }
}

impl Display for XSDType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            XSDType::Byte => "Byte".to_string(),
            XSDType::Short => "Short".to_string(),
            XSDType::Int => "Int".to_string(),
            XSDType::Long => "Long".to_string(),
            XSDType::UnsignedByte => "UnsignedByte".to_string(),
            XSDType::UnsignedShort => "UnsignedShort".to_string(),
            XSDType::UnsignedInt => "UnsignedInt".to_string(),
            XSDType::UnsignedLong => "UnsignedLong".to_string(),
            XSDType::Float => "Float".to_string(),
            XSDType::Double => "Double".to_string(),
            XSDType::Uri => "URI".to_string(),
            // XSDType::Base64Binary => "Base64Binary".to_string(),
            // XSDType::HexBinary => "HexBinary".to_string(),
            XSDType::Duration => "Duration".to_string(),
            XSDType::Language => "Language".to_string(),
            XSDType::DateTime(datetime) => { format!("Datetime with pattern: {}", datetime) }
            XSDType::String(pattern) => {
                if pattern.is_empty() {
                    "String".to_string()
                } else {
                    format!("String with pattern {}", pattern)
                }
            }
            XSDType::None => "None".to_string(),
        };
        write!(f, "{}", str)
    }
}
