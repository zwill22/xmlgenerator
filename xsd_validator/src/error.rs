#[derive(Debug)]
pub enum XSDValidationError {
    PathError,
    StringError,
    OutputRedirectError(String),
    GenerateContextError,
    ParseError(String),
}
