use xsdvalidator::XSDValidationError;

/// XML generator error
///
/// Struct which manages errors in the XMLGenerator crate
#[derive(Debug)]
pub enum XMLGeneratorError {
    /// Error from XSDvalidator module
    XSDValidatorError(String),
    /// Invalid XSD
    InvalidXSDError(String),
    ///  Error finding matching data type
    DataTypeError(String),
    /// Error parsing the input XSD file contents
    XSDParserError(String),
    /// Datatypes are in an invalid format
    DataTypesFormatError(String),
    /// Error generating the output XML structure
    XMLBuilderError(String),
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
