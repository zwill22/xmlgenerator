use crate::error::{XMLGeneratorError, unimplemented};
use crate::generator::Generator;
use crate::name::Name;
use crate::namespaces::Namespaces;
use crate::type_info::TypeInfo;
use crate::xsd::Xsd;
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

        attribute.name = Name::from_name(schema_info, namespaces, &attribute_type.name);

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
    
    fn get_full_name(&self, generator: &mut Generator, name: &Name) -> Result<String, XMLGeneratorError> {
        let current_namespace = generator.get_current_namespace();

        match name.get_prefix() {
            None => name.get_name(),
            Some(prefix) => {
                if Some(&prefix) == current_namespace {
                    name.get_suffix()
                } else {
                    generator.add_namespace(prefix);
                    name.get_name()
                }
            }
        }
    }

    fn get_name(&self, generator: &mut Generator) -> Result<String, XMLGeneratorError> {
        match &self.name {
            None => Err(XMLGeneratorError::DataTypesFormatError(
                "Attribute Name is empty".to_string(),
            )),
            Some(name) => self.get_full_name(generator, name),
        }
    }

    pub(crate) fn generate(
        &self,
        generator: &mut Generator,
        xml_element: &mut XMLElement,
        xsd: &Xsd,
    ) -> Result<(), XMLGeneratorError> {
        let mut generated = false;
        let n_namespaces  = generator.n_namespaces();

        if self.attribute_type == AttributeUseType::Prohibited {
            return Ok(());
        }

        let name = self.get_name(generator)?;
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

        generator.update_namespaces(n_namespaces);

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
