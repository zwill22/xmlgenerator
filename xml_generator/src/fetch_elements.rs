use crate::XMLGeneratorError;
use crate::element_generator::ElementGenerator;
use crate::error::unimplemented;
use crate::fetch_types::get_element_type;
use regextranslator::RegexTranslator;
use xsd_parser::Schemas;
use xsd_parser::models::schema::xs::SchemaContent;
use crate::metadata::SchemaMetadata;

fn fetch_element(
    content: &SchemaContent,
    regex_translator: &RegexTranslator,
    metadata: &SchemaMetadata,
) -> Option<Result<ElementGenerator, XMLGeneratorError>> {
    match content {
        SchemaContent::Include(_) => Some(unimplemented("Include")),
        SchemaContent::Import(_) => None,
        SchemaContent::Redefine(_) => Some(unimplemented("Redefine")),
        SchemaContent::Override(_) => Some(unimplemented("Override")),
        SchemaContent::Annotation(_) => None,
        SchemaContent::DefaultOpenContent(_) => Some(unimplemented("DefaultOpenContent")),
        SchemaContent::SimpleType(_) => None,
        SchemaContent::ComplexType(_) => None,
        SchemaContent::Group(_) => Some(unimplemented("Top-level group not supported")),
        SchemaContent::AttributeGroup(_) => Some(unimplemented("AttributeGroup")),
        SchemaContent::Element(x) => Some(get_element_type(x, regex_translator, metadata)),
        SchemaContent::Attribute(_) => Some(unimplemented("Attribute")),
        SchemaContent::Notation(_) => Some(unimplemented("Notation")),
    }
}

pub(crate) fn fetch_elements(
    schemas: &Schemas,
    regex_translator: &RegexTranslator,
    metadata: &SchemaMetadata,
) -> Result<Vec<ElementGenerator>, XMLGeneratorError> {
    let mut elements = vec![];
    for (_schema_id, schema_info) in schemas.schemas() {
        let schema = &schema_info.schema;
        for content in &schema.content {
            let element = fetch_element(content, regex_translator, metadata);
            if let Some(e) = element {
                elements.push(e?);
            }
        }
    }

    Ok(elements)
}
