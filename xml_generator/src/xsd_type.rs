use crate::XMLGeneratorError;
use crate::pattern::Pattern;
use const_format::formatcp;
use http::Uri;
use regex::Regex;
use std::fmt::Display;

#[derive(Default, PartialEq)]
pub(crate) enum XsdType {
    // Integer types
    Byte,          // A signed 8-bit integer
    Short,         // A signed 16-bit integer
    Int,           // A signed 32-bit integer
    Long,          // A signed 64-bit integer
    UnsignedByte,  // An unsigned 8-bit integer
    UnsignedShort, // An unsigned 16-bit integer
    UnsignedInt,   // An unsigned 32-bit integer
    UnsignedLong,  // An unsigned 64-bit integer

    // Floating point numbers
    Float,  // A 32-bit floating point number
    Double, // A 64-bit floating point number

    // Miscellaneous types
    URI,          // Any URI value
    Base64Binary, // A base64 value
    HexBinary,    // A hexidecimal binary value
    Duration,     // A duration of time

    String(Pattern), // A string type with a pattern

    #[default]
    None, // No recognised type
}

fn check_base64(_input: &str) -> bool {
    true
}

fn check_hex(_input: &str) -> bool {
    true
}

fn validate(input_str: &str, pattern: &str) -> Result<bool, XMLGeneratorError> {
    let regex = match Regex::new(pattern) {
        Ok(re) => re,
        Err(_) => return Err(XMLGeneratorError::RegexError(input_str.to_string())),
    };

    let result = regex.is_match(input_str);

    Ok(result)
}

fn validate_duration(input: &str) -> bool {
    // PnYnMnDTnHnMnS
    const DURATION: &str = r"[Pp](?:[0-9]+[Yy])?(?:[0-9]+[Mm])?(?:[0-9]+[Dd])?T?(?:[0-9]+[Hh])?(?:[0-9]+[Mm])?(?:[0-9]+[Ss])?";

    validate(input, DURATION).unwrap_or_else(|_| false)
}

fn validate_pattern(pattern: &Pattern, input: &str) -> bool {
    let re = pattern.get_pattern(true);
    let is_valid = match validate(input, re) {
        Ok(valid) => valid,
        Err(_) => return false,
    };

    if is_valid {
        return true;
    }

    let extended = pattern.get_pattern(false);
    validate(input, extended).unwrap_or_else(|_| false)
}

impl XsdType {
    pub(crate) fn string(string: &str) -> Self {
        let pattern = Pattern::from(string);
        Self::String(pattern)
    }

    fn from_date(string: &str) -> Self {
        const YEAR: &str = r"(?:[0-9]{3}[1-9]|[0-9]{2}[1-9][0-9]|[0-9][1-9][0-9]{2}|[1-9][0-9]{3})";
        const MONTH: &str = r"(?:0[1-9]|1[0-2])";
        const DAY: &str = r"(?:0[1-9]|1[0-9]|2[0-9]|3[0-1])";

        const HOUR: &str = r"(?:0[1-9]|1[0-9]|2[0-3])";
        const MINUTE: &str = r"[0-5][0-9]";
        const SECOND: &str = r"[0-5][0-9]";

        const MONTH_NAME: &str = r"(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)";

        let output = string
            .replace("%Y", YEAR)
            .replace("%m", MONTH)
            .replace("%d", DAY)
            .replace("%H", HOUR)
            .replace("%M", MINUTE)
            .replace("%S", SECOND)
            .replace("%b", MONTH_NAME);

        if output.contains("%") {
            panic!("Raw time string not replaced")
        }

        let pattern = Pattern::from(output.as_str());

        Self::String(pattern)
    }

    pub(crate) fn validate(&self, input: &str) -> bool {
        match self {
            XsdType::Byte => input.parse::<i8>().is_ok(),
            XsdType::Short => input.parse::<i16>().is_ok(),
            XsdType::Int => input.parse::<i32>().is_ok(),
            XsdType::Long => input.parse::<i64>().is_ok(),
            XsdType::UnsignedByte => input.parse::<u8>().is_ok(),
            XsdType::UnsignedShort => input.parse::<u16>().is_ok(),
            XsdType::UnsignedInt => input.parse::<u32>().is_ok(),
            XsdType::UnsignedLong => input.parse::<u64>().is_ok(),
            XsdType::Float => input.parse::<f32>().is_ok(),
            XsdType::Double => input.parse::<f64>().is_ok(),
            XsdType::URI => input.parse::<Uri>().is_ok(),
            XsdType::Base64Binary => check_base64(input),
            XsdType::HexBinary => check_hex(input),
            XsdType::Duration => validate_duration(input),
            XsdType::String(pattern) => validate_pattern(pattern, input),
            XsdType::None => false,
        }
    }
}

impl From<&str> for XsdType {
    fn from(s: &str) -> Self {
        const NAME: &str = r"\i\c*";
        const NCNAME: &str = r"[\i-[:]][\c-[:]]*";
        const NMTOKEN: &str = r"\c+";
        const LANGUAGE: &str = "r[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*";
        const NORMAL: &str = r"[^\r\n\t]*";
        const QNAME: &str = r"(?:[A-Z_a-z][-.0-9A-Z_a-z]*:)?[A-Z_a-z][-\.0-9A-Z_a-z]*";
        const TOKEN: &str = r"[^\s]*(?: [^\s]*)*";
        const NC_NAMES: &str = formatcp!(r"{0}(?:\s+{0})*", NCNAME);
        const BOOLEAN: &str = r"(?:true|false|0|1)";
        const NMTOKENS: &str = formatcp!(r"{0}(?:\s+{0})*", NMTOKEN);
        const NULL: &str = "";

        const DATE: &str = "%Y-%m-%d";
        const DATETIME: &str = "%Y-%m-%dT%H:%M:%S";
        const G_DAY: &str = "-%d";
        const G_MONTH: &str = "--%m";
        const G_MONTH_DAY: &str = "--%m-%d";
        const G_YEAR: &str = "%Y";
        const G_YEAR_MONTH: &str = "%Y-%m";
        const TIME: &str = "%H:%M:%S";

        match s {
            // Numeric Data Types
            "byte" => XsdType::Byte,
            "decimal" => XsdType::string(r"[+-]?[0-9]+\.?[0-9]*"),
            "short" => XsdType::Short,
            "int" => XsdType::Int,
            "long" => XsdType::Long,
            "integer" => XsdType::string(r"[+-]?[0-9]+"),
            "negativeInteger" => XsdType::string(r"-[1-9][0-9]+"),
            "nonNegativeInteger" => XsdType::string(r"(?:0|\+?[1-9][0-9]*)"),
            "nonPositiveInteger" => XsdType::string(r"(?:0|-[1-9][0-9]*)"),
            "positiveInteger" => XsdType::string(r"\+?[1-9][0-9]*"),
            "unsignedLong" => XsdType::UnsignedLong,
            "unsignedInt" => XsdType::UnsignedInt,
            "unsignedShort" => XsdType::UnsignedShort,
            "unsignedByte" => XsdType::UnsignedByte,

            // String data types
            "ENTITY" => XsdType::string(NCNAME),
            "ID" => XsdType::string(NCNAME),
            "IDREF" => XsdType::string(NCNAME),
            "language" => XsdType::string(LANGUAGE),
            "Name" => XsdType::string(NAME),
            "NCName" => XsdType::string(NCNAME),
            "NMTOKEN" => XsdType::string(NMTOKEN),
            "normalizedString" => XsdType::string(NORMAL),
            "QName" => XsdType::string(QNAME),
            "string" => XsdType::string(NULL),
            "token" => XsdType::string(TOKEN),

            // Date time data types
            "date" => XsdType::from_date(DATE),
            "dateTime" => XsdType::from_date(DATETIME),
            "gDay" => XsdType::from_date(G_DAY),
            "gMonth" => XsdType::from_date(G_MONTH),
            "gMonthDay" => XsdType::from_date(G_MONTH_DAY),
            "gYear" => XsdType::from_date(G_YEAR),
            "gYearMonth" => XsdType::from_date(G_YEAR_MONTH),
            "time" => XsdType::from_date(TIME),

            // Miscellaneous data types
            "duration" => XsdType::Duration,
            "anyURI" => XsdType::URI,
            "base64Binary" => XsdType::Base64Binary,
            "boolean" => XsdType::string(BOOLEAN),
            "float" => XsdType::Float,
            "double" => XsdType::Double,
            "hexBinary" => XsdType::HexBinary,
            "NOTATION" => XsdType::string(NULL),

            // List types
            "ENTITIES" => XsdType::string(NC_NAMES),
            "NMTOKENS" => XsdType::string(NMTOKENS),
            "IDREFS" => XsdType::string(NC_NAMES),

            // Just use a string for any type
            "anyType" => XsdType::string(NULL),
            "anySimpleType" => XsdType::string(NULL),

            _ => XsdType::None,
        }
    }
}

impl Display for XsdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            XsdType::Byte => "Byte".to_string(),
            XsdType::Short => "Short".to_string(),
            XsdType::Int => "Int".to_string(),
            XsdType::Long => "Long".to_string(),
            XsdType::UnsignedByte => "UnsignedByte".to_string(),
            XsdType::UnsignedShort => "UnsignedShort".to_string(),
            XsdType::UnsignedInt => "UnsignedInt".to_string(),
            XsdType::UnsignedLong => "UnsignedLong".to_string(),
            XsdType::Float => "Float".to_string(),
            XsdType::Double => "Double".to_string(),
            XsdType::URI => "URI".to_string(),
            XsdType::Base64Binary => "Base64Binary".to_string(),
            XsdType::HexBinary => "HexBinary".to_string(),
            XsdType::Duration => "Duration".to_string(),
            XsdType::String(pattern) => {
                if pattern.is_empty() {
                    "String".to_string()
                } else {
                    format!("String with pattern {}", pattern)
                }
            }
            XsdType::None => "None".to_string(),
        };
        write!(f, "{}", str)
    }
}
