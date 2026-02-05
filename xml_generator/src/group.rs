use crate::XMLGeneratorError;
use crate::element::{Element, Occurrence};
use crate::error::unimplemented;
use crate::generator::Generator;
use crate::namespaces::Namespaces;
use crate::tracker::Tracker;
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
    choose: bool,
}

impl Group {
    pub(crate) fn new(
        translator: &RegexTranslator,
        group_type: &GroupType,
        schema_info: &SchemaInfo,
        namespaces: &Namespaces,
        choose: bool,
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

        group.choose = choose;

        for content in &group_type.content {
            match content {
                GroupTypeContent::Element(element_type) => {
                    let element = Element::new(translator, element_type, namespaces, schema_info)?;
                    group.elements.push(element)
                }
                GroupTypeContent::Annotation(_) => unimplemented("GroupTypeContent::Annotation")?,
                GroupTypeContent::Group(_) => unimplemented("GroupTypeContent::Group")?,
                GroupTypeContent::All(_) => unimplemented("GroupTypeContent::All")?,
                GroupTypeContent::Choice(_) => unimplemented("GroupTypeContent::Choice")?,
                GroupTypeContent::Sequence(_) => unimplemented("GroupTypeContent::Sequence")?,
                GroupTypeContent::Any(_) => unimplemented("GroupTypeContent::Any")?,
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

    fn generate_element(
        generator: &mut Generator,
        xml_element: &mut XMLElement,
        tracker: &mut Tracker,
        xsd: &Xsd,
        element: &Element,
    ) -> Result<(), XMLGeneratorError> {
        let children = element.generate(generator, tracker, xsd)?;

        for child in children {
            xml_element.add_child(child)?;
        }

        Ok(())
    }

    fn generate_group(
        &self,
        generator: &mut Generator,
        xml_element: &mut XMLElement,
        tracker: &mut Tracker,
        xsd: &Xsd,
    ) -> Result<(), XMLGeneratorError> {
        if self.choose {
            match generator.choose(&self.elements) {
                Some(element) => {
                    Self::generate_element(generator, xml_element, tracker, xsd, element)?
                }
                None => {}
            }
        } else {
            for element in self.elements() {
                Self::generate_element(generator, xml_element, tracker, xsd, element)?;
            }
        }

        Ok(())
    }

    pub(crate) fn generate(
        &self,
        generator: &mut Generator,
        xml_element: &mut XMLElement,
        tracker: &mut Tracker,
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
