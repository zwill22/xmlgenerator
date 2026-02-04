use crate::XMLGeneratorError;
use crate::element_generator::{ElementGenerator, Occurrence};
use crate::error::unimplemented;
use crate::namespaces::Namespaces;
use crate::recursion_tracker::RecursionTracker;
use crate::xsd::XSD;
use regextranslator::RegexTranslator;
use xml_builder::XMLElement;
use xsd_parser::models::schema::xs::{GroupType, GroupTypeContent};
use xsd_parser::models::schema::{MaxOccurs, SchemaInfo};

#[derive(Default)]
pub struct GroupGenerator {
    elements: Vec<ElementGenerator>,
    min: usize,
    max: Option<usize>,
}

impl GroupGenerator {
    pub(crate) fn new(
        group: &GroupType,
        translator: &RegexTranslator,
        schema_info: &SchemaInfo,
        namespaces: &Namespaces,
    ) -> Result<GroupGenerator, XMLGeneratorError> {
        let mut generator = GroupGenerator::default();

        if group.name.is_some() {
            return unimplemented("Named groups");
        }

        if group.ref_.is_some() {
            return unimplemented("Group references");
        }

        generator.min = group.min_occurs;

        generator.max = match group.max_occurs {
            MaxOccurs::Unbounded => None,
            MaxOccurs::Bounded(x) => Some(x),
        };

        for content in &group.content {
            match content {
                GroupTypeContent::Element(element_type) => {
                    let element =
                        ElementGenerator::new(element_type, translator, schema_info, namespaces)?;
                    generator.elements.push(element)
                }
                _ => return unimplemented("Group type content"),
            }
        }

        Ok(generator)
    }

    pub(crate) fn get_content(&self, content: &mut Vec<String>) -> Result<(), XMLGeneratorError> {
        for element in self.elements.iter() {
            let name = element.get_name()?;
            content.push(name);
        }

        Ok(())
    }

    fn generate_group(
        &self,
        xml_element: &mut XMLElement,
        tracker: &mut RecursionTracker,
        xsd: &XSD,
    ) -> Result<(), XMLGeneratorError> {
        for element in self.elements.iter() {
            let children = element.generate(tracker, xsd)?;

            for child in children {
                xml_element.add_child(child)?;
            }
        }

        Ok(())
    }

    pub(crate) fn generate(
        &self,
        xml_element: &mut XMLElement,
        tracker: &mut RecursionTracker,
        xsd: &XSD,
    ) -> Result<(), XMLGeneratorError> {
        let n = self.get_occurrences();

        for _ in 0..n {
            self.generate_group(xml_element, tracker, xsd)?;
        }
        
        Ok(())
    }
}

impl Occurrence for GroupGenerator {
    fn get_min(&self) -> usize {
        self.min
    }
    
    fn get_max(&self) -> Option<usize> {
        self.max
    }
}

impl PartialEq for GroupGenerator {
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
