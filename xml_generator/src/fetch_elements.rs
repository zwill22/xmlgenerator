use crate::element_generator::ElementGenerator;
use crate::fetch_types::get_element_type;
use xsd_parser::Schemas;
use xsd_parser::models::schema::xs::SchemaContent;

fn fetch_element(content: &SchemaContent) -> Option<ElementGenerator> {
    match content {
        SchemaContent::Include(_) => unimplemented!("Include"),
        SchemaContent::Import(_) => unimplemented!("Import"),
        SchemaContent::Redefine(_) => unimplemented!("Redefine"),
        SchemaContent::Override(_) => unimplemented!("Override"),
        SchemaContent::Annotation(_) => unimplemented!("Annotation"),
        SchemaContent::DefaultOpenContent(_) => unimplemented!("DefaultOpenContent"),
        SchemaContent::SimpleType(_) => None,
        SchemaContent::ComplexType(_) => None,
        SchemaContent::Group(_) => unimplemented!("Top-level group not supported"),
        SchemaContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        SchemaContent::Element(x) => Some(get_element_type(x)),
        SchemaContent::Attribute(_) => unimplemented!("Attribute"),
        SchemaContent::Notation(_) => unimplemented!("Notation"),
    }
}

pub(crate) fn fetch_elements(schemas: &Schemas) -> Vec<ElementGenerator> {
    let mut elements = vec![];
    for (_schema_id, schema) in schemas.schemas() {
        for content in &schema.content {
            let element = fetch_element(content);
            if element.is_some() {
                elements.push(element.unwrap());
            }
        }
    }

    elements
}
