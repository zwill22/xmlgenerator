use const_format::{formatcp};
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

    String(String), // A string type with a pattern

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
        const NAME: &str = r"[:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}][-.0-9:A-Z_a-z\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u037D\u037F-\u1FFF\u200C-\u200D\u203F\u2040\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]*";
        const NCNAME: &str = r"[A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}][-.0-9A-Z_a-z\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u037D\u037F-\u1FFF\u200C-\u200D\u203F\u2040\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]*";
        const NMTOKEN: &str = r"[-.0-9:A-Z_a-z\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u037D\u037F-\u1FFF\u200C-\u200D\u203F\u2040\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]+";
        const LANGUAGE: &str = "r[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*";
        const NORMAL: &str = r"[^\r\n\t]*";
        const QNAME: &str = r"(?:[A-Z_a-z][-.0-9A-Z_a-z]*:)?[A-Z_a-z][-\.0-9A-Z_a-z]*";
        const TOKEN: &str = r"[^\s]*(?: [^\s]*)*";
        const NC_NAMES: &str = formatcp!(r"{0}(?:\s+{0})*", NCNAME);
        const BOOLEAN: &str = r"(?:true|false|0|1)";
        const NMTOKENS: &str = formatcp!(r"{0}(?:\s+{0})*", NMTOKEN);
        const NULL: &str = "";

        let date = strftime_to_regex("%Y-%m-%d");
        let datetime = strftime_to_regex("%Y-%m-%dT%H:%M:%S");
        let g_day = strftime_to_regex("-%d");
        let g_month = strftime_to_regex("--%m");
        let g_month_day = strftime_to_regex("--%m-%d");
        let g_year = strftime_to_regex("%Y");
        let g_year_month = strftime_to_regex("%Y-%m");
        let time = strftime_to_regex("%H:%M:%S");

        match s {
            // Numeric Data Types
            "byte" => XsdType::Byte,
            "decimal" => XsdType::String(r"[+-]?[0-9]+\.?[0-9]*".to_string()),
            "short" => XsdType::Short,
            "int" => XsdType::Int,
            "long" => XsdType::Long,
            "integer" => XsdType::String(r"[+-]?[0-9]+".to_string()),
            "negativeInteger" => XsdType::String(r"-[1-9][0-9]+".to_string()),
            "nonNegativeInteger" => XsdType::String(r"(?:0|\+?[1-9][0-9]*)".to_string()),
            "nonPositiveInteger" => XsdType::String(r"(?:0|-[1-9][0-9]*)".to_string()),
            "positiveInteger" => XsdType::String(r"\+?[1-9][0-9]*".to_string()),
            "unsignedLong" => XsdType::UnsignedLong,
            "unsignedInt" => XsdType::UnsignedInt,
            "unsignedShort" => XsdType::UnsignedShort,
            "unsignedByte" => XsdType::UnsignedByte,

            // String data types
            "ENTITY" => XsdType::String(NCNAME.to_string()),
            "ID" => XsdType::String(NCNAME.to_string()),
            "IDREF" => XsdType::String(NCNAME.to_string()),
            "language" => XsdType::String(LANGUAGE.to_string()),
            "Name" => XsdType::String(NAME.to_string()),
            "NCName" => XsdType::String(NCNAME.to_string()),
            "NMTOKEN" => XsdType::String(NMTOKEN.to_string()),
            "normalizedString" => XsdType::String(NORMAL.to_string()),
            "QName" => XsdType::String(QNAME.to_string()),
            "string" => XsdType::String(NULL.to_string()),
            "token" => XsdType::String(TOKEN.to_string()),

            // Date time data types
            "date" => XsdType::String(date),
            "dateTime" => XsdType::String(datetime),
            "gDay" => XsdType::String(g_day),
            "gMonth" => XsdType::String(g_month),
            "gMonthDay" => XsdType::String(g_month_day),
            "gYear" => XsdType::String(g_year),
            "gYearMonth" => XsdType::String(g_year_month),
            "time" => XsdType::String(time),

            // Miscellaneous data types
            "duration" => XsdType::Duration,
            "anyURI" => XsdType::URI,
            "base64Binary" => XsdType::Base64Binary,
            "boolean" => XsdType::String(BOOLEAN.to_string()),
            "float" => XsdType::Float,
            "double" => XsdType::Double,
            "hexBinary" => XsdType::HexBinary,
            "NOTATION" => XsdType::String(NULL.to_string()),

            // List types
            "ENTITIES" => XsdType::String(NC_NAMES.to_string()),
            "NMTOKENS" => XsdType::String(NMTOKENS.to_string()),
            "IDREFS" => XsdType::String(NC_NAMES.to_string()),

            // Just use a string for any type
            "anyType" => XsdType::String(NULL.to_string()),
            "anySimpleType" => XsdType::String(NULL.to_string()),

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
