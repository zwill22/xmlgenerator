use crate::element::Element;
use crate::error::XMLGeneratorError;

fn get_field_struct<'a>(generators: &'a [Element], field: &String) -> Option<&'a Element<'a>> {
    for generator in generators.iter() {
        if let Ok(name) = generator.get_name() {
            if name.eq(field) {
                return Some(generator);
            }
        }
    }

    None
}

pub(crate) fn find_root_element<'a>(
    elements: &'a [Element<'a>],
) -> Result<&'a Element<'a>, XMLGeneratorError> {
    if elements.is_empty() {
        return Err(XMLGeneratorError::NoElementsError);
    }

    let mut all_fields = vec![];
    let mut all_types = vec![];
    for generator in elements.iter() {
        generator.get_content(&mut all_fields, &mut all_types)?;
    }

    let mut dependent_elements = vec![];
    for field in all_fields {
        let structure = get_field_struct(elements, &field);
        if let Some(item) = structure {
            dependent_elements.push(item);
        }
    }

    let mut independent_elements = vec![];
    for generator in elements.iter() {
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

    for generator in elements.iter() {
        if independent_elements.contains(&generator) {
            return Ok(generator);
        }
    }

    unreachable!();
}
