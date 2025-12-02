use crate::element_generator::ElementGenerator;
use crate::error::XMLGeneratorError;
use crate::recursion_tracker::RecursionTracker;
use crate::type_generator::TypeGenerator;
use std::string::String;
use xml_builder::{XMLBuilder, XMLVersion};

pub(crate) fn generate_output(
    generator: &ElementGenerator,
    data_types: &Vec<TypeGenerator>,
    elements: &Vec<ElementGenerator>,
    version: XMLVersion,
) -> Result<String, XMLGeneratorError> {
    let mut xml = XMLBuilder::new()
        .version(version)
        .encoding("UTF-8".into())
        .build();

    let mut tracker = RecursionTracker::new();

    let root_element = generator.generate(&mut tracker, data_types, elements)?;

    let mut writer: Vec<u8> = Vec::new();
    xml.set_root_element(root_element);
    let result = xml.generate(&mut writer);
    if let Err(err) = result {
        return Err(XMLGeneratorError::XMLBuilderError(err.to_string()));
    }

    let output = String::from_utf8(writer).expect("Unable to convert XML output to string");

    Ok(output)
}
