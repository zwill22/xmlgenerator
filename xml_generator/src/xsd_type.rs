pub(crate) enum XsdType {
    // Integer types
    Byte,               // A signed 8-bit integer
    Short,              // A signed 16-bit integer
    Int,                // A signed 32-bit integer
    Long,               // A signed 64-bit integer
    NegativeInt,        // A signed 32-bit negative integer
    NonNegativeInt,     // A signed 32-bit positive integer
    NonPositiveInt,     // A signed 32-bit non-positive integer
    PositiveInt,        // A signed 32-bit positive integer
    UnsignedByte,       // An unsigned 8-bit integer
    UnsignedShort,      // An unsigned 16-bit integer
    UnsignedInt,        // An unsigned 32-bit integer
    UnsignedLong,       // An unsigned 64-bit integer

    // Floating point numbers
    Float,              // A 32-bit floating point number
    Double,             // A 64-bit floating point number

    // Miscellaneous types
    URI,             // Any URI value
    Base64Binary,       // A base64 value
    Boolean,            // A boolean
    HexBinary,          // A hexidecimal binary value
    Duration,           // A duration of time

    String(String),     // A string type with a pattern
    DateTime(String),   // A datetime type with a pattern

    None,                // No recognised type
}

impl From<&str> for XsdType {
    fn from(s: &str) -> Self {
        match s {
            // Numeric Data Types
            "byte" => XsdType::Byte,
            "decimal" => XsdType::Float,
            "short" => XsdType::Short,
            "int" => XsdType::Int,
            "integer" => XsdType::Int,
            "long" => XsdType::Long,
            "negativeInteger" => XsdType::NegativeInt,
            "nonNegativeInteger" => XsdType::NonNegativeInt,
            "nonPositiveInteger" => XsdType::NonPositiveInt,
            "positiveInteger" => XsdType::PositiveInt,
            "unsignedLong" => XsdType::UnsignedLong,
            "unsignedInt" => XsdType::UnsignedInt,
            "unsignedShort" => XsdType::UnsignedShort,
            "unsignedByte" => XsdType::UnsignedByte,

            // String data types
            "ENTITY" => XsdType::String(r"[A-Z_a-z][-.0-9A-Z_a-z]*".to_string()),
            "ID" => XsdType::String(r"[a-zA-Z_][a-zA-Z0-9._-]*".to_string()),
            "IDREF" => XsdType::String(r"[a-zA-Z_][a-zA-Z0-9._-]*".to_string()),
            "language" => XsdType::String(r"([a-zA-Z]{2}|[iI]-[a-zA-Z]+|[xX]-[a-zA-Z]{1,8})(-[a-zA-Z]{1,8})*".to_string()),
            "Name" => XsdType::String(r"[:A-Z_a-z][-.0-9:A-Z_a-z]*".to_string()),
            "NCName" => XsdType::String(r"[A-Z_a-z][-.0-9A-Z_a-z]*".to_string()),
            "NMTOKEN" => XsdType::String(r"[a-zA-Z0-9._\-:]*".to_string()),
            "normalizedString" => XsdType::String(r"[^\r\n\t]*".to_string()),
            "QName" => XsdType::String(r"(?:[A-Z_a-z][-.0-9A-Z_a-z]*:)?[A-Z_a-z][-.0-9A-Z_a-z]*".to_string()),
            "string" => XsdType::String("".to_string()),
            "token" => XsdType::String(r"[a-zA-Z0-9._\-:][a-zA-Z0-9._\-:\s]*[a-zA-Z0-9._\-:]".to_string()),

            // Date time data types
            "date" => XsdType::DateTime("%Y-%m-%d".to_string()),
            "dateTime" => XsdType::DateTime("%Y %b %d %H:%M:%S%.3f %z".to_string()),
            "duration" => XsdType::Duration,
            "gDay" => XsdType::DateTime("%d".to_string()),
            "gMonth" => XsdType::DateTime("%m".to_string()),
            "gMonthDay" => XsdType::DateTime("--%m-%d".to_string()),
            "gYear" => XsdType::DateTime("%Y".to_string()),
            "gYearMonth" => XsdType::DateTime("%Y-%m".to_string()),
            "time" => XsdType::DateTime("%H:%M:%S".to_string()),

            // Miscellaneous data types
            "anyURI" => XsdType::URI,
            "base64Binary" => XsdType::Base64Binary,
            "boolean" => XsdType::Boolean,
            "float" => XsdType::Float,
            "double" => XsdType::Double,
            "hexBinary" => XsdType::HexBinary,
            "NOTATION" => XsdType::String("".to_string()),

            // List types
            "ENTITIES" => XsdType::String(r"([A-Z_a-z][-.0-9A-Z_a-z]*)(\s+[A-Z_a-z][-.0-9A-Z_a-z]*)*".to_string()),
            "NMTOKENS" => XsdType::String(r"([a-zA-Z0-9._:-]+)(\s+[a-zA-Z0-9._:-]+)*".to_string()),
            "IDREFS" => XsdType::String(r"([A-Z_a-z][-.0-9A-Z_a-z]*)(\s+[A-Z_a-z][-.0-9A-Z_a-z]*)*".to_string()),

            // Just use a string for any type
            "anyType" => XsdType::String("".to_string()),
            "anySimpleType" => XsdType::String("".to_string()),

            _ => XsdType::None
        }
    }
}
