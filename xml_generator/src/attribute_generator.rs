use crate::error::{XMLGeneratorError, unimplemented};
use crate::name::Name;
use crate::namespaces::Namespaces;
use crate::type_generator::TypeGenerator;
use crate::type_info::{TypeInfo, generate_type, get_qname};
use crate::xsd::XSD;
use xml_builder::XMLElement;
use xsd_parser::models::schema::SchemaInfo;
use xsd_parser::models::schema::xs::{AttributeType, AttributeUseType};

fn generate_attribute_from_type(
    xml_element: &mut XMLElement,
    generator: &TypeGenerator,
    name: &String,
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

    if let Some(type_info) = &generator.type_info {
        return match type_info.generate() {
            Some(value) => {
                xml_element.add_attribute(name.as_str(), value.as_str());
                Ok(())
            }
            None => Err(XMLGeneratorError::DataTypeNotFoundError(
                type_info.name.clone(),
            )),
        };
    }

    Err(XMLGeneratorError::DataTypeInformationError(format!(
        "No type information found for type: {}",
        name
    )))
}

pub(crate) struct AttributeGenerator {
    pub(crate) name: Option<Name>,
    pub(crate) attribute_type: AttributeUseType,
    pub(crate) type_name: String,
    pub(crate) type_info: Option<TypeInfo>,
}

impl AttributeGenerator {
    pub(crate) fn new(
        attribute: &AttributeType,
        schema_info: &SchemaInfo,
        namespaces: &Namespaces,
    ) -> Result<Self, XMLGeneratorError> {
        let mut generator = AttributeGenerator {
            name: None,
            attribute_type: AttributeUseType::Required,
            type_name: String::new(),
            type_info: None,
        };

        generator.name = Name::from_name(&attribute.name, schema_info, namespaces);

        if let Some(attribute_type) = &attribute.type_ {
            generator.type_name = get_qname(attribute_type);
        }

        generator.attribute_type = attribute.use_;

        if attribute.ref_.is_some() {
            return unimplemented("Attribute references");
        }

        if attribute.default.is_some() {
            return unimplemented("Default attribute");
        }

        if attribute.fixed.is_some() {
            return unimplemented("Fixed attribute");
        }

        if attribute.form.is_some() {
            return unimplemented("Form attribute");
        }

        if attribute.target_namespace.is_some() {
            return unimplemented("Target namespace attribute");
        }

        if attribute.inheritable.is_some() {
            return unimplemented("Inheritable attribute");
        }

        if attribute.annotation.is_some() {
            return unimplemented("Annotation");
        }

        if attribute.simple_type.is_some() {
            return unimplemented("Simple type attribute");
        }

        Ok(generator)
    }

    fn get_attribute(&self) -> Option<String> {
        if let Some(type_info) = &self.type_info {
            if let Some(value) = type_info.generate() {
                return Some(value);
            }
        } else if let Some(value) = generate_type(&self.type_name) {
            return Some(value);
        }

        None
    }

    fn get_name(&self) -> Result<String, XMLGeneratorError> {
        match &self.name {
            None => Err(XMLGeneratorError::DataTypesFormatError(
                "Attribute Name is empty".to_string(),
            )),
            Some(name) => name.get_name(),
        }
    }

    pub(crate) fn generate(
        &self,
        xml_element: &mut XMLElement,
        xsd: &XSD,
    ) -> Result<(), XMLGeneratorError> {
        let mut generated = false;

        if self.attribute_type == AttributeUseType::Prohibited {
            return Ok(());
        }

        let name = self.get_name()?;
        if let Some(attribute) = self.get_attribute() {
            xml_element.add_attribute(name.as_str(), attribute.as_str());
            generated = true;
        }

        for type_generator in xsd.types() {
            if type_generator.name.eq(&self.type_name) {
                generate_attribute_from_type(xml_element, type_generator, &name)?;
                generated = true;
            }
        }

        if self.attribute_type == AttributeUseType::Required && !generated {
            if self.type_name.is_empty() && self.type_info.is_none() {
                let value = generate_type("string").unwrap();
                xml_element.add_attribute(name.as_str(), value.as_str());
            } else {
                return Err(XMLGeneratorError::TypeGenerationError(
                    "Required attribute not generated".to_string(),
                ));
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
