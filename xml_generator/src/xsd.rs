use crate::XMLGeneratorError;
use crate::element::Element;
use crate::error::unimplemented;
use crate::find_root::find_root_element;
use crate::namespaces::Namespaces;
use crate::schema_version::SchemaVersion;
use crate::tracker::RecursionTracker;
use crate::r#type::Type;
use regextranslator::RegexTranslator;
use crate::data_type::DataType;
use std::slice::Iter;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};
use xsd_parser::Schemas;
use xsd_parser::models::schema::xs::SchemaContent;

pub(crate) struct XSD {
    version: SchemaVersion,
    data_types: Vec<DataType>,
    elements: Vec<Element>,
    namespaces: Namespaces,
}

impl XSD {
    pub(crate) fn new(
        schemas: &Schemas,
        translator: &RegexTranslator,
    ) -> Result<Self, XMLGeneratorError> {
        let v = SchemaVersion::new(schemas)?;
        let ns = Namespaces::new(schemas)?;

        let mut types = vec![];
        let mut elements = vec![];
        for (_schema_id, schema_info) in schemas.schemas() {
            let schema = &schema_info.schema;
            for content in &schema.content {
                match content {
                    SchemaContent::Element(element) => {
                        let element = Element::new(&element, &translator, schema_info, &ns)?;
                        elements.push(element);
                    }
                    SchemaContent::Import(_) => {}
                    SchemaContent::Annotation(_) => {}
                    SchemaContent::SimpleType(simple) => {
                        let simple_type = DataType::simple_type(simple, translator)?;
                        data_types.push(simple_type);
                    }
                    SchemaContent::ComplexType(complex) => {
                        let complex_type =
                            DataType::complex_type(complex, translator, schema_info, &namespaces)?;
                        data_types.push(complex_type);
                    }
                    _ => return unimplemented("Unimplemented schema content type"),
                }
            }
        }

        let data = XSD {
            version: v,
            types: types,
            elements: elements,
            namespaces: ns,
            data_types,
        };

        Ok(data)
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

    pub(crate) fn find_root(&self) -> Result<&Element, XMLGeneratorError> {
        find_root_element(&self.elements)
    }

    pub(crate) fn elements(&self) -> Iter<'_, Element> {
        self.elements.iter()
    }

    pub(crate) fn types(&self) -> Iter<'_, DataType> {
        self.data_types.iter()
    }

    fn generate_root(
        &self,
        root: &Element,
        tracker: &mut RecursionTracker,
    ) -> Result<XMLElement, XMLGeneratorError> {
        let root_elements = root.generate(tracker, self)?;

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

    fn build_xml(&self) -> Result<XMLElement, XMLGeneratorError> {
        let root = self.find_root()?;

        let mut tracker = RecursionTracker::new();

        self.generate_root(root, &mut tracker)
    }

    pub(crate) fn generate_xml(&self) -> Result<String, XMLGeneratorError> {
        let schema_version = self.get_version()?;

        let mut xml = XMLBuilder::new()
            .expand_empty_tags(true)
            .version(schema_version)
            .encoding("UTF-8".into())
            .build();

        let root_element = self.build_xml()?;
        xml.set_root_element(root_element);

        let mut writer: Vec<u8> = Vec::new();

        match xml.generate(&mut writer) {
            Ok(_) => Ok(String::from_utf8(writer).expect("Invalid UTF-8 sequence")),
            Err(e) => Err(XMLGeneratorError::XMLBuilderError(e.to_string())),
        }
    }
}
