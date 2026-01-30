use crate::XMLGeneratorError;
use crate::element_generator::ElementGenerator;
use crate::fetch_elements::fetch_elements;
use crate::fetch_types::fetch_types;
use crate::find_root::find_root_element;
use crate::metadata::SchemaMetadata;
use crate::recursion_tracker::RecursionTracker;
use crate::type_generator::TypeGenerator;
use regextranslator::RegexTranslator;
use std::slice::Iter;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};
use xsd_parser::Schemas;

pub(crate) struct XSD {
    schema_metadata: SchemaMetadata,
    type_generators: Vec<TypeGenerator>,
    element_generators: Vec<ElementGenerator>,
}

impl XSD {
    pub(crate) fn new(
        schemas: &Schemas,
        translator: &RegexTranslator,
    ) -> Result<Self, XMLGeneratorError> {
        let metadata = SchemaMetadata::new(&schemas)?;
        let types = fetch_types(&schemas, translator, &metadata)?;
        let elements = fetch_elements(&schemas, translator, &metadata)?;

        let data = XSD {
            schema_metadata: metadata,
            type_generators: types,
            element_generators: elements,
        };

        Ok(data)
    }

    pub(crate) fn get_version(&self) -> Result<XMLVersion, XMLGeneratorError> {
        self.schema_metadata.get_version()
    }

    pub(crate) fn find_root(&self) -> Result<&ElementGenerator, XMLGeneratorError> {
        find_root_element(&self.element_generators)
    }

    pub(crate) fn apply_metadata_to(&self, element: &mut XMLElement) {
        self.schema_metadata.apply_to(element);
    }

    pub(crate) fn elements(&self) -> Iter<'_, ElementGenerator> {
        self.element_generators.iter()
    }

    pub(crate) fn types(&self) -> Iter<'_, TypeGenerator> {
        self.type_generators.iter()
    }

    fn build_xml(&self) -> Result<XMLElement, XMLGeneratorError> {
        let root = self.find_root()?;

        let mut tracker = RecursionTracker::new();

        let mut root_element = root.generate(&mut tracker, self)?;
        self.apply_metadata_to(&mut root_element);

        Ok(root_element)
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
