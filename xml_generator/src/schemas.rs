use crate::error::XMLGeneratorError;
use crate::generator::Generator;
use crate::namespaces::Namespaces;
use crate::schema_version::SchemaVersion;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::env::{current_dir, set_current_dir};
use std::path::Path;
use xsd_parser::models::schema::{SchemaId, SchemaInfo};
use xsd_parser::{Parser, Schemas};

fn build_schemas(path: &Path) -> Result<Schemas, XMLGeneratorError> {
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

fn get_encoding(path: &Path) -> Result<Option<String>, XMLGeneratorError> {
    let mut xsd = Reader::from_file(path)?;
    xsd.config_mut().trim_text(true);

    let mut buf = Vec::new();
    loop {
        match xsd.read_event_into(&mut buf)? {
            Event::Start(_) => {}
            Event::End(_) => {}
            Event::Empty(_) => {}
            Event::Text(_) => {}
            Event::CData(_) => {}
            Event::Comment(_) => {}
            Event::Decl(declaration) => {
                return match declaration.encoding() {
                    None => Ok(None),
                    Some(encoding) => {
                        let value: String = str::from_utf8(encoding?.as_ref()).unwrap().to_string();
                        if value.is_empty() {
                            return Err(XMLGeneratorError::EncodingError);
                        }

                        Ok(Some(value))
                    }
                };
            }
            Event::PI(_) => {}
            Event::DocType(_) => {}
            Event::GeneralRef(_) => {}
            Event::Eof => break,
        }
    }

    // Assume default encoding
    Ok(Some("UTF-8".to_string()))
}

pub(crate) struct SchemaData {
    schemas: Schemas,
    encoding: Option<String>,
}

impl SchemaData {
    pub(crate) fn new(path: &Path) -> Result<Self, XMLGeneratorError> {
        let schemas = build_schemas(path)?;
        let encoding = get_encoding(path)?;

        Ok(Self { schemas, encoding })
    }

    pub(crate) fn get_version(&self) -> Result<SchemaVersion, XMLGeneratorError> {
        SchemaVersion::new(&self.schemas)
    }

    pub(crate) fn get_namespaces(
        &self,
        generator: &mut Generator,
    ) -> Result<Namespaces, XMLGeneratorError> {
        Namespaces::new(generator, &self.schemas)
    }

    pub(crate) fn schemas(&self) -> std::collections::btree_map::Iter<'_, SchemaId, SchemaInfo> {
        self.schemas.schemas()
    }

    pub(crate) fn get_encoding(&self) -> Option<String> {
        self.encoding.clone()
    }
}
