use crate::element_generator::ElementGenerator;
use crate::error::XMLGeneratorError;
use crate::type_generator::TypeGenerator;

fn get_content_list(generator: &TypeGenerator) -> Result<Vec<&String>, XMLGeneratorError> {
    let mut output = vec![];
    for element in generator.elements.iter() {
        let name = element.get_name()?;
        output.push(name);
    }

    for group in generator.groups.iter() {
        for element in group.elements.iter() {
            let name = element.get_name()?;
            output.push(name);
        }
    }

    Ok(output)
}

fn get_field_struct<'a>(
    generators: &'a Vec<ElementGenerator>,
    field: &String,
) -> Option<&'a ElementGenerator> {
    for generator in generators.iter() {
        if let Some(name) = &generator.name {
            if name.eq(field) {
                return Option::from(generator);
            }
        }
    }

    None
}

pub(crate) fn find_root_element(
    generators: &Vec<ElementGenerator>,
) -> Result<&ElementGenerator, XMLGeneratorError> {
    if generators.is_empty() {
        return Err(XMLGeneratorError::DataTypesFormatError(
            "No elements found".to_string(),
        ));
    }

    let mut all_fields: Vec<&String> = vec![];
    let mut all_types: Vec<&String> = vec![];
    for generator in generators.iter() {
        if generator.reference.is_some() {
            let reference = generator.reference.as_ref().unwrap();
            all_fields.push(reference);
        }
        if generator.type_info.is_some() {
            let type_info = generator.type_info.as_ref().unwrap();
            if type_info.len() > 0 {
                all_types.push(type_info);
            }
        }

        for content in generator.contents.iter() {
            let content_list = get_content_list(content)?;
            for item in content_list {
                all_fields.push(&item);
            }
        }
    }

    let mut dependent_elements = vec![];
    for field in all_fields {
        let structure = get_field_struct(&generators, field);
        if structure.is_some() {
            dependent_elements.push(structure.unwrap());
        }
    }

    let mut independent_elements = vec![];
    for generator in generators.iter() {
        if !dependent_elements.contains(&generator) {
            independent_elements.push(generator);
        }
    }

    if independent_elements.is_empty() {
        return Err(XMLGeneratorError::DataTypesFormatError(
            "No independent elements found".to_string(),
        ));
    }

    if independent_elements.len() > 1 {
        for item in dependent_elements.iter() {
            println!("Dependent element: {:?}", item.name);
        }
        for item in independent_elements.iter() {
            println!("Independent element: {:?}", item.name);
        }
        return Err(XMLGeneratorError::DataTypesFormatError(
            "Multiple independent (root) elements found!".to_string(),
        ));
    }

    for generator in generators.iter() {
        if independent_elements.contains(&generator) {
            return Ok(generator);
        }
    }

    unreachable!();
}
