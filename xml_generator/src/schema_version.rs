use crate::error::XMLGeneratorError;
use xml_builder::XMLVersion;
use xsd_parser::Schemas;

pub(crate) struct SchemaVersion {
    version: String,
}

impl SchemaVersion {
    pub(crate) fn new(schemas: &Schemas) -> Result<Self, XMLGeneratorError> {
        let mut version = None;
        for (_, schema_info) in schemas.schemas() {
            let schema = &schema_info.schema;

            let version_string = match &schema.version {
                None => "1.0".to_string(),
                Some(v) => {
                    if v.as_str() == "1.0" || v.as_str() == "1.1" {
                        v.to_string()
                    } else {
                        return Err(XMLGeneratorError::InvalidXSDVersionError(v.to_string()));
                    }
                }
            };

            match &version {
                Some(v) => {
                    if v != &version_string {
                        return Err(XMLGeneratorError::InvalidXSDVersionError(
                            "Version mismatch".to_string(),
                        ));
                    }
                }
                None => {
                    version = Some(version_string);
                }
            }
        }

        let out = match version {
            Some(v) => SchemaVersion { version: v },
            None => SchemaVersion {
                version: "1.0".to_string(),
            },
        };

        Ok(out)
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
}
