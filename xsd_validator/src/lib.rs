use libc::{STDERR_FILENO, c_int, close, dup, dup2, pipe};
use libxml2_rs::{
    xmlCleanupParser, xmlErrorPtr, xmlInitParser, xmlSchemaFree, xmlSchemaNewParserCtxt,
    xmlSchemaParse, xmlSchemaParserCtxtPtr, xmlSchemaPtr, xmlSchemaSetParserStructuredErrors,
};
use std::env::{current_dir, set_current_dir};
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::Path;

#[derive(Debug)]
pub enum XSDValidationError {
    PathError,
    StringError,
    OutputRedirectError(String),
    GenerateContextError,
    ParseError(String),
}

struct WarningHandler {
    saved_stderr: Option<c_int>,
    pipe_fd: Option<[c_int; 2]>,
    pipe_read: usize,
    pipe_write: usize,
}

impl WarningHandler {
    fn new() -> WarningHandler {
        WarningHandler {
            saved_stderr: None,
            pipe_fd: None,
            pipe_read: 0,
            pipe_write: 1,
        }
    }

    fn redirect(&mut self) -> Result<(), XSDValidationError> {
        let saved_stderr = unsafe { dup(STDERR_FILENO) };

        if saved_stderr == -1 {
            return Err(XSDValidationError::OutputRedirectError(
                "cannot duplicate stderr".to_string(),
            ));
        }

        self.saved_stderr = Some(saved_stderr);

        let mut pipe_fd: [c_int; 2] = [-1; 2];

        if unsafe { pipe(&mut pipe_fd[0]) } == -1 {
            return Err(XSDValidationError::OutputRedirectError(
                "cannot create pipe".to_string(),
            ));
        }

        // redirect stderr to pipe/log_file
        if unsafe { dup2(pipe_fd[self.pipe_write], STDERR_FILENO) } == -1 {
            return Err(XSDValidationError::OutputRedirectError(
                "cannot redirect stderr to pipe".to_string(),
            ));
        }

        self.pipe_fd = Some(pipe_fd);

        Ok(())
    }

    fn restore_stderr(&mut self) {
        if let Some(val) = self.saved_stderr {
            unsafe {
                dup2(val, STDERR_FILENO);
                close(val);
            }
        }
    }

    fn close_pipe(&mut self) {
        if let Some(pipe) = self.pipe_fd {
            unsafe {
                close(pipe[self.pipe_read]);

                close(pipe[self.pipe_write]);
            }
        }
    }

    fn restore(&mut self) {
        self.restore_stderr();
        self.close_pipe();
    }
}

impl Drop for WarningHandler {
    fn drop(&mut self) {
        self.restore();
    }
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

pub struct XSDValidator {
    print_warnings: bool,
}

impl XSDValidator {
    pub fn new(print_warnings: bool) -> XSDValidator {
        unsafe { xmlInitParser() };

        XSDValidator { print_warnings }
    }

    fn validate_file(&self, path: &Path) -> Result<bool, XSDValidationError> {
        let mut errors: Vec<String> = Vec::new();

        let parser_context = SchemaParserContext::new(path)?;

        setup_error_handler(&mut errors, &parser_context);

        let mut warning_handler = WarningHandler::new();

        if !self.print_warnings {
            warning_handler.redirect()?;
        }

        let valid = parse_schema(&parser_context);

        warning_handler.restore();

        for error in &errors {
            if error.to_lowercase().contains("skipping") {
                if self.print_warnings {
                    eprintln!("{}", error);
                }
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
