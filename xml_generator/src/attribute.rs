use crate::error::{XMLGeneratorError, unimplemented};
use crate::name::Name;
use crate::namespaces::Namespaces;
use crate::generator::Generator;
use crate::type_info::TypeInfo;
use crate::xsd::XSD;
use xml_builder::XMLElement;
use xsd_parser::models::schema::SchemaInfo;
use xsd_parser::models::schema::xs::{AttributeType, AttributeUseType};

pub(crate) struct Attribute {
    name: Option<Name>,
    attribute_type: AttributeUseType,
    type_name: String,
    type_info: Option<TypeInfo>,
}

impl Attribute {
    pub(crate) fn new(
        attribute_type: &AttributeType,
        schema_info: &SchemaInfo,
        namespaces: &Namespaces,
    ) -> Result<Attribute, XMLGeneratorError> {
        let mut attribute = Attribute {
            name: None,
            attribute_type: AttributeUseType::Required,
            type_name: String::new(),
            type_info: None,
        };

        attribute.name = Name::from_name(&attribute_type.name, schema_info, namespaces);

        if let Some(attribute_type) = &attribute_type.type_ {
            attribute.type_name = String::from_utf8(attribute_type.local_name().to_vec()).unwrap()
        }

        attribute.attribute_type = attribute_type.use_;

        if attribute_type.ref_.is_some() {
            return unimplemented("Attribute references");
        }

        if attribute_type.default.is_some() {
            return unimplemented("Default attribute");
        }

        if attribute_type.fixed.is_some() {
            return unimplemented("Fixed attribute");
        }

        if attribute_type.form.is_some() {
            return unimplemented("Form attribute");
        }

        if attribute_type.target_namespace.is_some() {
            return unimplemented("Target namespace attribute");
        }

        if attribute_type.inheritable.is_some() {
            return unimplemented("Inheritable attribute");
        }

        if attribute_type.annotation.is_some() {
            return unimplemented("Annotation");
        }

        if attribute_type.simple_type.is_some() {
            return unimplemented("Simple type attribute");
        }

        Ok(attribute)
    }

    fn get_attribute(&self, generator: &mut Generator) -> Option<String> {
        if let Some(type_info) = &self.type_info {
            if let Some(value) = type_info.generate(generator) {
                return Some(value);
            }
        } else if let Some(value) = generator.generate_type(&self.type_name) {
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
        generator: &mut Generator,
        xml_element: &mut XMLElement,
        xsd: &XSD,
    ) -> Result<(), XMLGeneratorError> {
        let mut generated = false;

        if self.attribute_type == AttributeUseType::Prohibited {
            return Ok(());
        }

        let name = self.get_name()?;
        if let Some(attribute) = self.get_attribute(generator) {
            xml_element.add_attribute(name.as_str(), attribute.as_str());
            generated = true;
        }

        for type_generator in xsd.types() {
            if type_generator.name_equals(&self.type_name) {
                type_generator.generate_attribute(generator, xml_element, &name)?;
                generated = true;
            }
        }

        if self.attribute_type == AttributeUseType::Required && !generated {
            if self.type_name.is_empty() && self.type_info.is_none() {
                let value = generator.generate_type("string").unwrap();
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

impl PartialEq for Attribute {
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
