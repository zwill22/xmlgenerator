use crate::error::XMLGeneratorError;
use std::env::{current_dir, set_current_dir};
use std::path::Path;
use xsd_parser::{Parser, Schemas};

pub(crate) fn build_schemas(path: &Path) -> Result<Schemas, XMLGeneratorError> {
    let wd = current_dir().expect("current_dir() failed");
    let file_dir = match path.parent() {
        Some(dir) => dir,
        None => {
            return Err(XMLGeneratorError::InvalidPathError(
                path.to_str().unwrap().to_string(),
            ));
        }
    };

    set_current_dir(file_dir).expect("set_current_dir() failed");

    let schemas = Parser::new()
        .with_default_resolver()
        .with_default_namespaces()
        .resolve_includes(true)
        .add_schema_from_file(path);

    set_current_dir(wd).expect("set_current_dir() failed");

    match schemas {
        Ok(schema) => Ok(schema.finish()),
        Err(error) => Err(XMLGeneratorError::XSDParserError(error.to_string())),
    }
}
