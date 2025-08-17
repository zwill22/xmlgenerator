use crate::element_generator::ElementGenerator;
use crate::error::XMLGeneratorError;
use crate::type_generator::TypeGenerator;
use fake::{Fake, Faker};
use xml_builder::XMLElement;

pub(crate) fn generate_reference(
    reference: &String,
    data_types: &Vec<TypeGenerator>,
    elements: &Vec<ElementGenerator>,
) -> Result<XMLElement, XMLGeneratorError> {
    for element in elements.iter() {
        let name = element.get_name()?;
        if name.eq(reference) {
            return element.generate(data_types, elements);
        }
    }

    Err(XMLGeneratorError::XMLBuilderError(
        "Reference not found".to_string(),
    ))
}

fn make_fake<Output: fake::Dummy<Faker> + ToString>() -> Option<String> {
    Option::from(Faker.fake::<Output>().to_string())
}

pub(crate) fn generate(type_name: &String) -> Option<String> {
    match type_name.as_str() {
        "boolean" => make_fake::<bool>(),
        "decimal" => make_fake::<f32>(),
        "double" => make_fake::<f64>(),
        "integer" => make_fake::<i32>(),
        "positiveInteger" => make_fake::<u32>(),
        "string" => make_fake::<String>(),
        _ => None,
    }
}

pub fn generate_type_output(
    xml_element: &mut XMLElement,
    type_name: &String,
    data_types: &Vec<TypeGenerator>,
    elements: &Vec<ElementGenerator>,
) -> Result<(), XMLGeneratorError> {
    let output = generate(type_name);
    if output.is_some() {
        let result = xml_element.add_text(output.unwrap());
        return match result {
            Ok(_) => Ok(()),
            Err(err) => Err(XMLGeneratorError::XMLBuilderError(err.to_string())),
        };
    }

    for data_type in data_types {
        if data_type.name.eq(type_name) {
            return data_type.generate(xml_element, data_types, elements);
        }
    }

    Err(XMLGeneratorError::DataTypeError(format!(
        "Cannot find data type: {}",
        type_name
    )))
}
