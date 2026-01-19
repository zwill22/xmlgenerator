use crate::error::XMLGeneratorError;
use std::collections::{HashMap, HashSet};
use std::str::from_utf8;
use xml_builder::{XMLElement, XMLVersion};
use xsd_parser::Schemas;
use xsd_parser::models::schema::xs::{Import, SchemaContent};
use xsd_parser::models::schema::{NamespaceInfo, SchemaInfo};

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

#[derive(Default)]
struct Namespaces {
    default_namespace: Option<String>,
    target_namespace: Option<String>,
    locations: HashMap<String, String>,
    other_namespaces: HashMap<String, String>,
}

impl Namespaces {
    fn new() -> Namespaces {
        Self::default()
    }

    fn add_default_namespace(&mut self, ns: String) -> Result<(), XMLGeneratorError> {
        match self.default_namespace {
            None => self.default_namespace = Some(ns),
            Some(_) => {
                self.other_namespaces.insert(ns.to_string(), ns);
            }
        }

        Ok(())
    }

    fn add_target_namespace(&mut self, ns: String) -> Result<(), XMLGeneratorError> {
        match self.target_namespace {
            None => self.target_namespace = Some(ns),
            Some(_) => {
                self.other_namespaces.insert(ns.to_string(), ns);
            }
        }

        Ok(())
    }

    fn set_location(&mut self, import: &Import, location: &String) {
        match &import.namespace {
            None => {
                self.locations
                    .insert(location.to_string(), location.to_string());
            }
            Some(ns) => {
                self.locations.insert(ns.to_string(), location.to_string());
            }
        }
    }

    fn get_imports(&mut self, import: &Import) {
        match &import.schema_location {
            None => {}
            Some(location) => {
                self.set_location(import, location);
            }
        };
    }

    fn get_info(&mut self, ns_info: &NamespaceInfo) -> Result<(), XMLGeneratorError> {
        if ns_info.module_name.is_some() {
            return Err(XMLGeneratorError::UnimplementedFeature(
                "module_name".to_string(),
            ));
        }

        if let Some(namespace) = &ns_info.namespace {
            let ns = namespace.to_string();

            match &ns_info.prefix {
                None => {
                    self.add_default_namespace(ns)?;
                }
                Some(prefix) => {
                    self.other_namespaces.insert(prefix.to_string(), ns);
                }
            }
        }

        Ok(())
    }

    fn get_schema_info(
        &mut self,
        schemas: &Schemas,
        schema_info: &SchemaInfo,
    ) -> Result<(), XMLGeneratorError> {
        let schema = &schema_info.schema;

        if let Some(ns) = &schema.target_namespace {
            check_namespace(ns, schemas)?;
            self.add_target_namespace(ns.to_string())?;
        }

        for content in &schema.content {
            if let SchemaContent::Import(import) = content {
                self.get_imports(import)
            }
        }

        Ok(())
    }
}

fn get_namespaces(schemas: &Schemas) -> Result<Namespaces, XMLGeneratorError> {
    let mut namespaces = Namespaces::new();

    for (_ns_id, ns_info) in schemas.namespaces() {
        namespaces.get_info(ns_info)?;
    }

    for (_schema_id, schema_info) in schemas.schemas() {
        namespaces.get_schema_info(schemas, schema_info)?;
    }

    Ok(namespaces)
}

pub(crate) struct SchemaMetadata {
    version: String,
    namespaces: Namespaces,
}

impl SchemaMetadata {
    fn new(schemas: &Schemas) -> Result<Self, XMLGeneratorError> {
        let version = fetch_schema_version(schemas)?;

        let namespaces = get_namespaces(schemas)?;

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

    pub(crate) fn apply_to(&self, element: &mut XMLElement) -> Result<(), XMLGeneratorError> {
        let mut namespaces = HashSet::new();

        match &self.namespaces.default_namespace {
            Some(ns) => {
                element.add_attribute("xmlns", ns.as_str());
                namespaces.insert(ns);

                if let Some(target) = &self.namespaces.target_namespace
                    && !namespaces.contains(target)
                {
                    element.add_attribute("targetNamespace", target.as_str());
                }
            }
            None => match &self.namespaces.target_namespace {
                None => {}
                Some(target) => {
                    element.add_attribute("xmlns", target.as_str());
                    namespaces.insert(target);
                }
            },
        }

        for (prefix, namespace) in &self.namespaces.other_namespaces {
            if prefix == "xs" {
                let value = namespace.to_string() + "-instance";
                element.add_attribute("xmlns:xsi", value.as_str());
            } else if prefix == "xml" {
                // ignore
            } else if !namespaces.contains(namespace) {
                let name = "xmlns:".to_string() + prefix;
                element.add_attribute(name.as_str(), namespace.as_str());
                namespaces.insert(namespace);
            }
        }

        Ok(())
    }
}

pub(crate) fn get_metadata(schemas: &Schemas) -> Result<SchemaMetadata, XMLGeneratorError> {
    SchemaMetadata::new(schemas)
}
