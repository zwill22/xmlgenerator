use xsdvalidator::XSDValidationError;

/// XML generator error
///
/// Struct which manages errors in the XMLGenerator crate
#[derive(Debug)]
pub enum XMLGeneratorError {
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
    /// Generator Error
    TypeGenerationError(String),
}

impl From<XSDValidationError> for XMLGeneratorError {
    fn from(value: XSDValidationError) -> Self {
        match value {
            XSDValidationError::PathError => {
                XMLGeneratorError::XSDValidatorError("Cannot read path".to_string())
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
