use crate::data_type::DataType;
use crate::error::{XMLGeneratorError, unimplemented};
use crate::generator::Generator;
use crate::name::Name;
use crate::namespaces::Namespaces;
use crate::tracker::Tracker;
use crate::xsd::Xsd;
use rand::Rng;
use regextranslator::RegexTranslator;
use std::cmp::{max, min};
use uuid::Uuid;
use xml_builder::XMLElement;
use xsd_parser::models::schema::xs::{ElementType, ElementTypeContent};
use xsd_parser::models::schema::{MaxOccurs, SchemaInfo};

fn generate_type_output(
    generator: &mut Generator,
    xml_element: &mut XMLElement,
    tracker: &mut Tracker,
    xsd: &Xsd,
    type_name: &String,
) -> Result<(), XMLGeneratorError> {
    // TODO TypeGenerator?
    if let Some(output) = generator.generate_type(type_name) {
        return match xml_element.add_text(output) {
            Ok(_) => Ok(()),
            Err(err) => Err(XMLGeneratorError::XMLBuilderError(err.to_string())),
        };
    }

    for data_type in xsd.types() {
        if data_type.name_equals(type_name) {
            return data_type.generate(generator, xml_element, tracker, xsd);
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

pub(crate) struct Element {
    name: Option<Name>,
    data_types: Vec<DataType>,
    type_name: Option<String>,
    reference: Option<Name>,
    min: usize,
    max: Option<usize>,
    id: Uuid,
}

impl Element {
    pub(crate) fn new(
        translator: &RegexTranslator,
        element_type: &ElementType,
        namespaces: &Namespaces,
        schema: &SchemaInfo,
    ) -> Result<Self, XMLGeneratorError> {
        let mut element = Self::default();

        if let Some(element_ref) = &element_type.ref_ {
            let reference = Name::from_qname(element_ref, namespaces);
            element.reference = Some(reference);
        }

        if let Some(element_type) = &element_type.type_ {
            element.type_name = String::from_utf8(element_type.local_name().to_vec()).ok();
        }

        if element_type.substitution_group.is_some() {
            return unimplemented("Element Substitution Groups");
        }

        element.min = element_type.min_occurs;

        element.max = match element_type.max_occurs {
            MaxOccurs::Unbounded => None,
            MaxOccurs::Bounded(x) => Some(x),
        };

        if element_type.default.is_some() {
            return unimplemented("Default Element");
        }

        if element_type.fixed.is_some() {
            return unimplemented("Fixed elements");
        }

        if element_type.nillable.is_some() {
            return unimplemented("Nillable elements");
        }

        if element_type.abstract_ {
            return unimplemented("Abstract elements");
        }

        if element_type.final_.is_some() {
            return unimplemented("Final elements");
        }

        if element_type.block.is_some() {
            return unimplemented("Block elements");
        }

        if element_type.form.is_some() {
            return unimplemented("Form elements");
        }

        if element_type.target_namespace.is_some() {
            return unimplemented("Embedded target namespace");
        }

        element.name = Name::from_name(&element_type.name, schema, namespaces);

        for content in &element_type.content {
            match content {
                ElementTypeContent::SimpleType(simple_type) => {
                    let simple = DataType::simple_type(translator, simple_type)?;
                    element.data_types.push(simple);
                }
                ElementTypeContent::ComplexType(complex_type) => {
                    let complex =
                        DataType::complex_type(translator, complex_type, namespaces, schema)?;
                    element.data_types.push(complex);
                }
                ElementTypeContent::Annotation(_) => {
                    unimplemented("ElementTypeContent::Annotation")?
                }
                ElementTypeContent::Alternative(_) => {
                    unimplemented("ElementTypeContent::Alternative")?
                }
                ElementTypeContent::Unique(_) => unimplemented("ElementTypeContent::Unique")?,
                ElementTypeContent::Key(_) => unimplemented("ElementTypeContent::Key")?,
                ElementTypeContent::Keyref(_) => unimplemented("ElementTypeContent::Keyref")?,
            }
        }

        Ok(element)
    }

    pub(crate) fn get_content(
        &self,
        fields: &mut Vec<String>,
        types: &mut Vec<String>,
    ) -> Result<(), XMLGeneratorError> {
        if let Some(reference) = &self.reference {
            fields.push(reference.get_name()?);
        }
        if let Some(type_info) = &self.type_name
            && !type_info.is_empty()
        {
            types.push(type_info.to_string());
        }

        for type_generator in self.data_types.iter() {
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
        generator: &mut Generator,
        tracker: &mut Tracker,
        xsd: &Xsd,
    ) -> Result<XMLElement, XMLGeneratorError> {
        let name = self.get_name()?;
        tracker.add(self)?;
        let mut root_element = XMLElement::new(&name);

        match &self.type_name {
            Some(type_info) => {
                if !self.data_types.is_empty() {
                    return Err(XMLGeneratorError::DataTypesFormatError(
                        "Data has a type and contains type elements".to_string(),
                    ));
                }

                generate_type_output(generator, &mut root_element, tracker, xsd, type_info)?;
            }
            None => {
                for content in self.data_types.iter() {
                    content.generate(generator, &mut root_element, tracker, xsd)?;
                }
            }
        }

        tracker.remove(self);

        Ok(root_element)
    }

    fn generate_element(
        &self,
        generator: &mut Generator,
        tracker: &mut Tracker,
        xsd: &Xsd,
    ) -> Result<Vec<XMLElement>, XMLGeneratorError> {
        let n = self.get_occurrences();

        let mut elements = vec![];
        for _ in 0..n {
            let element = self.generate_type_from_name(generator, tracker, xsd)?;
            elements.push(element);
        }

        Ok(elements)
    }

    fn generate_reference(
        &self,
        generator: &mut Generator,
        tracker: &mut Tracker,
        xsd: &Xsd,
        reference: &Name,
    ) -> Result<Vec<XMLElement>, XMLGeneratorError> {
        if self.type_name.is_some() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Element is a reference and a type".to_string(),
            ));
        }
        if !self.data_types.is_empty() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Element references another element an contains content".to_string(),
            ));
        }

        for element in xsd.elements() {
            if let Some(name) = &element.name
                && name.eq(reference)
            {
                return element.generate(generator, tracker, xsd);
            }
        }

        Err(XMLGeneratorError::XMLBuilderError(
            "Reference not found".to_string(),
        ))
    }

    pub(crate) fn generate(
        &self,
        generator: &mut Generator,
        tracker: &mut Tracker,
        xsd: &Xsd,
    ) -> Result<Vec<XMLElement>, XMLGeneratorError> {
        match &self.reference {
            None => self.generate_element(generator, tracker, xsd),
            Some(reference) => self.generate_reference(generator, tracker, xsd, reference),
        }
    }

    pub(crate) fn get_id(&self) -> String {
        self.id.to_string()
    }
}

impl Default for Element {
    fn default() -> Self {
        Self {
            name: None,
            data_types: vec![],
            type_name: None,
            reference: None,
            min: 0,
            max: None,
            id: Uuid::new_v4(),
        }
    }
}

impl Occurrence for Element {
    fn get_min(&self) -> usize {
        self.min
    }

    fn get_max(&self) -> Option<usize> {
        self.max
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        if !self.name.eq(&other.name) {
            return false;
        }
        if !self.type_name.eq(&other.type_name) {
            return false;
        }

        if self.data_types.len() != other.data_types.len() {
            return false;
        }

        if !self.data_types.eq(&other.data_types) {
            return false;
        }

        for i in 0..self.data_types.len() {
            if !self.data_types[i].eq(&other.data_types[i]) {
                return false;
            }
        }

        true
    }
}
