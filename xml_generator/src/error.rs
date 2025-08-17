/// XML generator error
///
/// Struct which manages errors in the XMLGenerator crate
#[derive(Debug)]
pub enum XMLGeneratorError {
    ///  Error finding matching data type
    DataTypeError(String),
    /// Error parsing the input XSD file contents
    XSDParserError(String),
    /// Datatypes are in an invalid format
    DataTypesFormatError(String),
    /// Error generating the output XML structure
    XMLBuilderError(String),
}
