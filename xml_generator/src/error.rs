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
    /// No independent elements in XSD
    NoIndependentElementsError,
    /// Multiple Roots elements in XSD
    MultipleRootsError,
    /// Generator Error
    TypeGenerationError(String),
    /// Regex Compilation Error
    RegexError(String),
    /// Unimplemented Feature
    UnimplementedFeature(String),
}

impl From<XSDValidationError> for XMLGeneratorError {
    fn from(value: XSDValidationError) -> Self {
        match value {
            XSDValidationError::PathError => {
                XMLGeneratorError::InvalidPathError("".to_string())
            }
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
            RegexTranslationError::FileReadError(str) => XMLGeneratorError::InvalidXSDError(str),
            RegexTranslationError::DataError(str) => XMLGeneratorError::RegexError(str),
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
