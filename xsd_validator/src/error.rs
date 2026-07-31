#[derive(Debug)]
pub enum XSDValidationError {
    PathError,
    LibXML2InterfaceError(String),
    OutputRedirectError(String),
    ParseError(String),
    ReadFileError,
    XSDRecursionError,
}
