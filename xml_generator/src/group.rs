use crate::XMLGeneratorError;
use crate::element::{Element, Occurrence};
use crate::error::unimplemented;
use crate::generator::Generator;
use crate::namespaces::Namespaces;
use crate::tracker::RecursionTracker;
use crate::xsd::Xsd;
use regextranslator::RegexTranslator;
use std::slice::Iter;
use xml_builder::XMLElement;
use xsd_parser::models::schema::xs::{GroupType, GroupTypeContent};
use xsd_parser::models::schema::{MaxOccurs, SchemaInfo};

#[derive(Default)]
pub struct Group {
    elements: Vec<Element>,
    min: usize,
    max: Option<usize>,
}

impl Group {
    pub(crate) fn new(
        translator: &RegexTranslator,
        group_type: &GroupType,
        schema_info: &SchemaInfo,
        namespaces: &Namespaces,
    ) -> Result<Group, XMLGeneratorError> {
        let mut group = Group::default();

        if group_type.name.is_some() {
            return unimplemented("Named groups");
        }

        if group_type.ref_.is_some() {
            return unimplemented("Group references");
        }

        group.min = group_type.min_occurs;

        group.max = match group_type.max_occurs {
            MaxOccurs::Unbounded => None,
            MaxOccurs::Bounded(x) => Some(x),
        };

        for content in &group_type.content {
            match content {
                GroupTypeContent::Element(element_type) => {
                    let element = Element::new(translator, element_type, namespaces, schema_info)?;
                    group.elements.push(element)
                }
                _ => return unimplemented("Group type content"),
            }
        }

        Ok(group)
    }

    pub(crate) fn get_content(&self, content: &mut Vec<String>) -> Result<(), XMLGeneratorError> {
        for element in self.elements.iter() {
            let name = element.get_name()?;
            content.push(name);
        }

        Ok(())
    }

    fn elements(&self) -> Iter<'_, Element> {
        self.elements.iter()
    }

    fn generate_group(
        &self,
        generator: &mut Generator,
        xml_element: &mut XMLElement,
        tracker: &mut RecursionTracker,
        xsd: &Xsd,
    ) -> Result<(), XMLGeneratorError> {
        for element in self.elements() {
            let children = element.generate(generator, tracker, xsd)?;

            for child in children {
                xml_element.add_child(child)?;
            }
        }

        Ok(())
    }

    pub(crate) fn generate(
        &self,
        generator: &mut Generator,
        xml_element: &mut XMLElement,
        tracker: &mut RecursionTracker,
        xsd: &Xsd,
    ) -> Result<(), XMLGeneratorError> {
        let n = self.get_occurrences();

        for _ in 0..n {
            self.generate_group(generator, xml_element, tracker, xsd)?;
        }

        Ok(())
    }
}

impl Occurrence for Group {
    fn get_min(&self) -> usize {
        self.min
    }

    fn get_max(&self) -> Option<usize> {
        self.max
    }
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        if !self.elements.eq(&other.elements) {
            return false;
        }

        if self.min != other.min {
            return false;
        }

        if !self.max.eq(&other.max) {
            return false;
        }

        true
    }
}
