use crate::error::XSDValidationError;
use crate::schema::Schema;
use libxml2_rs::{
    xmlErrorPtr, xmlSchemaNewParserCtxt, xmlSchemaParse, xmlSchemaParserCtxtPtr,
    xmlSchemaSetParserStructuredErrors,
};
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::Path;

extern "C" fn structured_error_handler(user_data: *mut c_void, error: xmlErrorPtr) {
    if error.is_null() {
        return;
    }

    let line = unsafe { (*error).line };
    let column = unsafe { (*error).int2 };
    let result = unsafe { CStr::from_ptr((*error).message).to_str() };
    let message = match result {
        Ok(msg) => msg.to_string(),
        Err(_) => "Unknown Error".to_string(),
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

pub(crate) struct Parser(xmlSchemaParserCtxtPtr);

impl Parser {
    pub(crate) fn new(path: &Path) -> Result<Self, XSDValidationError> {
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

    pub(crate) fn setup_error_handler(&self, errors: &mut Vec<String>) {
        unsafe {
            xmlSchemaSetParserStructuredErrors(
                self.0,
                Some(structured_error_handler),
                errors as *mut _ as *mut c_void,
            );
        };
    }

    pub(crate) fn parse(&self) -> bool {
        let schema_ptr = unsafe { xmlSchemaParse(self.0) };

        if schema_ptr.is_null() {
            return false;
        }

        let _schema = Schema::new(schema_ptr);

        true
    }
}
