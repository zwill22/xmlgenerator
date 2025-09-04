use crate::error::XMLGeneratorError;
use std::path::Path;
use xsd_parser::pipeline::parser::resolver::FileResolver;
use xsd_parser::{Parser, Schemas};

pub(crate) fn generate_schema(path: &Path) -> Result<Schemas, XMLGeneratorError> {
    let schemas = Parser::new()
        .with_resolver(FileResolver::new())
        .with_default_namespaces()
        .add_schema_from_file(path);

    match schemas {
        Ok(schema) => Ok(schema.finish()),
        Err(error) => Err(XMLGeneratorError::XSDParserError(error.to_string())),
    }
}
