/// Error handler for the XSD validator [crate].
/// 
#[derive(Debug)]
pub enum XSDValidationError {
    /// Invalid input path
    PathError,
    /// Error occurred while interfacing with libxml2
    LibXML2InterfaceError(String),
    /// Error redirecting libxml2 warnings
    OutputRedirectError(String),
    /// Error parsing XSD file
    ParseError(String),
    /// Error reading file with [file_to_string]
    ReadFileError,
    /// Recursion encountered in XSD
    XSDRecursionError,
}
