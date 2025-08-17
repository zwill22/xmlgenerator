use crate::error::XMLGeneratorError;
use xsd_parser::pipeline::parser::resolver::FileResolver;
use xsd_parser::{Parser, Schemas};

pub(crate) fn generate_schema(string: &String) -> Result<Schemas, XMLGeneratorError> {
    let schemas = Parser::new()
        .with_resolver(FileResolver::new())
        .with_default_namespaces()
        .add_schema_from_str(string);

    if let Err(err) = schemas {
        return Err(XMLGeneratorError::XSDParserError(err.to_string()));
    }

    Ok(schemas.unwrap().finish())
}
