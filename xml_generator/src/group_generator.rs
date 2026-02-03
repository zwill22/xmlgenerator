use crate::XMLGeneratorError;
use crate::element_generator::ElementGenerator;
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
    pub(crate) elements: Vec<ElementGenerator>,
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
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

    pub(crate) fn generate(
        &self,
        xml_element: &mut XMLElement,
        data_tracker: &mut RecursionTracker,
        xsd: &XSD,
    ) -> Result<(), XMLGeneratorError> {
        for element in self.elements.iter() {
            let children = element.generate(data_tracker, xsd)?;
            
            for child in children {
                xml_element.add_child(child)?;
            }
        }
    
        Ok(())
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
