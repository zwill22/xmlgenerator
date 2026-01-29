use crate::error::XMLGeneratorError;
use crate::recursion_tracker::RecursionTracker;
use crate::type_info::generate_type;
use crate::xsd::XSD;
use xml_builder::XMLElement;

pub(crate) fn generate_reference(
    tracker: &mut RecursionTracker,
    xsd: &XSD,
    reference: &str,
) -> Result<XMLElement, XMLGeneratorError> {
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

pub fn generate_type_output(
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
