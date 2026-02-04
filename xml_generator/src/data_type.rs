use crate::attribute::Attribute;
use crate::element::Element;
use crate::error::{XMLGeneratorError, unimplemented};
use crate::group::Group;
use crate::namespaces::Namespaces;
use crate::tracker::RecursionTracker;
use crate::type_generator::TypeGenerator;
use crate::type_info::TypeInfo;
use crate::xsd::XSD;
use std::ops::Deref;
use xml_builder::XMLElement;
use xsd_parser::models::schema::SchemaInfo;
use xsd_parser::models::schema::xs::{ComplexBaseType, ComplexBaseTypeContent, SimpleBaseType};

pub(crate) struct DataType<'a> {
    generator: &'a TypeGenerator,
    name: String,
    type_info: Option<TypeInfo>,
    elements: Vec<Element<'a>>,
    groups: Vec<Group<'a>>,
    attributes: Vec<Attribute>,
}

impl<'a> DataType<'a> {
    fn new(generator: &'a TypeGenerator) -> DataType<'a> {
        Self {
            generator,
            name: "".to_string(),
            type_info: None,
            elements: vec![],
            groups: vec![],
            attributes: vec![],
        }
    }

    pub(crate) fn simple_type(
        generator: &'a TypeGenerator,
        simple: &SimpleBaseType,
    ) -> Result<Self, XMLGeneratorError> {
        let mut data_type = Self::new(generator);

        data_type.name = simple.name.clone().unwrap_or("".to_string());
        if data_type.name.is_empty() {
            return unimplemented("Empty type");
        }

        if simple.final_.is_some() {
            return unimplemented("Final");
        }

        let type_info = TypeInfo::new(generator, &simple.content)?;

        data_type.type_info = Some(type_info);

        Ok(data_type)
    }

    pub(crate) fn complex_type(
        generator: &'a TypeGenerator,
        complex: & ComplexBaseType,
        namespaces: &Namespaces,
        schema_info: &SchemaInfo,
    ) -> Result<Self, XMLGeneratorError> {
        let mut data_type = Self::new(generator);

        data_type.name = complex.name.clone().unwrap_or("".to_string());

        if complex.mixed.is_some() {
            return unimplemented("Mixed types");
        }

        if complex.abstract_ {
            return unimplemented("Abstract types");
        }

        if complex.final_.is_some() {
            return unimplemented("Final types");
        }

        if complex.block.is_some() {
            return unimplemented("Block types");
        }

        let default_attributes_apply = complex.default_attributes_apply;
        if !default_attributes_apply {
            return unimplemented("Non-default attributes");
        }

        for content in &complex.content {
            match content {
                ComplexBaseTypeContent::Group(group_type) => {
                    let group = Group::new(generator, group_type, schema_info, namespaces)?;
                    data_type.groups.push(group);
                }
                ComplexBaseTypeContent::All(group_type) => {
                    let group = Group::new(generator, group_type, schema_info, namespaces)?;
                    data_type.groups.push(group);
                }
                ComplexBaseTypeContent::Choice(group_type) => {
                    let group = Group::new(generator, group_type, schema_info, namespaces)?;
                    data_type.groups.push(group);
                }
                ComplexBaseTypeContent::Sequence(group_type) => {
                    let group = Group::new(generator, group_type, schema_info, namespaces)?;
                    data_type.groups.push(group);
                }
                ComplexBaseTypeContent::Attribute(attribute_type) => {
                    let attribute = Attribute::new(attribute_type, schema_info, namespaces)?;
                    data_type.attributes.push(attribute);
                }
                _ => return unimplemented("Complex base type"),
            }
        }

        Ok(data_type)
    }

    pub(crate) fn get_content(&self, content: &mut Vec<String>) -> Result<(), XMLGeneratorError> {
        for element in self.elements.iter() {
            let name = element.get_name()?;
            content.push(name);
        }

        for group in self.groups.iter() {
            group.get_content(content)?;
        }

        Ok(())
    }

    pub(crate) fn name_equals(&self, name: &String) -> bool {
        self.name.eq(name)
    }

    pub(crate) fn generate_attribute(
        &self,
        xml_element: &mut XMLElement,
        name: &String,
    ) -> Result<(), XMLGeneratorError> {
        if !self.elements.is_empty() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Attributes can contain complex elements".to_string(),
            ));
        }

        if !self.groups.is_empty() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Attributes cannot include groups".to_string(),
            ));
        }

        if !self.attributes.is_empty() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Attributes cannot have their own attributes".to_string(),
            ));
        }

        if let Some(type_info) = &self.type_info {
            return match type_info.generate() {
                Some(value) => {
                    xml_element.add_attribute(name.as_str(), value.as_str());
                    Ok(())
                }
                None => Err(XMLGeneratorError::DataTypeNotFoundError(
                    type_info.get_name(),
                )),
            };
        }

        Err(XMLGeneratorError::DataTypeInformationError(format!(
            "No type information found for type: {}",
            name
        )))
    }

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
            let children = element.generate(data_tracker, xsd)?;
            for child in children {
                xml_element.add_child(child)?;
            }
        }

        for group in self.groups.iter() {
            group.generate(xml_element, data_tracker, xsd)?;
        }

        for attribute in self.attributes.iter() {
            attribute.generate(xml_element, xsd)?;
        }

        Ok(())
    }
}

impl PartialEq for DataType<'_> {
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

        true
    }
}
