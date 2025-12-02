use crate::element_generator::ElementGenerator;
use crate::error::XMLGeneratorError;
use crate::recursion_tracker::RecursionTracker;
use crate::type_generator::TypeGenerator;
use crate::type_info::generate_type;
use xml_builder::XMLElement;

pub(crate) fn generate_reference(
    data_tracker: &mut RecursionTracker,
    reference: &String,
    data_types: &Vec<TypeGenerator>,
    elements: &Vec<ElementGenerator>,
) -> Result<XMLElement, XMLGeneratorError> {
    for element in elements.iter() {
        let name = element.get_name()?;
        if name.eq(reference) {
            return element.generate(data_tracker, data_types, elements);
        }
    }

    Err(XMLGeneratorError::XMLBuilderError(
        "Reference not found".to_string(),
    ))
}

pub fn generate_type_output(
    xml_element: &mut XMLElement,
    data_tracker: &mut RecursionTracker,
    type_name: &String,
    data_types: &Vec<TypeGenerator>,
    elements: &Vec<ElementGenerator>,
) -> Result<(), XMLGeneratorError> {
    if let Some(output) = generate_type(type_name) {
        let result = xml_element.add_text(output);
        return match result {
            Ok(_) => Ok(()),
            Err(err) => Err(XMLGeneratorError::XMLBuilderError(err.to_string())),
        };
    }

    for data_type in data_types {
        if data_type.name.eq(type_name) {
            return data_type.generate(xml_element, data_tracker, data_types, elements);
        }
    }

    Err(XMLGeneratorError::DataTypeNotFoundError(type_name.clone()))
}
