use crate::XMLGeneratorError;
use crate::data_type::DataType;
use crate::element::Element;
use crate::error::unimplemented;
use crate::generator::Generator;
use crate::namespaces::Namespaces;
use crate::schema_version::SchemaVersion;
use crate::schemas::SchemaData;
use crate::special::replace_specials;
use crate::whitespace::WhiteSpace;
use regextranslator::RegexTranslator;
use std::slice::Iter;
use xml_builder::{XML, XMLBuilder, XMLElement, XMLVersion};
use xsd_parser::models::schema::xs::SchemaContent;

pub(crate) struct Xsd {
    version: SchemaVersion,
    namespaces: Namespaces,
    data_types: Vec<DataType>,
    elements: Vec<Element>,
    encoding: Option<String>,
}

impl Xsd {
    pub(crate) fn new(
        generator: &mut Generator,
        translator: &RegexTranslator,
        schemas: &SchemaData,
    ) -> Result<Xsd, XMLGeneratorError> {
        let mut xsd = Xsd {
            version: schemas.get_version()?,
            namespaces: schemas.get_namespaces(generator)?,
            data_types: vec![],
            elements: vec![],
            encoding: schemas.get_encoding(),
        };

        for (_, schema_info) in schemas.schemas() {
            let schema = &schema_info.schema;
            for content in &schema.content {
                match content {
                    SchemaContent::Element(element) => {
                        let element =
                            Element::new(translator, element, &xsd.namespaces, schema_info)?;
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

    pub(crate) fn apply_metadata_to(
        &self,
        element: &mut XMLElement,
    ) -> Result<(), XMLGeneratorError> {
        if let Some(location) = &self.namespaces.get_default_namespace() {
            let loc = replace_specials(location, &WhiteSpace::Preserve)?;
            match self.namespaces.get_root_namespace() {
                None => {
                    element.add_attribute("xmlns", &loc);
                }
                Some(root_namespace) => {
                    let prefix = "xmlns:".to_string() + root_namespace.as_str();
                    element.add_attribute(&prefix, &loc);
                }
            }
        }

        for (prefix, location) in self.namespaces.get_other_namespaces() {
            if prefix == "xs" {
                let value = location.to_string() + "-instance";
                element.add_attribute("xmlns:xsi", &value);
            } else if prefix == "xml" {
                // ignore
            } else {
                let name = "xmlns:".to_string() + prefix;
                element.add_attribute(&name, location);
            }
        }

        Ok(())
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
            if let Ok(name) = element.get_name()
                && name.eq(field)
            {
                return Some(element);
            }
        }

        None
    }

    pub(crate) fn get_root_name(&self, name: &str) -> Result<String, XMLGeneratorError> {
        match self.namespaces.get_root_namespace() {
            None => Ok(name.to_string()),
            Some(root_ns) => {
                if name.contains(":") {
                    let names = name.split(":").collect::<Vec<&str>>();
                    if names.len() == 2 && names[0] == root_ns {
                        return Ok(name.to_string());
                    }

                    Err(XMLGeneratorError::DataTypeInformationError(
                        "multiple root namespaces".to_string(),
                    ))
                } else {
                    Ok(format!("{}:{}", root_ns, name))
                }
            }
        }
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

    fn build_xml(&self) -> Result<XML, XMLGeneratorError> {
        let schema_version = self.get_version()?;

        let mut xml_builder = XMLBuilder::new()
            .expand_empty_tags(true)
            .version(schema_version);

        if let Some(encoding) = &self.encoding {
            xml_builder = xml_builder.encoding(encoding.clone());
        } else {
            xml_builder = xml_builder.encoding("UTF-8".to_string());
        }

        Ok(xml_builder.build())
    }

    fn generate_root(
        &self,
        generator: &mut Generator,
        root: &Element,
    ) -> Result<XMLElement, XMLGeneratorError> {
        let root_elements = root.generate(generator, self)?;

        if root_elements.len() > 1 {
            return Err(XMLGeneratorError::MultipleRootsError);
        }

        match root_elements.into_iter().next() {
            Some(mut root_element) => {
                self.apply_metadata_to(&mut root_element)?;
                Ok(root_element)
            }
            None => Err(XMLGeneratorError::TypeGenerationError(
                "No root elements generated".to_string(),
            )),
        }
    }

    fn build_root(&self, generator: &mut Generator) -> Result<XMLElement, XMLGeneratorError> {
        let root = self.find_root()?;
        self.generate_root(generator, root)
    }

    pub(crate) fn generate_xml(
        &self,
        generator: &mut Generator,
    ) -> Result<String, XMLGeneratorError> {
        let mut xml = self.build_xml()?;

        let root_element = self.build_root(generator)?;
        xml.set_root_element(root_element);

        let mut writer: Vec<u8> = Vec::new();

        match xml.generate(&mut writer) {
            Ok(_) => Ok(String::from_utf8(writer).expect("Invalid UTF-8 sequence")),
            Err(e) => Err(XMLGeneratorError::XMLBuilderError(e.to_string())),
        }
    }
}
