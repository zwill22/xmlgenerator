use crate::attribute_generator::AttributeGenerator;
use crate::element_generator::ElementGenerator;
use crate::error::XMLGeneratorError;
use crate::group_generator::GroupGenerator;
use crate::recursion_tracker::RecursionTracker;
use crate::type_info::TypeInfo;
use crate::xsd::XSD;
use std::ops::Deref;
use xml_builder::XMLElement;

pub(crate) struct TypeGenerator {
    pub(crate) name: String,
    pub(crate) type_info: Option<TypeInfo>,
    pub(crate) elements: Vec<ElementGenerator>,
    pub(crate) groups: Vec<GroupGenerator>,
    pub(crate) attributes: Vec<AttributeGenerator>,
    pub(crate) min: u32,
    pub(crate) max: Option<u32>,
}

impl TypeGenerator {
    pub(crate) fn generate(
        &self,
        xml_element: &mut XMLElement,
        data_tracker: &mut RecursionTracker,
        xsd: &XSD,
    ) -> Result<(), XMLGeneratorError> {
        if let Some(type_info) = &self.type_info {
            if !self.elements.is_empty() {
                return Err(XMLGeneratorError::DataTypesFormatError(
                    "Type includes type information and elements".to_string(),
                ));
            }

            if !self.groups.is_empty() {
                return Err(XMLGeneratorError::DataTypesFormatError(
                    "Type includes type information and groups".to_string(),
                ));
            }

            let output = type_info.generate();
            match output {
                None => {
                    return Err(XMLGeneratorError::TypeGenerationError(
                        "No output generated".to_string(),
                    ));
                }
                Some(value) => {
                    let result = xml_element.add_text(value);
                    if let Err(err) = result {
                        return Err(XMLGeneratorError::XMLBuilderError(err.to_string()));
                    }
                }
            }
        }

        for element in self.elements.iter() {
            let child = element.generate(data_tracker, xsd)?;

            xml_element.add_child(child)?;
        }

        for group in self.groups.iter() {
            group.generate(xml_element, data_tracker, xsd)?;
        }

        for attribute in self.attributes.iter() {
            attribute.generate(xml_element, xsd)?;
        }

        Ok(())
    }

    pub(crate) fn new() -> Self {
        TypeGenerator {
            name: String::new(),
            type_info: None,
            elements: vec![],
            groups: vec![],
            attributes: vec![],
            min: 1,
            max: None,
        }
    }
}

impl PartialEq for TypeGenerator {
    fn eq(&self, other: &Self) -> bool {
        if !self.name.eq(&other.name) {
            return false;
        }

        if !self.type_info.eq(&other.type_info) {
            return false;
        }

        if !self.elements.eq(&other.elements) {
            return false;
        }

        if !self.groups.deref().iter().eq(&other.groups) {
            return false;
        }
        if !self.attributes.eq(&other.attributes) {
            return false;
        }
        if self.min != other.min {
            return false;
        }

        if self.max != other.max {
            return false;
        }

        true
    }
}
