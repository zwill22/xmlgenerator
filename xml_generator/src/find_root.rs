use crate::element_generator::ElementGenerator;
use crate::error::XMLGeneratorError;
use crate::type_generator::TypeGenerator;

fn get_content_list(generator: &TypeGenerator) -> Result<Vec<String>, XMLGeneratorError> {
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
    generators: &'a [ElementGenerator],
    field: &String,
) -> Option<&'a ElementGenerator> {
    for generator in generators.iter() {
        if let Some(name) = &generator.name
            && name.eq(field)
        {
            return Option::from(generator);
        }
    }

    None
}

pub(crate) fn find_root_element(
    generators: &[ElementGenerator],
) -> Result<&ElementGenerator, XMLGeneratorError> {
    if generators.is_empty() {
        return Err(XMLGeneratorError::NoElementsError);
    }

    let mut all_fields = vec![];
    let mut all_types = vec![];
    for generator in generators.iter() {
        if let Some(reference) = &generator.reference {
            all_fields.push(reference.clone());
        }
        if let Some(type_info) = &generator.type_info
            && !type_info.is_empty()
        {
            all_types.push(type_info);
        }

        for content in generator.types.iter() {
            let content_list = get_content_list(content)?;
            for item in content_list {
                all_fields.push(item);
            }
        }
    }

    let mut dependent_elements = vec![];
    for field in all_fields {
        let structure = get_field_struct(generators, &field);
        if let Some(item) = structure {
            dependent_elements.push(item);
        }
    }

    let mut independent_elements = vec![];
    for generator in generators.iter() {
        if !dependent_elements.contains(&generator) {
            independent_elements.push(generator);
        }
    }

    if independent_elements.is_empty() {
        return Err(XMLGeneratorError::NoIndependentElementsError);
    }

    if independent_elements.len() > 1 {
        return Err(XMLGeneratorError::MultipleRootsError);
    }

    for generator in generators.iter() {
        if independent_elements.contains(&generator) {
            return Ok(generator);
        }
    }

    unreachable!();
}
