use http::Uri;
use std::fmt::Display;

fn strftime_to_regex(input: &str) -> String {
    const YEAR: &str = r"(?:[0-9]{3}[1-9]|[0-9]{2}[1-9][0-9]|[0-9][1-9][0-9]{2}|[1-9][0-9]{3})";
    const MONTH: &str = r"(?:0[1-9]|1[0-2])";
    const DAY: &str = r"(?:0[1-9]|1[0-9]|2[0-9]|3[0-1])";

    const HOUR: &str = r"(?:0[1-9]|1[0-9]|2[0-3])";
    const MINUTE: &str = r"[0-5][0-9]";
    const SECOND: &str = r"[0-5][0-9]";

    const MONTH_NAME: &str = r"(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)";

    let output = input
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

    output
}

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

    String(String),   // A string type with a pattern

    #[default]
    None, // No recognised type
}

fn check_base64(_input: &str) -> bool {
    true
}

fn check_hex(_input: &str) -> bool {
    true
}

fn check_duration(_input: &str) -> bool {
    true
}

fn check_string(_input: &str, _pattern: &str) -> bool {
    true
}

impl XsdType {
    pub(crate) fn is_valid(&self, input: &str) -> bool {
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
            XsdType::Duration => check_duration(input),
            XsdType::String(pattern) => check_string(input, pattern),
            XsdType::None => false,
        }
    }
}

impl From<&str> for XsdType {
    fn from(s: &str) -> Self {
        match s {
            // Numeric Data Types
            "byte" => XsdType::Byte,
            "decimal" => XsdType::String(r"[+-]?[1-9][0-9]*(?:.[0-9]+)?".into()),
            "short" => XsdType::Short,
            "int" => XsdType::Int,
            "long" => XsdType::Long,
            "integer" => XsdType::String(r"[+-]?[0-9]+".into()),
            "negativeInteger" => XsdType::String(r"-[1-9][0-9]+".into()),
            "nonNegativeInteger" => XsdType::String(r"(?:0|\+?[1-9][0-9]*)".into()),
            "nonPositiveInteger" => XsdType::String(r"(?:0|-[1-9][0-9]*)".into()),
            "positiveInteger" => XsdType::String(r"\+?[1-9][0-9]*".into()),
            "unsignedLong" => XsdType::UnsignedLong,
            "unsignedInt" => XsdType::UnsignedInt,
            "unsignedShort" => XsdType::UnsignedShort,
            "unsignedByte" => XsdType::UnsignedByte,

            // String data types
            "ENTITY" => XsdType::String(r"[A-Z_a-z][-.0-9A-Z_a-z]*".to_string()),
            "ID" => XsdType::String(r"[a-zA-Z_][a-zA-Z0-9._-]*".to_string()),
            "IDREF" => XsdType::String(r"[a-zA-Z_][a-zA-Z0-9._-]*".to_string()),
            "language" => XsdType::String(
                r"([a-zA-Z]{2}|[iI]-[a-zA-Z]+|[xX]-[a-zA-Z]{1,8})(-[a-zA-Z]{1,8})*".to_string(),
            ),
            "Name" => XsdType::String(r"[:A-Z_a-z][-.0-9:A-Z_a-z]*".to_string()),
            "NCName" => XsdType::String(r"[A-Z_a-z][-.0-9A-Z_a-z]*".to_string()),
            "NMTOKEN" => XsdType::String(r"[a-zA-Z0-9._\-:]*".to_string()),
            "normalizedString" => XsdType::String(r"[^\r\n\t]*".to_string()),
            "QName" => XsdType::String(
                r"(?:[A-Z_a-z][-.0-9A-Z_a-z]*:)?[A-Z_a-z][-.0-9A-Z_a-z]*".to_string(),
            ),
            "string" => XsdType::String("".to_string()),
            "token" => {
                XsdType::String(r"[a-zA-Z0-9._\-:][a-zA-Z0-9._\-:\s]*[a-zA-Z0-9._\-:]".to_string())
            }

            // Date time data types
            "date" => XsdType::String(strftime_to_regex("%Y-%m-%d")),
            "dateTime" => XsdType::String(strftime_to_regex("%Y-%m-%dT%H:%M:%S")),
            "gDay" => XsdType::String(strftime_to_regex("-%d")),
            "gMonth" => XsdType::String(strftime_to_regex("--%m")),
            "gMonthDay" => XsdType::String(strftime_to_regex("--%m-%d")),
            "gYear" => XsdType::String(strftime_to_regex("%Y")),
            "gYearMonth" => XsdType::String(strftime_to_regex("%Y-%m")),
            "time" => XsdType::String(strftime_to_regex("%H:%M:%S")),

            // Miscellaneous data types
            "duration" => XsdType::Duration,
            "anyURI" => XsdType::URI,
            "base64Binary" => XsdType::Base64Binary,
            "boolean" => XsdType::String(r"(?:true|false|0|1)".to_string()),
            "float" => XsdType::Float,
            "double" => XsdType::Double,
            "hexBinary" => XsdType::HexBinary,
            "NOTATION" => XsdType::String("".to_string()),

            // List types
            "ENTITIES" => XsdType::String(
                r"([A-Z_a-z][-.0-9A-Z_a-z]*)(\s+[A-Z_a-z][-.0-9A-Z_a-z]*)*".to_string(),
            ),
            "NMTOKENS" => XsdType::String(r"([a-zA-Z0-9._:-]+)(\s+[a-zA-Z0-9._:-]+)*".to_string()),
            "IDREFS" => XsdType::String(
                r"([A-Z_a-z][-.0-9A-Z_a-z]*)(\s+[A-Z_a-z][-.0-9A-Z_a-z]*)*".to_string(),
            ),

            // Just use a string for any type
            "anyType" => XsdType::String("".to_string()),
            "anySimpleType" => XsdType::String("".to_string()),

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
