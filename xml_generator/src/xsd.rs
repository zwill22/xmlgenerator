use crate::XMLGeneratorError;
use crate::data_type::DataType;
use crate::element::Element;
use crate::error::unimplemented;
use crate::namespaces::Namespaces;
use crate::generator::Generator;
use crate::schema_version::SchemaVersion;
use crate::tracker::RecursionTracker;
use regextranslator::RegexTranslator;
use std::slice::Iter;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};
use xsd_parser::Schemas;
use xsd_parser::models::schema::xs::SchemaContent;

pub(crate) struct XSD {
    version: SchemaVersion,
    namespaces: Namespaces,
    data_types: Vec<DataType>,
    elements: Vec<Element>,
}

impl XSD {
    pub(crate) fn new<'a>(
        generator: &mut Generator,
        translator: &RegexTranslator,
        schemas: &Schemas,
    ) -> Result<XSD, XMLGeneratorError> {
        let mut xsd = XSD {
            version: SchemaVersion::new(schemas)?,
            namespaces: Namespaces::new(generator, schemas)?,
            data_types: vec![],
            elements: vec![],
        };

        for (_schema_id, schema_info) in schemas.schemas() {
            let schema = &schema_info.schema;
            for content in &schema.content {
                match content {
                    SchemaContent::Element(element) => {
                        let element =
                            Element::new(translator, &element, &xsd.namespaces, schema_info)?;
                        xsd.elements.push(element);
                    }
                    SchemaContent::Import(_) => {}
                    SchemaContent::Annotation(_) => {}
                    SchemaContent::SimpleType(simple) => {
                        let simple_type = DataType::simple_type(translator, simple)?;
                        xsd.data_types.push(simple_type);
                    }
                    SchemaContent::ComplexType(complex) => {
                        let complex_type = DataType::complex_type(
                            translator,
                            complex,
                            &xsd.namespaces,
                            schema_info,
                        )?;
                        xsd.data_types.push(complex_type);
                    }
                    _ => return unimplemented("Unimplemented schema content type"),
                }
            }
        }

        Ok(xsd)
    }

    pub(crate) fn apply_metadata_to(&self, element: &mut XMLElement) {
        if let Some(location) = &self.namespaces.get_default_namespace() {
            let name = "xmlns";
            element.add_attribute(name, location);
        }

        for (prefix, location) in self.namespaces.get_other_namespaces() {
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

    pub(crate) fn get_version(&self) -> Result<XMLVersion, XMLGeneratorError> {
        self.version.get_version()
    }

    pub(crate) fn elements(&self) -> Iter<'_, Element> {
        self.elements.iter()
    }

    pub(crate) fn types(&self) -> Iter<'_, DataType> {
        self.data_types.iter()
    }

    fn get_element(&self, field: &String) -> Option<&Element> {
        for element in self.elements() {
            if let Ok(name) = element.get_name() {
                if name.eq(field) {
                    return Some(element);
                }
            }
        }

        None
    }

    pub(crate) fn find_root(&self) -> Result<&Element, XMLGeneratorError> {
        if self.elements.is_empty() {
            return Err(XMLGeneratorError::NoElementsError);
        }

        let mut all_fields = vec![];
        let mut all_types = vec![];
        for generator in self.elements.iter() {
            generator.get_content(&mut all_fields, &mut all_types)?;
        }

        let mut dependent_elements = vec![];
        for field in all_fields {
            let structure = self.get_element(&field);
            if let Some(item) = structure {
                dependent_elements.push(item);
            }
        }

        let mut independent_elements = vec![];
        for element in self.elements() {
            if !dependent_elements.contains(&element) {
                independent_elements.push(element);
            }
        }

        if independent_elements.is_empty() {
            return Err(XMLGeneratorError::NoIndependentElementsError);
        }

        if independent_elements.len() > 1 {
            return Err(XMLGeneratorError::MultipleRootsError);
        }

        for generator in self.elements() {
            if independent_elements.contains(&generator) {
                return Ok(generator);
            }
        }

        unreachable!();
    }

    fn generate_root(
        &self,
        generator: &mut Generator,
        tracker: &mut RecursionTracker,
        root: &Element,
    ) -> Result<XMLElement, XMLGeneratorError> {
        let root_elements = root.generate(generator, tracker, self)?;

        if root_elements.len() > 1 {
            return Err(XMLGeneratorError::MultipleRootsError);
        }

        for mut root_element in root_elements {
            self.apply_metadata_to(&mut root_element);

            return Ok(root_element);
        }

        Err(XMLGeneratorError::TypeGenerationError(
            "No root elements generated".to_string(),
        ))
    }

    fn build_xml(&self, generator: &mut Generator) -> Result<XMLElement, XMLGeneratorError> {
        let root = self.find_root()?;

        let mut tracker = RecursionTracker::new();

        self.generate_root(generator, &mut tracker, root)
    }

    pub(crate) fn generate_xml(
        &self,
        generator: &mut Generator,
    ) -> Result<String, XMLGeneratorError> {
        let schema_version = self.get_version()?;

        let mut xml = XMLBuilder::new()
            .expand_empty_tags(true)
            .version(schema_version)
            .encoding("UTF-8".into())
            .build();

        let root_element = self.build_xml(generator)?;
        xml.set_root_element(root_element);

        let mut writer: Vec<u8> = Vec::new();

        match xml.generate(&mut writer) {
            Ok(_) => Ok(String::from_utf8(writer).expect("Invalid UTF-8 sequence")),
            Err(e) => Err(XMLGeneratorError::XMLBuilderError(e.to_string())),
        }
    }
}
