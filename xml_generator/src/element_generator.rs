use crate::error::XMLGeneratorError;
use crate::recursion_tracker::RecursionTracker;
use crate::type_generator::TypeGenerator;
use crate::type_info::generate_type;
use crate::xsd::XSD;
use uuid::Uuid;
use xml_builder::XMLElement;

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
        if data_type.name.eq(type_name) {
            return data_type.generate(xml_element, tracker, xsd);
        }
    }

    Err(XMLGeneratorError::DataTypeNotFoundError(type_name.clone()))
}

pub(crate) struct ElementGenerator {
    pub(crate) name: Option<String>,
    pub(crate) namespace: Option<String>,
    pub(crate) contents: Vec<TypeGenerator>,
    pub(crate) type_info: Option<String>,
    pub(crate) reference: Option<String>,
    pub(crate) min: usize,
    pub(crate) max: Option<usize>,
    id: Uuid,
}

impl ElementGenerator {
    pub(crate) fn new() -> Self {
        ElementGenerator {
            name: None,
            namespace: None,
            contents: vec![],
            type_info: None,
            reference: None,
            min: 1,
            max: None,
            id: Uuid::new_v4(),
        }
    }

    fn get_suffix(&self) -> Result<String, XMLGeneratorError> {
        if let Some(name) = &self.name {
            return Ok(name.clone());
        }

        if let Some(reference) = &self.reference {
            return Ok(reference.clone());
        }

        Err(XMLGeneratorError::DataTypesFormatError(
            "Element does not have a name or a reference".to_string(),
        ))
    }

    pub(crate) fn get_name(&self) -> Result<String, XMLGeneratorError> {
        let suffix = self.get_suffix()?;

        match &self.namespace {
            None => Ok(suffix),
            Some(ns) => {
                let output = ns.clone() + ":" + &suffix;
                Ok(output)
            }
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

        match self.type_info {
            Some(ref type_info) => {
                if !self.contents.is_empty() {
                    return Err(XMLGeneratorError::DataTypesFormatError(
                        "Data has a type and contains type elements".to_string(),
                    ));
                }

                generate_type_output(&mut root_element, tracker, xsd, type_info)?;
            }
            None => {
                for content in self.contents.iter() {
                    content.generate(&mut root_element, tracker, xsd)?;
                }
            }
        }

        tracker.remove(self);

        Ok(root_element)
    }

    fn generate_reference(
        &self,
        tracker: &mut RecursionTracker,
        xsd: &XSD,
        reference: &String,
    ) -> Result<XMLElement, XMLGeneratorError> {
        if self.type_info.is_some() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Element is a reference and a type".to_string(),
            ));
        }
        if !self.contents.is_empty() {
            return Err(XMLGeneratorError::DataTypesFormatError(
                "Element references another element an contains content".to_string(),
            ));
        }

        for element in xsd.elements() {
            let name = element.get_name()?;
            if name.eq(reference) {
                return element.generate(tracker, xsd);
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
    ) -> Result<XMLElement, XMLGeneratorError> {
        match &self.reference {
            None => self.generate_type_from_name(tracker, xsd),
            Some(reference) => self.generate_reference(tracker, xsd, reference),
        }
    }

    pub(crate) fn get_id(&self) -> String {
        self.id.to_string()
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

        if self.contents.len() != other.contents.len() {
            return false;
        }

        if !self.contents.eq(&other.contents) {
            return false;
        }

        for i in 0..self.contents.len() {
            if !self.contents[i].eq(&other.contents[i]) {
                return false;
            }
        }

        true
    }
}
