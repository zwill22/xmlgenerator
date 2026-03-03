use crate::error::{XMLGeneratorError, unimplemented};
use crate::generator::Generator;
use crate::name::Name;
use crate::namespaces::Namespaces;
use crate::xsd::Xsd;
use crate::xsd_type::XsdType;
use std::cmp::PartialEq;
use xml_builder::XMLElement;
use xsd_parser::models::schema::SchemaInfo;
use xsd_parser::models::schema::xs::{AttributeType, AttributeUseType};
use crate::special::replace_specials;
use crate::whitespace::WhiteSpace;

pub(crate) struct Attribute {
    name: Option<Name>,
    type_name: Option<String>,
    xsd_type: XsdType,
    use_type: AttributeUseType,
}

impl Attribute {
    pub(crate) fn new(
        attribute_type: &AttributeType,
        schema_info: &SchemaInfo,
        namespaces: &Namespaces,
    ) -> Result<Attribute, XMLGeneratorError> {
        let mut attribute = Attribute {
            name: None,
            type_name: None,
            xsd_type: XsdType::None,
            use_type: AttributeUseType::Required,
        };

        attribute.name = Name::from_name(schema_info, namespaces, &attribute_type.name)?;

        if let Some(attribute_type) = &attribute_type.type_ {
            let type_name = String::from_utf8(attribute_type.local_name().to_vec()).unwrap();
            attribute.xsd_type = XsdType::from_string(&type_name)?;
            if matches!(attribute.xsd_type, XsdType::None) {
                attribute.type_name = Some(type_name);
            }
        }

        attribute.use_type = attribute_type.use_;

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

    fn get_full_name(
        &self,
        generator: &mut Generator,
        name: &Name,
    ) -> Result<String, XMLGeneratorError> {
        let current_namespace = generator.get_current_namespace();

        match name.get_prefix() {
            None => name.get_name(),
            Some(prefix) => {
                if Some(&prefix) == current_namespace && !generator.attributes_qualified() {
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
        let n_namespaces = generator.n_namespaces();

        if self.use_type == AttributeUseType::Prohibited {
            return Ok(());
        }

        let name = self.get_name(generator)?;
        if let Some(attribute) = generator.generate_type(&self.xsd_type) {
            let value = replace_specials(&attribute, &self.xsd_type.whitespace())?;
            xml_element.add_attribute(&name, &value);
            generated = true;
        }

        if let Some(type_name) = &self.type_name {
            for type_generator in xsd.types() {
                if type_generator.name_equals(type_name) {
                    generator.track_ref(type_name.as_str());
                    type_generator.generate_attribute(generator, xml_element, xsd, &name)?;
                    generator.untrack_ref(type_name.as_str())?;
                    generated = true;
                }
            }
        }

        if !generated {
            if self.type_name.is_none() {
                let xsd_type = XsdType::string("", &WhiteSpace::Collapse)?;
                let value = generator.generate_type(&xsd_type).unwrap();
                let attribute = replace_specials(&value, &xsd_type.whitespace())?;
                xml_element.add_attribute(&name, &attribute);
            } else if self.use_type == AttributeUseType::Required {
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

        if self.xsd_type != other.xsd_type {
            return false;
        }

        if self.use_type != other.use_type {
            return false;
        }

        true
    }
}
