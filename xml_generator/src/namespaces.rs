use crate::XMLGeneratorError;
use std::collections::HashMap;
use std::str::from_utf8;
use xsd_parser::Schemas;
use xsd_parser::models::schema::xs::{Import, SchemaContent};
use xsd_parser::models::schema::{NamespaceInfo, SchemaInfo};

pub(crate) fn check_namespace(ns: &String, schemas: &Schemas) -> Result<(), XMLGeneratorError> {
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

#[derive(Default)]
pub(crate) struct Namespaces {
    default_namespace: Option<String>,
    target_namespace: Option<String>,
    locations: HashMap<String, String>,
    other_namespaces: HashMap<String, String>,
}

impl Namespaces {
    pub(crate) fn new(schemas: &Schemas) -> Result<Namespaces, XMLGeneratorError> {
        let mut namespaces = Namespaces::default();

        for (_ns_id, ns_info) in schemas.namespaces() {
            namespaces.get_info(ns_info)?;
        }

        let mut root = true;
        for (_, schema_info) in schemas.schemas() {
            namespaces.get_schema_info(schemas, schema_info, root)?;
            root = false;
        }

        Ok(namespaces)
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

    fn find_target_namespace(&self, location: &str) -> Option<String> {
        if let Some(default_ns) = &self.default_namespace
            && default_ns.eq(&location)
        {
            // Target namespace is the default namespace, no need to add prefix
            return None;
        }

        for (k, v) in self.other_namespaces.iter() {
            if v.eq(&location) {
                return Some(k.clone());
            }
        }

        None
    }

    fn set_target_namespace(&mut self, target_location: String) -> Result<(), XMLGeneratorError> {
        // A target namespace is provided but does not match any given namespace
        // There are two options here:
        // 1. Throw an error, do not accept an unnamed target namespace (easy, but may not be standard)
        // 2. Assign target namespace a random name and prefix all elements with this prefix
        // TODO Consider the above
        match self.find_target_namespace(&target_location) {
            None => Err(XMLGeneratorError::DataTypesFormatError(
                "No target namespace found.".to_string(),
            )),
            Some(ns) => {
                self.target_namespace = Some(ns);
                Ok(())
            }
        }
    }

    fn check_target_namespace(&self, ns: String, target: &String) -> Result<(), XMLGeneratorError> {
        if let Some(new_target) = self.find_target_namespace(&ns)
            && new_target != *target
        {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Multiple target namespaces found.".to_string(),
            ));
        }

        Ok(())
    }

    fn add_target_namespace(&mut self, ns: String) -> Result<(), XMLGeneratorError> {
        if let Some(default_ns) = &self.default_namespace
            && default_ns.eq(&ns)
        {
            return Ok(());
        }

        match &self.target_namespace {
            None => self.set_target_namespace(ns)?,
            Some(target) => self.check_target_namespace(ns, target)?,
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
        root: bool,
    ) -> Result<(), XMLGeneratorError> {
        let schema = &schema_info.schema;

        if let Some(ns) = &schema.target_namespace
            && root
        {
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

    pub(crate) fn get_default_namespace(&self) -> Option<String> {
        self.default_namespace.clone()
    }
    
    pub(crate) fn get_other_namespaces(&self) -> &HashMap<String, String> {
        &self.other_namespaces
    }
    
    pub(crate) fn get_target_namespace(&self) -> Option<String> {
        self.target_namespace.clone()
    }
}
