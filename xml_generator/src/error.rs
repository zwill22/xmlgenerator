//! Module containing the [XMLGeneratorError] struct
//!
//! All errors in the [crate] should raise one of these errors rather than panicing
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

impl From<AttrError> for XMLGeneratorError {
    fn from(err: AttrError) -> XMLGeneratorError {
        XMLGeneratorError::InvalidXSDError(err.to_string())
    }
}

impl From<IntersectError> for XMLGeneratorError {
    fn from(error: IntersectError) -> XMLGeneratorError {
        match error {
            IntersectError::Parse(error) => XMLGeneratorError::RegexError(error.to_string()),
            IntersectError::Error(error) => XMLGeneratorError::RegexError(error.to_string()),
        }
    }
}

impl From<XSDValidationError> for XMLGeneratorError {
    fn from(value: XSDValidationError) -> Self {
        match value {
            XSDValidationError::PathError => XMLGeneratorError::InvalidPathError("".to_string()),
            XSDValidationError::StringError => {
                XMLGeneratorError::XSDValidatorError("Cannot read path string".to_string())
            }
            XSDValidationError::OutputRedirectError(e) => {
                XMLGeneratorError::XSDValidatorError(e.to_string())
            }
            XSDValidationError::GenerateContextError => {
                XMLGeneratorError::XSDValidatorError("Cannot generate context".to_string())
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

impl From<RegexTranslationError> for XMLGeneratorError {
    fn from(value: RegexTranslationError) -> Self {
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

impl From<XMLError> for XMLGeneratorError {
    fn from(value: XMLError) -> Self {
        match value {
            XMLError::InsertError(str) => XMLGeneratorError::XMLBuilderError(str),
            XMLError::IOError(str) => XMLGeneratorError::XMLBuilderError(str),
        }
    }
}

pub(crate) fn unimplemented<Type>(name: &str) -> Result<Type, XMLGeneratorError> {
    Err(XMLGeneratorError::UnimplementedFeature(name.to_string()))
}
