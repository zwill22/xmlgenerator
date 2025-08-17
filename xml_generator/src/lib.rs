use crate::error::XMLGeneratorError;
use crate::fetch_elements::fetch_elements;
use crate::fetch_types::fetch_types;
use crate::find_root::find_root_element;
use crate::generate_output::generate_output;
use crate::generate_schema::generate_schema;

mod attribute;
mod element_generator;
pub mod error;
mod fetch_elements;
mod fetch_types;
mod find_root;
mod generate;
mod generate_output;
mod generate_schema;
mod group;
mod restriction;
mod type_generator;

/// Generate an XML string containing fake data
///
/// Using an XSD file contents as a string, generate an XML file string of the
/// same format with fake data.
///
/// The function uses the `xsd_parser` crate to parse the input. If this library returns
/// en error, then the function returns an `XMLGeneratorError::XMLParserError`.
/// This crate generates a `data_types` object which the XMLGenerator uses th
/// generate the output xml.
///
/// If the `data_types` contains data which is not in the required format, then an
/// `XMLGeneratorError::DataTypeFormatError` is returned. This includes cases
/// such as multiple root nodes or circular dependencies.
///
/// The function sorts the data into a dependency tree and uses this to generate an
/// `XMLBuilder` object using the `xml_builder` crate. If the `XMLBuilder` returns
/// an error when generating the output xml, then an `XMLGeneratorError::XMLBuilderError`
/// is returned.
pub fn generate_xml(xsd_string: &String) -> Result<String, XMLGeneratorError> {
    let schemas = generate_schema(xsd_string)?;
    let data_types = fetch_types(&schemas);
    let elements = fetch_elements(&schemas);
    let root_element = find_root_element(&elements)?;

    generate_output(root_element, &data_types, &elements)
}
