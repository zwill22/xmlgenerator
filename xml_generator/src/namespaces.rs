use crate::XMLGeneratorError;
use crate::generator::Generator;
use crate::pattern::Pattern;
use crate::xsd_type::XsdType;
use std::collections::HashMap;
use std::str::from_utf8;
use xsd_parser::Schemas;
use xsd_parser::models::schema::xs::{Import, SchemaContent};
use xsd_parser::models::schema::{NamespaceInfo, SchemaInfo};

fn check_namespace_exists(ns: &String, schemas: &Schemas) -> Result<(), XMLGeneratorError> {
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

fn check_namespace_is_valid(ns: &str) -> bool {
    if ns.eq("xs") {
        return false;
    }

    XsdType::from("name").validate(ns)
}

#[derive(Default)]
pub(crate) struct Namespaces {
    root_namespace: Option<String>,
    default_namespace: Option<String>,
    target_namespace: Option<String>,
    locations: HashMap<String, String>,
    other_namespaces: HashMap<String, String>,
}

impl Namespaces {
    pub(crate) fn new(
        generator: &mut Generator,
        schemas: &Schemas,
    ) -> Result<Namespaces, XMLGeneratorError> {
        let mut namespaces = Namespaces::default();

        for (_ns_id, ns_info) in schemas.namespaces() {
            namespaces.get_info(generator, ns_info, schemas)?;
        }

        let mut root = true;
        for (_, schema_info) in schemas.schemas() {
            namespaces.get_schema_info(generator, schemas, schema_info, root)?;
            root = false;
        }

        Ok(namespaces)
    }

    fn generate_valid_name(
        &mut self,
        generator: &mut Generator,
        schemas: &Schemas,
    ) -> Result<String, XMLGeneratorError> {
        let pattern = Pattern::from(r"[a-z]{2}");

        match generator.generate_regex(&pattern) {
            Some(output) => match check_namespace_exists(&output, schemas) {
                Ok(_) => self.generate_valid_name(generator, schemas),
                Err(_) => Ok(output),
            },
            None => Err(XMLGeneratorError::RegexError(
                "Unable to generate regex".to_string(),
            )),
        }
    }

    fn add_default_namespace(
        &mut self,
        generator: &mut Generator,
        ns: String,
        schemas: &Schemas,
    ) -> Result<(), XMLGeneratorError> {
        match self.default_namespace {
            None => {
                if !check_namespace_is_valid(&ns) {
                    let valid_name = self.generate_valid_name(generator, schemas)?;
                    self.root_namespace = Some(valid_name);
                }

                self.default_namespace = Some(ns);
            }
            Some(_) => {
                if check_namespace_is_valid(&ns) {
                    self.other_namespaces.insert(ns.to_string(), ns);
                } else {
                    let valid_name = self.generate_valid_name(generator, schemas)?;
                    self.other_namespaces.insert(valid_name, ns);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn find(&self, location: &str) -> Option<&String> {
        if let Some(default_ns) = &self.default_namespace
            && default_ns.eq(location)
        {
            // If namespace is default namespace, no need to include prefix
            // unless it has been aliased
            return match &self.root_namespace {
                Some(root_ns) => Some(root_ns),
                None => None,
            };
        }

        for (k, v) in self.other_namespaces.iter() {
            if v.eq(location) {
                return Some(k);
            }
        }

        None
    }

    fn set_target_namespace(
        &mut self,
        generator: &mut Generator,
        schemas: &Schemas,
        target_location: String,
    ) -> Result<(), XMLGeneratorError> {
        // A target namespace is provided but does not match any given namespace
        // There are two options here:
        // 1. Throw an error, do not accept an unnamed target namespace (easy, but may not be standard)
        // 2. Assign target namespace a random name and prefix all elements with this prefix
        // TODO Consider the above
        match self.find(&target_location) {
            None => Err(XMLGeneratorError::DataTypesFormatError(
                "No target namespace found.".to_string(),
            )),
            Some(ns) => {
                if ns.eq("xs") {
                    let valid_name = self.generate_valid_name(generator, schemas)?;
                    self.target_namespace = Some(valid_name.clone());
                    self.other_namespaces.insert(valid_name, target_location);
                    return Ok(());
                }
                self.target_namespace = Some(ns.clone());
                Ok(())
            }
        }
    }

    fn check_target_namespace(&self, ns: String, target: &String) -> Result<(), XMLGeneratorError> {
        if let Some(new_target) = self.find(&ns)
            && new_target.eq(target)
        {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Multiple target namespaces found.".to_string(),
            ));
        }

        Ok(())
    }

    fn add_target_namespace(
        &mut self,
        generator: &mut Generator,
        schemas: &Schemas,
        ns: String,
    ) -> Result<(), XMLGeneratorError> {
        if let Some(default_ns) = &self.default_namespace
            && default_ns.eq(&ns)
        {
            return Ok(());
        }

        match &self.target_namespace {
            None => self.set_target_namespace(generator, schemas, ns)?,
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

    fn get_info(
        &mut self,
        generator: &mut Generator,
        ns_info: &NamespaceInfo,
        schemas: &Schemas,
    ) -> Result<(), XMLGeneratorError> {
        if ns_info.module_name.is_some() {
            return Err(XMLGeneratorError::UnimplementedFeature(
                "module_name".to_string(),
            ));
        }

        if let Some(namespace) = &ns_info.namespace {
            let ns = namespace.to_string();

            match &ns_info.prefix {
                None => {
                    self.add_default_namespace(generator, ns, schemas)?;
                }
                Some(prefix) => {
                    let prefix_str = prefix.to_string();
                    self.other_namespaces.insert(prefix_str, ns);
                }
            }
        }

        Ok(())
    }

    fn get_schema_info(
        &mut self,
        generator: &mut Generator,
        schemas: &Schemas,
        schema_info: &SchemaInfo,
        root: bool,
    ) -> Result<(), XMLGeneratorError> {
        let schema = &schema_info.schema;

        if let Some(ns) = &schema.target_namespace
            && root
        {
            check_namespace_exists(ns, schemas)?;
            self.add_target_namespace(generator, schemas, ns.to_string())?;
        }

        for content in &schema.content {
            if let SchemaContent::Import(import) = content {
                self.get_imports(import)
            }
        }

        generator.set_qualification(schema_info);

        Ok(())
    }

    pub(crate) fn get_default_namespace(&self) -> Option<String> {
        self.default_namespace.clone()
    }

    pub(crate) fn get_root_namespace(&self) -> Option<String> {
        self.root_namespace.clone()
    }

    pub(crate) fn get_other_namespaces(&self) -> &HashMap<String, String> {
        &self.other_namespaces
    }
}
