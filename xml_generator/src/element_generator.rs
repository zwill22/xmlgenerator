use crate::error::XMLGeneratorError;
use crate::generate;
use crate::tracker::Tracker;
use crate::type_generator::TypeGenerator;
use uuid::Uuid;
use xml_builder::XMLElement;

pub(crate) struct ElementGenerator {
    pub(crate) name: Option<String>,
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
            contents: vec![],
            type_info: None,
            reference: None,
            min: 1,
            max: None,
            id: Uuid::new_v4(),
        }
    }

    pub(crate) fn get_name(&self) -> Result<&String, XMLGeneratorError> {
        if let Some(name) = &self.name {
            return Ok(name);
        }

        if let Some(reference) = &self.reference {
            return Ok(reference);
        }

        Err(XMLGeneratorError::DataTypesFormatError(
            "Element does not have a name or a reference".to_string(),
        ))
    }

    pub(crate) fn generate(
        &self,
        data_tracker: &mut Tracker,
        data_types: &Vec<TypeGenerator>,
        elements: &Vec<ElementGenerator>,
    ) -> Result<XMLElement, XMLGeneratorError> {
        if let Some(reference) = &self.reference {
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

            return generate::generate_reference(data_tracker, reference, data_types, elements);
        }

        let name = self.get_name()?;
        data_tracker.add(self)?;
        let mut root_element = XMLElement::new(name);

        if self.type_info.is_some() {
            if !self.contents.is_empty() {
                return Err(XMLGeneratorError::DataTypesFormatError(
                    "Data has a type and contains type elements".to_string(),
                ));
            }

            let type_info = self.type_info.as_ref().unwrap();

            generate::generate_type_output(
                &mut root_element,
                data_tracker,
                type_info,
                data_types,
                elements,
            )?;
        } else {
            for content in self.contents.iter() {
                content.generate(&mut root_element, data_tracker, data_types, elements)?;
            }
        }

        data_tracker.remove(self);

        Ok(root_element)
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
