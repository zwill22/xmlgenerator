use libxml2_rs::{
    xmlCleanupParser, xmlErrorPtr, xmlInitParser, xmlSchemaFree,
    xmlSchemaNewParserCtxt, xmlSchemaParse, xmlSchemaParserCtxtPtr, xmlSchemaPtr,
    xmlSchemaSetParserStructuredErrors,
};
use std::env::{current_dir, set_current_dir};
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::Path;

#[derive(Debug)]
pub enum XSDValidationError {
    PathError,
    StringError,
    ReadFileError,
    GenerateContextError,
    ParseError(String),
}

extern "C" fn structured_error_handler(user_data: *mut c_void, error: xmlErrorPtr) {
    if error.is_null() {
        return;
    }

    let line = unsafe { (*error).line };
    let column = unsafe { (*error).int2 };
    let result = unsafe { CStr::from_ptr((*error).message).to_str() };
    let message = match result {
        Ok(msg) => msg.to_string(),
        Err(_) => "".to_string(),
    };

    let context = unsafe { &mut *(user_data as *mut Vec<String>) };

    let error = format!("Error: Line {}, column {}: {}", line, column, message);

    context.push(error);
}

fn get_cstring(str: &str) -> Result<CString, XSDValidationError> {
    match CString::new(str.to_string()) {
        Ok(c) => Ok(c),
        Err(_) => Err(XSDValidationError::StringError),
    }
}

struct SchemaParserContext(xmlSchemaParserCtxtPtr);

impl SchemaParserContext {
    fn new(path: &Path) -> Result<Self, XSDValidationError> {
        let path_str = match path.to_str() {
            Some(x) => get_cstring(x)?,
            None => return Err(XSDValidationError::PathError),
        };

        let file_ptr = path_str.as_ptr() as *const c_char;

        let context_ptr = unsafe { xmlSchemaNewParserCtxt(file_ptr) };

        if context_ptr.is_null() {
            return Err(XSDValidationError::GenerateContextError);
        }

        Ok(Self(context_ptr))
    }
}

struct Schema(xmlSchemaPtr);

impl Drop for Schema {
    fn drop(&mut self) {
        unsafe { xmlSchemaFree(self.0) }
    }
}

fn parse_schema(parser_context: &SchemaParserContext) -> bool {
    let schema_ptr = unsafe { xmlSchemaParse(parser_context.0) };

    if schema_ptr.is_null() {
        return false;
    }

    let _schema = Schema(schema_ptr);

    true
}

fn setup_error_handler(errors: &mut Vec<String>, parser_context: &SchemaParserContext) {
    unsafe {
        xmlSchemaSetParserStructuredErrors(
            parser_context.0,
            Some(structured_error_handler),
            errors as *mut _ as *mut c_void,
        );
    };
}

pub struct XSDValidator {}

impl XSDValidator {
    pub fn new() -> XSDValidator {
        unsafe { xmlInitParser() };

        XSDValidator {}
    }

    fn validate_file(&self, path: &Path) -> Result<bool, XSDValidationError> {
        let mut errors: Vec<String> = Vec::new();

        let parser_context = SchemaParserContext::new(path)?;

        setup_error_handler(&mut errors, &parser_context);

        let valid = parse_schema(&parser_context);

        for error in &errors {
            if error.to_lowercase().contains("skipping") {
                eprintln!("{}", error);
            } else {
                return Err(XSDValidationError::ParseError(error.clone()));
            }
        }

        Ok(valid)
    }

    pub fn validate(&self, path: &Path) -> Result<bool, XSDValidationError> {
        let wd = current_dir().expect("current_dir() failed");
        let file_dir = path.parent().expect("Filepath has no parent");

        set_current_dir(file_dir).expect("set_current_dir() failed");

        let result = self.validate_file(path);

        set_current_dir(wd).expect("set_current_dir() failed");

        result
    }
}

impl Drop for XSDValidator {
    fn drop(&mut self) {
        unsafe { xmlCleanupParser() };
    }
}
