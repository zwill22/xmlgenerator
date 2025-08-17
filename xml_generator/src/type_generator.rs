use crate::attribute::AttributeInfo;
use crate::element_generator::ElementGenerator;
use crate::error::XMLGeneratorError;
use crate::generate::generate;
use crate::group::GroupInfo;
use std::ops::Deref;
use xml_builder::XMLElement;

pub(crate) struct TypeGenerator {
    pub(crate) name: String,
    pub(crate) type_info: Vec<String>,
    pub(crate) elements: Vec<ElementGenerator>,
    pub(crate) groups: Vec<GroupInfo>,
    pub(crate) attributes: Vec<AttributeInfo>,
    pub(crate) min: u32,
    pub(crate) max: Option<u32>,
}

impl TypeGenerator {
    pub(crate) fn generate(
        &self,
        xml_element: &mut XMLElement,
        data_types: &Vec<TypeGenerator>,
        elements: &Vec<ElementGenerator>,
    ) -> Result<(), XMLGeneratorError> {
        for type_information in &self.type_info {
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

            let output = generate(type_information);
            if output.is_some() {
                let result = xml_element.add_text(output.unwrap());
                if let Err(err) = result {
                    return Err(XMLGeneratorError::XMLBuilderError(err.to_string()));
                }
            }
        }

        for element in self.elements.iter() {
            let child = element.generate(data_types, elements)?;

            let result = xml_element.add_child(child);
            if result.is_err() {
                return Err(XMLGeneratorError::XMLBuilderError(
                    "Unable to add child to element".to_string(),
                ));
            }
        }

        for group in self.groups.iter() {
            for element in group.elements.iter() {
                let child = element.generate(data_types, elements)?;

                let result = xml_element.add_child(child);
                if result.is_err() {
                    return Err(XMLGeneratorError::XMLBuilderError(
                        "Unable to add group child to element".to_string(),
                    ));
                }
            }
        }

        for attribute in self.attributes.iter() {
            println!("Attribute: {}", attribute.name);
        }

        Ok(())
    }

    pub(crate) fn new() -> Self {
        TypeGenerator {
            name: String::new(),
            type_info: vec![],
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

        if !self.groups.deref().into_iter().eq(&other.groups) {
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
