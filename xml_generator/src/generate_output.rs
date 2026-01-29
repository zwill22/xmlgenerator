use crate::error::XMLGeneratorError;
use crate::xsd::XSD;
use std::string::String;
use xml_builder::{XMLBuilder, XMLElement};
use crate::recursion_tracker::RecursionTracker;

fn build_xml(xsd: &XSD) -> Result<XMLElement, XMLGeneratorError> {
    let root = xsd.find_root()?;

    let mut tracker = RecursionTracker::new();

    let mut root_element = root.generate(&mut tracker, xsd)?;
    xsd.apply_metadata_to(&mut root_element);

    Ok(root_element)
}

pub(crate) fn generate_output(xsd: &XSD) -> Result<String, XMLGeneratorError> {
    let schema_version = xsd.get_version()?;

    let mut xml = XMLBuilder::new()
        .expand_empty_tags(true)
        .version(schema_version)
        .encoding("UTF-8".into())
        .build();


    let root_element = build_xml(xsd)?;
    xml.set_root_element(root_element);

    let mut writer: Vec<u8> = Vec::new();

    let output = match xml.generate(&mut writer) {
        Ok(_) => String::from_utf8(writer).expect("Invalid UTF-8 sequence"),
        Err(e) => {
            return Err(XMLGeneratorError::XMLBuilderError(e.to_string()));
        }
    };

    Ok(output)
}
