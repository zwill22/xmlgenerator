use crate::attribute::Attribute;
use crate::error::{XMLGeneratorError, unimplemented};
use crate::generator::Generator;
use crate::group::Group;
use crate::namespaces::Namespaces;
use crate::special::replace_specials;
use crate::type_info::TypeInfo;
use crate::whitespace::WhiteSpace;
use crate::xsd::Xsd;
use regextranslator::RegexTranslator;
use std::ops::Deref;
use xml_builder::XMLElement;
use xsd_parser::models::schema::SchemaInfo;
use xsd_parser::models::schema::xs::{ComplexBaseType, ComplexBaseTypeContent, SimpleBaseType};

#[derive(Default)]
pub(crate) struct DataType {
    name: String,
    type_info: Option<TypeInfo>,
    groups: Vec<Group>,
    attributes: Vec<Attribute>,
}

impl DataType {
    pub(crate) fn simple_type(
        translator: &RegexTranslator,
        simple: &SimpleBaseType,
    ) -> Result<Self, XMLGeneratorError> {
        let name = simple.name.clone().unwrap_or("".to_string());
        if name.is_empty() {
            return unimplemented("Empty type");
        }

        if simple.final_.is_some() {
            return unimplemented("Final");
        }

        let type_info = TypeInfo::new(translator, &simple.content)?;

        let data_type = DataType {
            name,
            type_info: Some(type_info),
            groups: vec![],
            attributes: vec![],
        };

        Ok(data_type)
    }

    pub(crate) fn complex_type(
        translator: &RegexTranslator,
        complex: &ComplexBaseType,
        namespaces: &Namespaces,
        schema_info: &SchemaInfo,
    ) -> Result<Self, XMLGeneratorError> {
        let name = complex.name.clone().unwrap_or("".to_string());

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

        let mut groups = vec![];
        let mut attributes = vec![];

        for content in &complex.content {
            match content {
                ComplexBaseTypeContent::Group(group_type) => {
                    let group = Group::new(translator, group_type, schema_info, namespaces, false)?;
                    groups.push(group);
                }
                ComplexBaseTypeContent::All(all) => {
                    let group = Group::new(translator, all, schema_info, namespaces, false)?;
                    groups.push(group);
                }
                ComplexBaseTypeContent::Choice(choice) => {
                    let group = Group::new(translator, choice, schema_info, namespaces, true)?;
                    groups.push(group);
                }
                ComplexBaseTypeContent::Sequence(sequence) => {
                    let group = Group::new(translator, sequence, schema_info, namespaces, false)?;
                    groups.push(group);
                }
                ComplexBaseTypeContent::Attribute(attribute_type) => {
                    let attribute = Attribute::new(attribute_type, schema_info, namespaces)?;
                    attributes.push(attribute);
                }
                ComplexBaseTypeContent::Annotation(_) => {
                    unimplemented("ComplexBaseTypeContent::Annotation")?
                }
                ComplexBaseTypeContent::SimpleContent(_) => {
                    unimplemented("ComplexBaseTypeContent::SimpleContent")?
                }
                ComplexBaseTypeContent::ComplexContent(_) => {
                    unimplemented("ComplexBaseTypeContent::ComplexContent")?
                }
                ComplexBaseTypeContent::OpenContent(_) => {
                    unimplemented("ComplexBaseTypeContent::OpenContent")?
                }
                ComplexBaseTypeContent::AttributeGroup(_) => {
                    unimplemented("ComplexBaseTypeContent::AttributeGroup")?
                }
                ComplexBaseTypeContent::AnyAttribute(_) => {
                    unimplemented("ComplexBaseTypeContent::AnyAttribute")?
                }
                ComplexBaseTypeContent::Assert(_) => {
                    unimplemented("ComplexBaseTypeContent::Assert")?
                }
            }
        }

        let data_type = DataType {
            name,
            type_info: None,
            groups,
            attributes,
        };

        Ok(data_type)
    }

    pub(crate) fn get_content(&self, content: &mut Vec<String>) -> Result<(), XMLGeneratorError> {
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
        generator: &mut Generator,
        xml_element: &mut XMLElement,
        xsd: &Xsd,
        name: &String,
    ) -> Result<(), XMLGeneratorError> {
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
            return match type_info.generate(generator, xsd)? {
                Some(value) => {
                    let attribute = replace_specials(&value, &WhiteSpace::Collapse);
                    xml_element.add_attribute(&name, &attribute);
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
        generator: &mut Generator,
        xml_element: &mut XMLElement,
        xsd: &Xsd,
    ) -> Result<(), XMLGeneratorError> {
        if let Some(type_info) = &self.type_info {
            if !self.groups.is_empty() {
                return Err(XMLGeneratorError::DataTypesFormatError(
                    "Type includes type information and groups".to_string(),
                ));
            }

            match type_info.generate(generator, xsd)? {
                None => {
                    return Err(XMLGeneratorError::TypeGenerationError(
                        "No output generated".to_string(),
                    ));
                }
                Some(value) => {
                    let result = replace_specials(&value, &WhiteSpace::Preserve);
                    if let Err(error) = xml_element.add_text(result) {
                        return Err(XMLGeneratorError::XMLBuilderError(error.to_string()));
                    }
                }
            }
        }

        for group in self.groups.iter() {
            group.generate(generator, xml_element, xsd)?;
        }

        for attribute in self.attributes.iter() {
            attribute.generate(generator, xml_element, xsd)?;
        }

        Ok(())
    }
}

impl PartialEq for DataType {
    fn eq(&self, other: &Self) -> bool {
        if !self.name.eq(&other.name) {
            return false;
        }

        if !self.type_info.eq(&other.type_info) {
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
