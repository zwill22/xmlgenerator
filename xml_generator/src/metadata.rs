use crate::error::XMLGeneratorError;
use crate::namespaces::{Namespaces, check_namespace};
use std::collections::HashMap;
use xml_builder::{XMLElement, XMLVersion};
use xsd_parser::Schemas;

fn fetch_schema_version(schemas: &Schemas) -> Result<String, XMLGeneratorError> {
    if let Some((_schema_id, schema_info)) = schemas.schemas().next() {
        let schema = &schema_info.schema;
        if let Some(ns) = &schema.target_namespace {
            check_namespace(ns, schemas)?;
        }

        let version = &schema.version;
        return match version {
            None => Ok("1.0".to_string()),
            Some(v) => {
                if v.as_str() == "1.0" || v.as_str() == "1.1" {
                    Ok(v.to_string())
                } else {
                    Err(XMLGeneratorError::InvalidXSDVersionError(v.to_string()))
                }
            }
        };
    }

    Ok("1.0".to_string())
}

pub(crate) struct SchemaMetadata {
    version: String,
    namespaces: Namespaces,
}

impl SchemaMetadata {
    pub fn new(schemas: &Schemas) -> Result<Self, XMLGeneratorError> {
        let version = fetch_schema_version(schemas)?;

        let namespaces = Namespaces::new(schemas)?;

        let metadata = SchemaMetadata {
            version,
            namespaces,
        };

        Ok(metadata)
    }

    pub(crate) fn get_version(&self) -> Result<XMLVersion, XMLGeneratorError> {
        let version = self.version.as_str();

        if version == "1.0" {
            Ok(XMLVersion::XML1_0)
        } else if version == "1.1" {
            Ok(XMLVersion::XML1_1)
        } else {
            Err(XMLGeneratorError::InvalidXSDVersionError(
                version.to_string(),
            ))
        }
    }

    pub(crate) fn get_default_namespace(&self) -> Option<String> {
        self.namespaces.get_default_namespace()
    }

    pub(crate) fn get_other_namespaces(&self) -> &HashMap<String, String> {
        self.namespaces.get_other_namespaces()
    }

    pub(crate) fn get_target_namespace(&self) -> Option<String> {
        self.namespaces.get_target_namespace()
    }

    pub(crate) fn apply_to(&self, element: &mut XMLElement) {
        if let Some(location) = &self.get_default_namespace() {
            let name = "xmlns";
            element.add_attribute(name, location);
        }

        for (prefix, location) in self.get_other_namespaces() {
            if prefix == "xs" {
                let name = "xmlns:xsi";
                let value = location.to_string() + "-instance";
                element.add_attribute(name, value.as_str());
            } else if prefix == "xml" {
                // ignore
            } else {
                let name = "xmlns:".to_string() + prefix;
                element.add_attribute(name.as_str(), location);
            }
        }
    }
}
