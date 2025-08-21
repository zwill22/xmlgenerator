use crate::error::XMLGeneratorError;
use crate::generate::generate;
use crate::type_generator::TypeGenerator;
use xml_builder::XMLElement;
use xsd_parser::models::schema::xs::AttributeUseType;

fn generate_attribute_from_type(
    xml_element: &mut XMLElement,
    generator: &TypeGenerator,
    name: &String
) -> Result<(), XMLGeneratorError> {
    if !generator.elements.is_empty() {
        return Err(XMLGeneratorError::DataTypesFormatError(
            "Attributes can contain complex elements".to_string(),
        ));
    }

    if !generator.groups.is_empty() {
        return Err(XMLGeneratorError::DataTypesFormatError(
            "Attributes cannot include groups".to_string(),
        ));
    }

    if !generator.attributes.is_empty() {
        return Err(XMLGeneratorError::DataTypesFormatError(
            "Attributes cannot have their own attributes".to_string(),
        ));
    }

    if generator.type_info.is_empty() {
        return Err(XMLGeneratorError::DataTypeError(
            "No type dat for attribute".to_string(),
        ));
    }

    let output = generate(&generator.type_info);
    match output {
        Some(value) => {
            xml_element.add_attribute(name.as_str(), value.as_str());
            Ok(())
        }
        None => Err(XMLGeneratorError::DataTypeError(
            "Data type not found".to_string(),
        )),
    }
}

pub struct AttributeGenerator {
    pub(crate) name: String,
    pub(crate) attribute_type: AttributeUseType,
    pub(crate) type_name: String,
}

impl AttributeGenerator {
    pub(crate) fn new() -> Self {
        AttributeGenerator {
            name: String::new(),
            attribute_type: AttributeUseType::Required,
            type_name: String::new(),
        }
    }

    pub(crate) fn generate(
        &self,
        xml_element: &mut XMLElement,
        data_types: &Vec<TypeGenerator>,
    ) -> Result<(), XMLGeneratorError> {
        if self.name.is_empty() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Attribute Name is empty".to_string(),
            ));
        }

        if self.type_name.is_empty() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Attribute type name is empty".to_string(),
            ));
        }

        if self.attribute_type == AttributeUseType::Prohibited {
            return Ok(());
        }

        let name = &self.name;
        let type_name = &vec![self.type_name.clone()];
        let value = generate(type_name);
        if let Some(val) = value {
            xml_element.add_attribute(name.as_str(), val.as_str());

            return Ok(());
        }

        for type_generator in data_types {
            if type_generator.name.eq(&self.type_name) {
                generate_attribute_from_type(xml_element, type_generator, name)?;
            }
        }

        Ok(())
    }
}

impl PartialEq for AttributeGenerator {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        if !self.type_name.eq(&other.type_name) {
            return false;
        }

        if self.attribute_type != other.attribute_type {
            return false;
        }

        true
    }
}
