use crate::type_generator::TypeGenerator;
use xsd_parser::models::schema::xs::AttributeUseType;

pub struct AttributeGenerator {
    pub(crate) name: String,
    pub(crate) attribute_type: AttributeUseType,
    pub(crate) type_generator: Option<TypeGenerator>,
    pub(crate) ref_name: Option<String>,
    pub(crate) type_name: Option<String>,
}

impl AttributeGenerator {
    pub(crate) fn new() -> Self {
        AttributeGenerator {
            name: String::new(),
            attribute_type: AttributeUseType::Required,
            type_generator: None,
            ref_name: None,
            type_name: None,
        }
    }
}

impl PartialEq for AttributeGenerator {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        if !self.type_generator.eq(&other.type_generator) {
            return false;
        }

        if !self.ref_name.eq(&other.ref_name) {
            return false;
        }

        if !self.type_name.eq(&other.type_name) {
            return false;
        }

        true
    }
}
