//! Module containing the [XMLGeneratorError] struct
//!
//! All errors in the [crate] should raise an [XMLGeneratorError] rather than panicking.
//!
use quick_xml::events::attributes::AttrError;
use quick_xml::Error as QuickXMLError;
use regex_intersect::IntersectError;
use regextranslator::RegexTranslationError;
use xml_builder::XMLError;
use xsdvalidator::XSDValidationError;

/// XML generator error
///
/// Struct which manages errors in the XMLGenerator crate
#[derive(Debug)]
pub enum XMLGeneratorError {
    /// Error due to invalid input path
    InvalidPathError(String),
    /// Error from XSDvalidator module
    XSDValidatorError(String),
    /// Data type Error
    DataTypeInformationError(String),
    /// Error finding matching data type
    DataTypeNotFoundError(String),
    /// Error parsing the input XSD file contents
    XSDParserError(String),
    /// Datatypes are in an invalid format
    DataTypesFormatError(String),
    /// Error generating the output XML structure
    XMLBuilderError(String),
    /// Invalid XSD version
    InvalidXSDVersionError(String),
    /// Recursion detected
    InfiniteRecursionError,
    /// XSD contains no elements
    NoElementsError,
    /// Invalid XSD
    InvalidXSDError(String),
    /// Invalid name in XSD
    InvalidXSDNameError(String),
    /// No independent elements in XSD
    NoIndependentElementsError,
    /// Multiple Roots elements in XSD
    MultipleRootsError,
    /// Generator Error
    TypeGenerationError(String),
    /// Regex Compilation Error
    RegexError(String),
    /// Regex mismatch error
    RegexMismatchError(String, String),
    /// Line endings (platform dependent)
    LineEndingsError(String),
    /// Encoding error
    EncodingError,
    /// Unimplemented Feature
    UnimplementedFeature(String),
}

/// Converts errors from [quick-xml] to [XMLGeneratorError::InvalidXSDError]
/// 
/// [quick-xml]: https://crates.io/crates/quick_xml
/// 
/// # Arguments
/// 
/// - `err` (`QuickXMLError`) - An error from [quick-xml].
/// 
/// # Returns
/// 
/// - [XMLGeneratorError] - An [XMLGeneratorError::InvalidXSDError]
/// 
impl From<QuickXMLError> for XMLGeneratorError {
    fn from(err: QuickXMLError) -> XMLGeneratorError {
        match err {
            QuickXMLError::Io(error) => XMLGeneratorError::InvalidPathError(error.to_string()),
            QuickXMLError::Syntax(error) => XMLGeneratorError::InvalidXSDError(error.to_string()),
            QuickXMLError::IllFormed(error) => {
                XMLGeneratorError::InvalidXSDError(error.to_string())
            }
            QuickXMLError::InvalidAttr(error) => {
                XMLGeneratorError::InvalidXSDError(error.to_string())
            }
            QuickXMLError::Encoding(error) => XMLGeneratorError::InvalidXSDError(error.to_string()),
            QuickXMLError::Escape(error) => XMLGeneratorError::InvalidXSDError(error.to_string()),
            QuickXMLError::Namespace(error) => {
                XMLGeneratorError::InvalidXSDError(error.to_string())
            }
        }
    }
}

/// Converts an `AttrError` from [quick-xml] into an [XMLGeneratorError::InvalidXSDError]
/// 
/// # Arguments
/// 
/// - `err` (`AttrError`) - An `AttrError` thrown by [quick-xml].
/// 
/// # Returns
/// 
/// - [XMLGeneratorError] - An [XMLGeneratorError::InvalidXSDError]
/// 
impl From<AttrError> for XMLGeneratorError {
    fn from(err: AttrError) -> XMLGeneratorError {
        XMLGeneratorError::InvalidXSDError(err.to_string())
    }
}

/// Converts an `IntersectError` from [regex-intersect] into an [XMLGeneratorError::RegexError]
/// 
/// [regex-intersect]: https://crates.io/crates/regex_intersect
/// 
/// # Arguments
/// 
/// - `error` (`IntersectError`) - An `IntersectError` thrown by [regex-intersect]
/// 
/// # Returns
/// 
/// - [XMLGeneratorError] - An [XMLGeneratorError::RegexError]
/// 
impl From<IntersectError> for XMLGeneratorError {
    fn from(error: IntersectError) -> XMLGeneratorError {
        match error {
            IntersectError::Parse(error) => XMLGeneratorError::RegexError(error.to_string()),
            IntersectError::Error(error) => XMLGeneratorError::RegexError(error.to_string()),
        }
    }
}

/// Converts am [XSDValidationError] into an [XMLGeneratorError]
/// 
/// # Arguments
/// 
/// - `value` (`XSDValidationError`) - An [XSDValidationError] from [xsdvalidator]
/// 
/// # Returns
/// 
/// - [XMLGeneratorError] - An [XMLGeneratorError::XSDValidatorError] or an [XMLGeneratorError::InvalidXSDError]
/// 
impl From<XSDValidationError> for XMLGeneratorError {
    fn from(value: XSDValidationError) -> XMLGeneratorError {
        match value {
            XSDValidationError::PathError => XMLGeneratorError::InvalidPathError("".to_string()),
            XSDValidationError::LibXML2InterfaceError(e) => {
                XMLGeneratorError::XSDValidatorError(format!("LibXML2 interface error: {}", e))
            }
            XSDValidationError::OutputRedirectError(e) => {
                XMLGeneratorError::XSDValidatorError(e.to_string())
            }
            XSDValidationError::ParseError(e) => {
                XMLGeneratorError::XSDValidatorError(e.to_string())
            }
            XSDValidationError::ReadFileError => {
                XMLGeneratorError::XSDValidatorError("File read error".to_string())
            }
            XSDValidationError::XSDRecursionError => {
                XMLGeneratorError::InvalidXSDError("Recursive elements found".to_string())
            }
        }
    }
}

/// Converts a [RegexTranslationError] into an [XMLGeneratorError]
/// 
/// A [RegexTranslationError] is converted into one of the following:
/// 
/// - [XMLGeneratorError::InvalidXSDError] - if pattern is invalid
/// - [XMLGeneratorError::RegexError] - if there are problems converting unicode to int
/// - [XMLGeneratorError::UnimplementedFeature] - if surrogates are found
/// 
/// # Arguments
/// 
/// - `value` (`RegexTranslationError`) - An [RegexTranslationError] from [regextranslator]
/// 
/// # Returns
/// 
/// - [XMLGeneratorError] - An [XMLGeneratorError]
///
impl From<RegexTranslationError> for XMLGeneratorError {
    fn from(value: RegexTranslationError) -> XMLGeneratorError {
        match value {
            RegexTranslationError::InvalidInput(str) => XMLGeneratorError::InvalidXSDError(str),
            RegexTranslationError::RegexError(str) => XMLGeneratorError::InvalidXSDError(str),
            RegexTranslationError::DataError(str) => XMLGeneratorError::RegexError(str),
            RegexTranslationError::UnicodeError(str) => XMLGeneratorError::RegexError(str),
            RegexTranslationError::SurrogatesError => {
                XMLGeneratorError::UnimplementedFeature("Regex Surrogates".to_string())
            }
        }
    }
}

/// Converts an `XMLError` from [xml-builder] into an [XMLGeneratorError::XMLBuilderError]
/// 
/// [xml-builder]: https://crates.io/crates/xml_builder
/// 
/// # Arguments
/// 
/// - `value` (`XMLError`) - An `XMLError` from [xml-builder]
/// 
/// # Returns
/// 
/// - [XMLGeneratorError] - An [XMLGeneratorError::XMLBuilderError]
/// 
impl From<XMLError> for XMLGeneratorError {
    fn from(value: XMLError) -> XMLGeneratorError {
        match value {
            XMLError::InsertError(str) => XMLGeneratorError::XMLBuilderError(str),
            XMLError::IOError(str) => XMLGeneratorError::XMLBuilderError(str),
        }
    }
}

/// Shortcut function to return an [XMLGeneratorError::UnimplementedFeature] error
/// 
/// # Arguments
/// 
/// - `name` (`&str`) - Name of the unimplemented feature
/// 
/// # Returns
/// 
/// - `Result<Type, XMLGeneratorError>` - A result object holding an [XMLGeneratorError::UnimplementedFeature] error
/// 
/// # Errors
/// 
/// - [XMLGeneratorError::UnimplementedFeature] - Always returned
/// 
pub(crate) fn unimplemented<Type>(name: &str) -> Result<Type, XMLGeneratorError> {
    Err(XMLGeneratorError::UnimplementedFeature(name.to_string()))
}
