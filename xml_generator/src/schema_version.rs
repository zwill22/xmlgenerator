use crate::error::XMLGeneratorError;
use std::str::from_utf8;
use xml_builder::XMLVersion;
use xsd_parser::Schemas;

fn check_namespace(ns: &String, schemas: &Schemas) -> Result<(), XMLGeneratorError> {
    for (_ns_id, ns_info) in schemas.namespaces() {
        if let Some(namespace) = &ns_info.namespace {
            let ns_name = match from_utf8(namespace) {
                Ok(name) => name,
                Err(e) => return Err(XMLGeneratorError::DataTypesFormatError(e.to_string())),
            };
            if ns_name.eq(ns) {
                return Ok(());
            }
        }
    }

    Err(XMLGeneratorError::InvalidXSDError(format!(
        "Invalid namespace in schema: {}",
        ns
    )))
}

pub(crate) fn fetch_schema_version(schemas: &Schemas) -> Result<XMLVersion, XMLGeneratorError> {
    for (_schema_id, schema_info) in schemas.schemas() {
        let schema = &schema_info.schema;
        if let Some(ns) = &schema.target_namespace {
            check_namespace(ns, schemas)?;
        }

        let version = &schema.version;
        return match version {
            None => Ok(XMLVersion::XML1_0),
            Some(v) => {
                if v.as_str() == "1.0" {
                    Ok(XMLVersion::XML1_0)
                } else if v.as_str() == "1.1" {
                    Ok(XMLVersion::XML1_1)
                } else {
                    Err(XMLGeneratorError::InvalidXSDVersionError(v.clone()))
                }
            }
        };
    }

    Ok(XMLVersion::XML1_0)
}
