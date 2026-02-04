use crate::error::{XMLGeneratorError, unimplemented};
use crate::name::Name;
use crate::namespaces::Namespaces;
use crate::recursion_tracker::RecursionTracker;
use crate::type_generator::TypeGenerator;
use crate::type_info::{generate_type, get_qname};
use crate::xsd::XSD;
use rand::Rng;
use regextranslator::RegexTranslator;
use std::cmp::{max, min};
use uuid::Uuid;
use xml_builder::XMLElement;
use xsd_parser::models::schema::xs::{ElementType, ElementTypeContent};
use xsd_parser::models::schema::{MaxOccurs, SchemaInfo};

fn generate_type_output(
    xml_element: &mut XMLElement,
    tracker: &mut RecursionTracker,
    xsd: &XSD,
    type_name: &String,
) -> Result<(), XMLGeneratorError> {
    if let Some(output) = generate_type(type_name) {
        return match xml_element.add_text(output) {
            Ok(_) => Ok(()),
            Err(err) => Err(XMLGeneratorError::XMLBuilderError(err.to_string())),
        };
    }

    for data_type in xsd.types() {
        if data_type.name_equals(type_name) {
            return data_type.generate(xml_element, tracker, xsd);
        }
    }

    Err(XMLGeneratorError::DataTypeNotFoundError(type_name.clone()))
}

pub(crate) trait Occurrence {
    fn get_min(&self) -> usize;
    fn get_max(&self) -> Option<usize>;

    fn get_occurrences(&self) -> usize {
        let min_occ = self.get_min();
        let max_occ = self.get_max();

        let mut rng = rand::rng();

        let max_range = 10;
        let max_val = match max_occ {
            None => min_occ + max_range,
            Some(m) => min(m, min_occ + max_range),
        };

        if min_occ == max_val {
            return max_val;
        }

        let min_val = max(1, min_occ);

        rng.random_range(min_val..=max_val)
    }
}

#[derive(Default)]
pub(crate) struct ElementGenerator {
    name: Option<Name>,
    types: Vec<TypeGenerator>,
    type_info: Option<String>,
    reference: Option<Name>,
    min: usize,
    max: Option<usize>,
    id: Uuid,
}

impl ElementGenerator {
    pub(crate) fn new(
        element: &ElementType,
        translator: &RegexTranslator,
        schema: &SchemaInfo,
        namespaces: &Namespaces,
    ) -> Result<Self, XMLGeneratorError> {
        let mut generator = ElementGenerator::default();

        if let Some(element_ref) = &element.ref_ {
            let reference = Name::from_qname(element_ref, namespaces);
            generator.reference = Some(reference);
        }

        if let Some(element_type) = &element.type_ {
            let type_info = get_qname(element_type);
            generator.type_info = Some(type_info);
        }

        if element.substitution_group.is_some() {
            return unimplemented("Element Substitution Groups");
        }

        generator.min = element.min_occurs;

        generator.max = match element.max_occurs {
            MaxOccurs::Unbounded => None,
            MaxOccurs::Bounded(x) => Some(x),
        };

        if element.default.is_some() {
            return unimplemented("Default Element");
        }

        if element.fixed.is_some() {
            return unimplemented("Fixed elements");
        }

        if element.nillable.is_some() {
            return unimplemented("Nillable elements");
        }

        if element.abstract_ {
            return unimplemented("Abstract elements");
        }

        if element.final_.is_some() {
            return unimplemented("Final elements");
        }

        if element.block.is_some() {
            return unimplemented("Block elements");
        }

        if element.form.is_some() {
            return unimplemented("Form elements");
        }

        if element.target_namespace.is_some() {
            return unimplemented("Embedded target namespace");
        }

        generator.name = Name::from_name(&element.name, schema, namespaces);

        for content in &element.content {
            match content {
                ElementTypeContent::SimpleType(simple_type) => {
                    let simple = TypeGenerator::simple_type(simple_type, translator)?;
                    generator.types.push(simple);
                }
                ElementTypeContent::ComplexType(complex_type) => {
                    let complex =
                        TypeGenerator::complex_type(complex_type, translator, schema, namespaces)?;
                    generator.types.push(complex);
                }
                _ => return unimplemented("Element content type"),
            }
        }

        generator.id = Uuid::new_v4();

        Ok(generator)
    }

    pub(crate) fn get_content(
        &self,
        fields: &mut Vec<String>,
        types: &mut Vec<String>,
    ) -> Result<(), XMLGeneratorError> {
        if let Some(reference) = &self.reference {
            fields.push(reference.get_name()?);
        }
        if let Some(type_info) = &self.type_info
            && !type_info.is_empty()
        {
            types.push(type_info.to_string());
        }

        for type_generator in self.types.iter() {
            type_generator.get_content(fields)?;
        }

        Ok(())
    }

    pub(crate) fn get_name(&self) -> Result<String, XMLGeneratorError> {
        match &self.name {
            None => match &self.reference {
                None => Err(XMLGeneratorError::DataTypesFormatError(
                    "Element does not have a name or reference".to_string(),
                )),
                Some(reference) => reference.get_name(),
            },
            Some(name) => name.get_name(),
        }
    }

    fn generate_type_from_name(
        &self,
        tracker: &mut RecursionTracker,
        xsd: &XSD,
    ) -> Result<XMLElement, XMLGeneratorError> {
        let name = self.get_name()?;
        tracker.add(self)?;
        let mut root_element = XMLElement::new(&name);

        match &self.type_info {
            Some(type_info) => {
                if !self.types.is_empty() {
                    return Err(XMLGeneratorError::DataTypesFormatError(
                        "Data has a type and contains type elements".to_string(),
                    ));
                }

                generate_type_output(&mut root_element, tracker, xsd, type_info)?;
            }
            None => {
                for content in self.types.iter() {
                    content.generate(&mut root_element, tracker, xsd)?;
                }
            }
        }

        tracker.remove(self);

        Ok(root_element)
    }

    fn generate_element(
        &self,
        tracker: &mut RecursionTracker,
        xsd: &XSD,
    ) -> Result<Vec<XMLElement>, XMLGeneratorError> {
        let n = self.get_occurrences();

        let mut elements = vec![];
        for _ in 0..n {
            let element = self.generate_type_from_name(tracker, xsd)?;
            elements.push(element);
        }

        Ok(elements)
    }

    fn generate_reference(
        &self,
        tracker: &mut RecursionTracker,
        xsd: &XSD,
        reference: &Name,
    ) -> Result<Vec<XMLElement>, XMLGeneratorError> {
        if self.type_info.is_some() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Element is a reference and a type".to_string(),
            ));
        }
        if !self.types.is_empty() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Element references another element an contains content".to_string(),
            ));
        }

        for element in xsd.elements() {
            if let Some(name) = &element.name {
                if name.eq(reference) {
                    return element.generate(tracker, xsd);
                }
            }
        }

        Err(XMLGeneratorError::XMLBuilderError(
            "Reference not found".to_string(),
        ))
    }

    pub(crate) fn generate(
        &self,
        tracker: &mut RecursionTracker,
        xsd: &XSD,
    ) -> Result<Vec<XMLElement>, XMLGeneratorError> {
        match &self.reference {
            None => self.generate_element(tracker, xsd),
            Some(reference) => self.generate_reference(tracker, xsd, reference),
        }
    }

    pub(crate) fn get_id(&self) -> String {
        self.id.to_string()
    }
}

impl Occurrence for ElementGenerator {
    fn get_min(&self) -> usize {
        self.min
    }

    fn get_max(&self) -> Option<usize> {
        self.max
    }
}

impl PartialEq for ElementGenerator {
    fn eq(&self, other: &Self) -> bool {
        if !self.name.eq(&other.name) {
            return false;
        }
        if !self.type_info.eq(&other.type_info) {
            return false;
        }

        if self.types.len() != other.types.len() {
            return false;
        }

        if !self.types.eq(&other.types) {
            return false;
        }

        for i in 0..self.types.len() {
            if !self.types[i].eq(&other.types[i]) {
                return false;
            }
        }

        true
    }
}
