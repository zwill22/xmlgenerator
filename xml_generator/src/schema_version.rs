use crate::error::XMLGeneratorError;
use xml_builder::XMLVersion;
use xsd_parser::Schemas;

pub(crate) fn fetch_schema_version(schemas: &Schemas) -> Result<XMLVersion, XMLGeneratorError> {
    for (_schema_id, schema) in schemas.schemas() {
        if let Some(ns) = &schema.target_namespace {
            panic!("Target namespace {} is not supported", ns);
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
                    Err(XMLGeneratorError::DataTypesFormatError(
                        "Invalid schema version".parse().unwrap(),
                    ))
                }
            }
        };
    }

    Ok(XMLVersion::XML1_0)
}
