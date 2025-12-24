use crate::XMLGeneratorError;
use crate::attribute_generator::AttributeGenerator;
use crate::element_generator::ElementGenerator;
use crate::error::unimplemented;
use crate::group_generator::GroupGenerator;
use crate::type_generator::TypeGenerator;
use crate::type_info::{generate_type_info, get_qname};
use regextranslator::RegexTranslator;
use xsd_parser::Schemas;
use xsd_parser::models::schema::MaxOccurs;
use xsd_parser::models::schema::xs::{
    AttributeType, ComplexBaseType, ComplexBaseTypeContent, ElementType, ElementTypeContent,
    GroupType, GroupTypeContent, SchemaContent, SimpleBaseType,
};

fn get_simple_type(
    simple: &SimpleBaseType,
    regex_translator: &RegexTranslator,
) -> Result<TypeGenerator, XMLGeneratorError> {
    let mut generator = TypeGenerator::new();
    generator.name = simple.name.clone().unwrap_or("".to_string());
    if generator.name.is_empty() {
        return unimplemented("Empty type");
    }

    if simple.final_.is_some() {
        return unimplemented("Final");
    }

    let type_info = generate_type_info(&simple.content, regex_translator)?;

    generator.type_info = Some(type_info);

    Ok(generator)
}

fn fetch_type(
    content: &SchemaContent,
    regex_translator: &RegexTranslator,
) -> Option<Result<TypeGenerator, XMLGeneratorError>> {
    match content {
        SchemaContent::Include(_) => None,
        SchemaContent::Import(_) => None,
        SchemaContent::Redefine(_) => Some(unimplemented("Redefine")),
        SchemaContent::Override(_) => Some(unimplemented("Override")),
        SchemaContent::Annotation(_) => None,
        SchemaContent::DefaultOpenContent(_) => Some(unimplemented("DefaultOpenContent")),
        SchemaContent::SimpleType(x) => Some(get_simple_type(x, regex_translator)),
        SchemaContent::ComplexType(x) => Some(get_complex_type(x, regex_translator)),
        SchemaContent::Group(_) => Some(unimplemented("Top-level group not supported")),
        SchemaContent::AttributeGroup(_) => Some(unimplemented("AttributeGroup")),
        SchemaContent::Element(_) => None,
        SchemaContent::Attribute(_) => Some(unimplemented("Attribute")),
        SchemaContent::Notation(_) => Some(unimplemented("Notation")),
    }
}

fn get_element_content(
    content: &ElementTypeContent,
    regex_translator: &RegexTranslator,
) -> Result<TypeGenerator, XMLGeneratorError> {
    match content {
        ElementTypeContent::Annotation(_) => unimplemented("Annotation"),
        ElementTypeContent::SimpleType(x) => get_simple_type(x, regex_translator),
        ElementTypeContent::ComplexType(x) => get_complex_type(x, regex_translator),
        ElementTypeContent::Alternative(_) => unimplemented("Alternative"),
        ElementTypeContent::Unique(_) => unimplemented("Unique"),
        ElementTypeContent::Key(_) => unimplemented("Key"),
        ElementTypeContent::Keyref(_) => unimplemented("Keyref"),
    }
}

pub(crate) fn get_element_type(
    element: &ElementType,
    regex_translator: &RegexTranslator,
) -> Result<ElementGenerator, XMLGeneratorError> {
    let mut generator = ElementGenerator::new();

    generator.name = element.name.clone();

    if let Some(element_ref) = &element.ref_ {
        let reference = get_qname(element_ref);
        generator.reference = Some(reference);
    }

    if let Some(element_type) = &element.type_ {
        let type_info = get_qname(element_type);
        generator.type_info = Some(type_info);
    }

    if element.substitution_group.is_some() {
        return unimplemented("Element Substitution Groups");
    }

    generator.min = element.min_occurs;

    generator.max = match element.max_occurs {
        MaxOccurs::Unbounded => None,
        MaxOccurs::Bounded(x) => Some(x),
    };

    if element.default.is_some() {
        return unimplemented("Default Element");
    }

    if element.fixed.is_some() {
        return unimplemented("Fixed elements");
    }

    if element.nillable.is_some() {
        return unimplemented("Nillable elements");
    }

    if element.abstract_ {
        return unimplemented("Abstract elements");
    }

    if element.final_.is_some() {
        return unimplemented("Final elements");
    }

    if element.block.is_some() {
        return unimplemented("Block elements");
    }

    if element.form.is_some() {
        return unimplemented("Form elements");
    }

    if element.target_namespace.is_some() {
        return unimplemented("Namespace elements");
    }

    for content in &element.content {
        let result = get_element_content(content, regex_translator)?;
        generator.contents.push(result);
    }

    Ok(generator)
}

fn get_group_content(
    content: &GroupTypeContent,
    regex_translator: &RegexTranslator,
) -> Result<ElementGenerator, XMLGeneratorError> {
    match content {
        GroupTypeContent::Annotation(_) => unimplemented("Annotation"),
        GroupTypeContent::Element(x) => get_element_type(x, regex_translator),
        GroupTypeContent::Group(_) => unimplemented("Embedded groups"),
        GroupTypeContent::All(_) => unimplemented("Embedded groups"),
        GroupTypeContent::Choice(_) => unimplemented("Embedded groups"),
        GroupTypeContent::Sequence(_) => unimplemented("Embedded groups"),
        GroupTypeContent::Any(_) => unimplemented("Any"),
    }
}

fn get_group(
    group: &GroupType,
    regex_translator: &RegexTranslator,
) -> Result<GroupGenerator, XMLGeneratorError> {
    let mut generator = GroupGenerator::new();

    if group.name.is_some() {
        return unimplemented("Named groups");
    }

    if group.ref_.is_some() {
        return unimplemented("Group references");
    }

    generator.min = group.min_occurs;

    generator.max = match group.max_occurs {
        MaxOccurs::Unbounded => None,
        MaxOccurs::Bounded(x) => Some(x),
    };

    for content in &group.content {
        let element = get_group_content(content, regex_translator)?;
        generator.elements.push(element);
    }

    Ok(generator)
}

fn get_complex_group(
    content: &ComplexBaseTypeContent,
    regex_translator: &RegexTranslator,
) -> Option<Result<GroupGenerator, XMLGeneratorError>> {
    match content {
        ComplexBaseTypeContent::Annotation(_) => Some(unimplemented("Annotation")),
        ComplexBaseTypeContent::SimpleContent(_) => Some(unimplemented("SimpleContent")),
        ComplexBaseTypeContent::ComplexContent(_) => Some(unimplemented("ComplexContent")),
        ComplexBaseTypeContent::OpenContent(_) => Some(unimplemented("OpenContent")),
        ComplexBaseTypeContent::Group(x) => Some(get_group(x, regex_translator)),
        ComplexBaseTypeContent::All(x) => Some(get_group(x, regex_translator)),
        ComplexBaseTypeContent::Choice(x) => Some(get_group(x, regex_translator)),
        ComplexBaseTypeContent::Sequence(x) => Some(get_group(x, regex_translator)),
        ComplexBaseTypeContent::Attribute(_) => None,
        ComplexBaseTypeContent::AttributeGroup(_) => Some(unimplemented("AttributeGroup")),
        ComplexBaseTypeContent::AnyAttribute(_) => Some(unimplemented("AnyAttribute")),
        ComplexBaseTypeContent::Assert(_) => Some(unimplemented("Assert")),
    }
}

fn get_attribute(attribute: &AttributeType) -> Result<AttributeGenerator, XMLGeneratorError> {
    let mut generator = AttributeGenerator::new();
    generator.name = attribute.name.clone().unwrap_or("".to_string());

    if let Some(attribute_type) = &attribute.type_ {
        generator.type_name = get_qname(attribute_type);
    }

    generator.attribute_type = attribute.use_;

    if attribute.ref_.is_some() {
        return unimplemented("Attribute references");
    }

    if attribute.default.is_some() {
        return unimplemented("Default attribute");
    }

    if attribute.fixed.is_some() {
        return unimplemented("Fixed attribute");
    }

    if attribute.form.is_some() {
        return unimplemented("Form attribute");
    }

    if attribute.target_namespace.is_some() {
        return unimplemented("Target namespace attribute");
    }

    if attribute.inheritable.is_some() {
        return unimplemented("Inheritable attribute");
    }

    if attribute.annotation.is_some() {
        return unimplemented("Annotation");
    }

    if attribute.simple_type.is_some() {
        return unimplemented("Simple type attribute");
    }

    Ok(generator)
}

fn get_complex_attributes(
    content: &ComplexBaseTypeContent,
) -> Option<Result<AttributeGenerator, XMLGeneratorError>> {
    match content {
        ComplexBaseTypeContent::Annotation(_) => Some(unimplemented("Annotation")),
        ComplexBaseTypeContent::SimpleContent(_) => Some(unimplemented("SimpleContent")),
        ComplexBaseTypeContent::ComplexContent(_) => Some(unimplemented("ComplexContent")),
        ComplexBaseTypeContent::OpenContent(_) => Some(unimplemented("OpenContent")),
        ComplexBaseTypeContent::Group(_) => None,
        ComplexBaseTypeContent::All(_) => None,
        ComplexBaseTypeContent::Choice(_) => None,
        ComplexBaseTypeContent::Sequence(_) => None,
        ComplexBaseTypeContent::Attribute(x) => Some(get_attribute(x)),
        ComplexBaseTypeContent::AttributeGroup(_) => Some(unimplemented("AttributeGroup")),
        ComplexBaseTypeContent::AnyAttribute(_) => Some(unimplemented("AnyAttribute")),
        ComplexBaseTypeContent::Assert(_) => Some(unimplemented("Assert")),
    }
}

fn get_complex_type(
    complex: &ComplexBaseType,
    regex_translator: &RegexTranslator,
) -> Result<TypeGenerator, XMLGeneratorError> {
    let mut generator = TypeGenerator::new();
    generator.name = complex.name.clone().unwrap_or("".to_string());

    if complex.mixed.is_some() {
        return unimplemented("Mixed types");
    }

    if complex.abstract_ {
        return unimplemented("Abstract types");
    }

    if complex.final_.is_some() {
        return unimplemented("Final types");
    }

    if complex.block.is_some() {
        return unimplemented("Block types");
    }

    let default_attributes_apply = complex.default_attributes_apply;
    if !default_attributes_apply {
        return unimplemented("Non-default attributes");
    }

    for content in &complex.content {
        if let Some(group) = get_complex_group(content, regex_translator) {
            generator.groups.push(group?);
        }
        if let Some(attribute) = get_complex_attributes(content) {
            generator.attributes.push(attribute?);
        }
    }

    Ok(generator)
}

pub(crate) fn fetch_types(
    schemas: &Schemas,
    regex_translator: &RegexTranslator,
) -> Result<Vec<TypeGenerator>, XMLGeneratorError> {
    let mut types = vec![];
    for (_schema_id, schema_info) in schemas.schemas() {
        let schema = &schema_info.schema;
        for content in &schema.content {
            if let Some(data_type) = fetch_type(content, regex_translator) {
                types.push(data_type?);
            }
        }
    }

    Ok(types)
}
