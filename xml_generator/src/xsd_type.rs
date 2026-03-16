use crate::XMLGeneratorError;
use crate::datetime::Datetime;
use crate::error::unimplemented;
use crate::pattern::Pattern;
use crate::whitespace::WhiteSpace;
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
    Uri, // Any URI value

    // TODO Implement these types properly
    // Base64Binary, // A base64 value
    // HexBinary,    // A hexidecimal binary value
    Duration, // A duration of time

    DateTime(Datetime),
    String(Pattern), // A string type with a pattern

    #[default]
    None, // No recognised type
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

    validate(input, DURATION).unwrap_or(false)
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
    validate(input, extended).unwrap_or(false)
}

impl XsdType {
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
            XsdType::Uri => input.parse::<Uri>().is_ok(),
            // XsdType::Base64Binary => unimplemented!("Base64Binary"),
            // XsdType::HexBinary => unimplemented!("HexBinary"),
            XsdType::Duration => validate_duration(input),
            XsdType::DateTime(_) => unimplemented!("Datetime validation"),
            XsdType::String(pattern) => validate_pattern(pattern, input),
            XsdType::None => false,
        }
    }

    pub(crate) fn whitespace(&self) -> WhiteSpace {
        match self {
            XsdType::Byte => WhiteSpace::Preserve,
            XsdType::Short => WhiteSpace::Preserve,
            XsdType::Int => WhiteSpace::Preserve,
            XsdType::Long => WhiteSpace::Collapse,
            XsdType::UnsignedByte => WhiteSpace::Preserve,
            XsdType::UnsignedShort => WhiteSpace::Preserve,
            XsdType::UnsignedInt => WhiteSpace::Preserve,
            XsdType::UnsignedLong => WhiteSpace::Preserve,
            XsdType::Float => WhiteSpace::Collapse,
            XsdType::Double => WhiteSpace::Collapse,
            XsdType::Uri => WhiteSpace::Collapse,
            // XsdType::Base64Binary => WhiteSpace::Collapse,
            // XsdType::HexBinary => WhiteSpace::Collapse,
            XsdType::Duration => WhiteSpace::Collapse,
            XsdType::DateTime(_) => WhiteSpace::Collapse,
            XsdType::String(pattern) => pattern.get_whitespace(),
            XsdType::None => WhiteSpace::Preserve,
        }
    }

    pub(crate) fn get_pattern(&self) -> Option<&Pattern> {
        match self {
            XsdType::DateTime(datetime) => Some(datetime.get_pattern()),
            XsdType::String(pattern) => {
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
        const NCNAME: &str = r"[\i--[:]][\c--[:]]*";
        const NMTOKEN: &str = r"\c+";
        const LANGUAGE: &str = "r[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*";
        const NORMAL: &str = r"[^\r\n\t]*";
        const TOKEN: &str = r"[^\s]*(?: [^\s]*)*";
        const NC_NAMES: &str = formatcp!(r"{0}(?:\s+{0})*", NCNAME);
        const BOOLEAN: &str = r"(?:true|false|0|1)";
        const NMTOKENS: &str = formatcp!(r"{0}(?:\s+{0})*", NMTOKEN);
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
            "byte" => Ok(XsdType::Byte),
            "decimal" => XsdType::string(r"(\+|-)?([0-9]+(\.[0-9]*)?|\.[0-9]+)", &COLLAPSE),
            "short" => Ok(XsdType::Short),
            "int" => Ok(XsdType::Int),
            "long" => Ok(XsdType::Long),
            "integer" => XsdType::string(r"[+-]?[0-9]+", &COLLAPSE),
            "negativeInteger" => XsdType::string(r"-[1-9][0-9]+", &COLLAPSE),
            "nonNegativeInteger" => XsdType::string(r"0|(?:\+?[1-9][0-9]*)", &COLLAPSE),
            "nonPositiveInteger" => XsdType::string(r"0|(?:-[1-9][0-9]*)", &COLLAPSE),
            "positiveInteger" => XsdType::string(r"\+?[1-9][0-9]*", &COLLAPSE),
            "unsignedLong" => Ok(XsdType::UnsignedLong),
            "unsignedInt" => Ok(XsdType::UnsignedInt),
            "unsignedShort" => Ok(XsdType::UnsignedShort),
            "unsignedByte" => Ok(XsdType::UnsignedByte),

            // String data types
            "ENTITY" => XsdType::string(NCNAME, &COLLAPSE),
            "ID" => XsdType::string(NCNAME, &COLLAPSE),
            "IDREF" => unimplemented("IDREF"), // Requires cross-referencing
            "language" => XsdType::string(LANGUAGE, &COLLAPSE),
            "Name" => XsdType::string(NAME, &COLLAPSE),
            "NCName" => XsdType::string(NCNAME, &COLLAPSE),
            "NMTOKEN" => XsdType::string(NMTOKEN, &COLLAPSE),
            "normalizedString" => XsdType::string(NORMAL, &REPLACE),
            "QName" => unimplemented("QName"), // Requires cross-referencing
            "string" => XsdType::string(NULL, &PRESERVE),
            "token" => XsdType::string(TOKEN, &COLLAPSE),

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
            "duration" => Ok(XsdType::Duration),
            "anyURI" => Ok(XsdType::Uri),
            "base64Binary" => unimplemented("base64Binary"),
            "boolean" => XsdType::string(BOOLEAN, &COLLAPSE),
            "float" => Ok(XsdType::Float),
            "double" => Ok(XsdType::Double),
            "hexBinary" => unimplemented("hexBinary"),
            "NOTATION" => XsdType::string(NULL, &COLLAPSE),

            // List types
            "ENTITIES" => XsdType::string(NC_NAMES, &PRESERVE),
            "NMTOKENS" => XsdType::string(NMTOKENS, &PRESERVE),
            "IDREFS" => unimplemented("IDREFS"), // Requires cross-referencing

            // Just use a string for any type
            "anyType" => XsdType::string(NULL, &PRESERVE),
            "anySimpleType" => XsdType::string(NULL, &PRESERVE),

            _ => Ok(XsdType::None),
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
            XsdType::Uri => "URI".to_string(),
            // XsdType::Base64Binary => "Base64Binary".to_string(),
            // XsdType::HexBinary => "HexBinary".to_string(),
            XsdType::Duration => "Duration".to_string(),
            XsdType::DateTime(datetime) => {
                format!("Datetime with pattern: {}", datetime)
            }
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
