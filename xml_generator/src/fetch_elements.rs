use crate::XMLGeneratorError;
use crate::element_generator::ElementGenerator;
use crate::fetch_types::get_element_type;
use regextranslator::RegexTranslator;
use xsd_parser::Schemas;
use xsd_parser::models::schema::xs::SchemaContent;

fn fetch_element(
    content: &SchemaContent,
    regex_translator: &RegexTranslator,
) -> Option<Result<ElementGenerator, XMLGeneratorError>> {
    match content {
        SchemaContent::Include(_) => unimplemented!("Include"),
        SchemaContent::Import(_) => unimplemented!("Import"),
        SchemaContent::Redefine(_) => unimplemented!("Redefine"),
        SchemaContent::Override(_) => unimplemented!("Override"),
        SchemaContent::Annotation(_) => None,
        SchemaContent::DefaultOpenContent(_) => unimplemented!("DefaultOpenContent"),
        SchemaContent::SimpleType(_) => None,
        SchemaContent::ComplexType(_) => None,
        SchemaContent::Group(_) => unimplemented!("Top-level group not supported"),
        SchemaContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        SchemaContent::Element(x) => Some(get_element_type(x, regex_translator)),
        SchemaContent::Attribute(_) => unimplemented!("Attribute"),
        SchemaContent::Notation(_) => unimplemented!("Notation"),
    }
}

pub(crate) fn fetch_elements(
    schemas: &Schemas,
    regex_translator: &RegexTranslator,
) -> Result<Vec<ElementGenerator>, XMLGeneratorError> {
    let mut elements = vec![];
    for (_schema_id, schema_info) in schemas.schemas() {
        let schema = &schema_info.schema;
        for content in &schema.content {
            let element = fetch_element(&content, regex_translator);
            if let Some(e) = element {
                elements.push(e?);
            }
        }
    }

    Ok(elements)
}
