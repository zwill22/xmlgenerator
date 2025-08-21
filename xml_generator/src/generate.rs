use crate::element_generator::ElementGenerator;
use crate::error::XMLGeneratorError;
use crate::type_generator::TypeGenerator;
use fake::{Fake, Faker};
use rand::{Rng, SeedableRng};
use xml_builder::XMLElement;
use rand_regex;
use rand_xorshift::XorShiftRng;

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

pub(crate) fn generate_type(type_name: &String) -> Option<String> {
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

fn generate_regex(type_name: &String, pattern: &String) -> Option<String> {
    if type_name.to_lowercase().ne("string") {
        return None;
    }

    let mut rng = XorShiftRng::from_seed([0; 16]);

    // creates a generator for sampling strings
    let generator = rand_regex::Regex::compile(pattern, 1).unwrap();

    let samples = (&mut rng).sample_iter(&generator).take(1).collect::<Vec<String>>();

    if samples.is_empty() {
        return None;
    }

    samples.last().cloned()
}


pub(crate) fn generate(type_name: &Vec<String>) -> Option<String> {
    if type_name.len() == 1 {
        let name = type_name.first().unwrap();

        return generate_type(name);
    } else if type_name.len() == 2 {
        let name = type_name.first().unwrap();
        let pattern = type_name.last().unwrap();

        return generate_regex(name, pattern);
    }

    None
}

pub fn generate_type_output(
    xml_element: &mut XMLElement,
    type_name: &String,
    data_types: &Vec<TypeGenerator>,
    elements: &Vec<ElementGenerator>,
) -> Result<(), XMLGeneratorError> {
    let output = generate_type(type_name);
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
