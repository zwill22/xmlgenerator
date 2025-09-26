use crate::attribute_generator::AttributeGenerator;
use crate::element_generator::ElementGenerator;
use crate::group_generator::GroupGenerator;
use crate::type_generator::TypeGenerator;
use crate::type_info::{generate_type_info, get_qname};
use xsd_parser::Schemas;
use xsd_parser::models::schema::MaxOccurs;
use xsd_parser::models::schema::xs::{
    AttributeType, ComplexBaseType, ComplexBaseTypeContent, ElementType, ElementTypeContent,
    GroupType, GroupTypeContent, SchemaContent, SimpleBaseType,
};

fn get_simple_type(simple: &SimpleBaseType) -> TypeGenerator {
    let mut generator = TypeGenerator::new();
    generator.name = simple.name.clone().unwrap_or("".to_string());
    if generator.name.is_empty() {
        unimplemented!("Empty type");
    }

    if simple.final_.is_some() {
        unimplemented!("Final");
    }

    let type_info = generate_type_info(&simple.content);

    generator.type_info = Some(type_info);

    generator
}

fn fetch_type(content: &SchemaContent) -> Option<TypeGenerator> {
    match content {
        SchemaContent::Include(_) => unimplemented!("Include"),
        SchemaContent::Import(_) => unimplemented!("Import"),
        SchemaContent::Redefine(_) => unimplemented!("Redefine"),
        SchemaContent::Override(_) => unimplemented!("Override"),
        SchemaContent::Annotation(_) => None,
        SchemaContent::DefaultOpenContent(_) => unimplemented!("DefaultOpenContent"),
        SchemaContent::SimpleType(x) => Some(get_simple_type(x)),
        SchemaContent::ComplexType(x) => Some(get_complex_type(x)),
        SchemaContent::Group(_) => unimplemented!("Top-level group not supported"),
        SchemaContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        SchemaContent::Element(_) => None,
        SchemaContent::Attribute(_) => unimplemented!("Attribute"),
        SchemaContent::Notation(_) => unimplemented!("Notation"),
    }
}

fn get_element_content(content: &ElementTypeContent) -> TypeGenerator {
    match content {
        ElementTypeContent::Annotation(_) => unimplemented!("Annotation"),
        ElementTypeContent::SimpleType(x) => get_simple_type(x),
        ElementTypeContent::ComplexType(x) => get_complex_type(x),
        ElementTypeContent::Alternative(_) => unimplemented!("Alternative"),
        ElementTypeContent::Unique(_) => unimplemented!("Unique"),
        ElementTypeContent::Key(_) => unimplemented!("Key"),
        ElementTypeContent::Keyref(_) => unimplemented!("Keyref"),
    }
}

pub(crate) fn get_element_type(element: &ElementType) -> ElementGenerator {
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
        unimplemented!("Element Substitution Groups");
    }

    generator.min = element.min_occurs;

    generator.max = match element.max_occurs {
        MaxOccurs::Unbounded => None,
        MaxOccurs::Bounded(x) => Some(x),
    };

    if element.default.is_some() {
        unimplemented!("Default Element");
    }

    if element.fixed.is_some() {
        unimplemented!("Fixed elements");
    }

    if element.nillable.is_some() {
        unimplemented!("Nillable elements");
    }

    if element.abstract_ {
        unimplemented!("Abstract elements");
    }

    if element.final_.is_some() {
        unimplemented!("Final elements");
    }

    if element.block.is_some() {
        unimplemented!("Block elements");
    }

    if element.form.is_some() {
        unimplemented!("Form elements");
    }

    if element.target_namespace.is_some() {
        unimplemented!("Namespace elements");
    }

    for content in &element.content {
        let result = get_element_content(content);
        generator.contents.push(result);
    }

    generator
}

fn get_group_content(content: &GroupTypeContent) -> ElementGenerator {
    match content {
        GroupTypeContent::Annotation(_) => unimplemented!("Annotation"),
        GroupTypeContent::Element(x) => get_element_type(x),
        GroupTypeContent::Group(_) => unimplemented!("Embedded groups"),
        GroupTypeContent::All(_) => unimplemented!("Embedded groups"),
        GroupTypeContent::Choice(_) => unimplemented!("Embedded groups"),
        GroupTypeContent::Sequence(_) => unimplemented!("Embedded groups"),
        GroupTypeContent::Any(_) => unimplemented!("Any"),
    }
}

fn get_group(group: &GroupType) -> GroupGenerator {
    let mut generator = GroupGenerator::new();

    if group.name.is_some() {
        unimplemented!("Named groups");
    }

    if group.ref_.is_some() {
        unimplemented!("Group references");
    }

    generator.min = group.min_occurs;

    generator.max = match group.max_occurs {
        MaxOccurs::Unbounded => None,
        MaxOccurs::Bounded(x) => Some(x),
    };

    for content in &group.content {
        let element = get_group_content(content);
        generator.elements.push(element);
    }

    generator
}

fn get_complex_group(content: &ComplexBaseTypeContent) -> Option<GroupGenerator> {
    match content {
        ComplexBaseTypeContent::Annotation(_) => unimplemented!("Annotation"),
        ComplexBaseTypeContent::SimpleContent(_) => unimplemented!("SimpleContent"),
        ComplexBaseTypeContent::ComplexContent(_) => unimplemented!("ComplexContent"),
        ComplexBaseTypeContent::OpenContent(_) => unimplemented!("OpenContent"),
        ComplexBaseTypeContent::Group(x) => Option::from(get_group(x)),
        ComplexBaseTypeContent::All(x) => Option::from(get_group(x)),
        ComplexBaseTypeContent::Choice(x) => Option::from(get_group(x)),
        ComplexBaseTypeContent::Sequence(x) => Option::from(get_group(x)),
        ComplexBaseTypeContent::Attribute(_) => None,
        ComplexBaseTypeContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        ComplexBaseTypeContent::AnyAttribute(_) => unimplemented!("AnyAttribute"),
        ComplexBaseTypeContent::Assert(_) => unimplemented!("Assert"),
    }
}

fn get_attribute(attribute: &AttributeType) -> AttributeGenerator {
    let mut generator = AttributeGenerator::new();
    generator.name = attribute.name.clone().unwrap_or("".to_string());

    if let Some(attribute_type) = &attribute.type_ {
        generator.type_name = get_qname(attribute_type);
    }

    generator.attribute_type = attribute.use_.clone();

    if attribute.ref_.is_some() {
        unimplemented!("Attribute references");
    }

    if attribute.default.is_some() {
        unimplemented!("Default attribute");
    }

    if attribute.fixed.is_some() {
        unimplemented!("Fixed attribute");
    }

    if attribute.form.is_some() {
        unimplemented!("Form attribute");
    }

    if attribute.target_namespace.is_some() {
        unimplemented!("Target namespace attribute");
    }

    if attribute.inheritable.is_some() {
        unimplemented!("Inheritable attribute");
    }

    if attribute.annotation.is_some() {
        unimplemented!("Annotation");
    }

    if attribute.simple_type.is_some() {
        unimplemented!("Simple type attribute");
    }

    generator
}

fn get_complex_attributes(content: &ComplexBaseTypeContent) -> Option<AttributeGenerator> {
    match content {
        ComplexBaseTypeContent::Annotation(_) => unimplemented!("Annotation"),
        ComplexBaseTypeContent::SimpleContent(_) => unimplemented!("SimpleContent"),
        ComplexBaseTypeContent::ComplexContent(_) => unimplemented!("ComplexContent"),
        ComplexBaseTypeContent::OpenContent(_) => unimplemented!("OpenContent"),
        ComplexBaseTypeContent::Group(_) => None,
        ComplexBaseTypeContent::All(_) => None,
        ComplexBaseTypeContent::Choice(_) => None,
        ComplexBaseTypeContent::Sequence(_) => None,
        ComplexBaseTypeContent::Attribute(x) => Some(get_attribute(x)),
        ComplexBaseTypeContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        ComplexBaseTypeContent::AnyAttribute(_) => unimplemented!("AnyAttribute"),
        ComplexBaseTypeContent::Assert(_) => unimplemented!("Assert"),
    }
}

fn get_complex_type(complex: &ComplexBaseType) -> TypeGenerator {
    let mut generator = TypeGenerator::new();
    generator.name = complex.name.clone().unwrap_or("".to_string());

    if complex.mixed.is_some() {
        unimplemented!("Mixed types");
    }

    if complex.abstract_ {
        unimplemented!("Abstract types");
    }

    if complex.final_.is_some() {
        unimplemented!("Final types");
    }

    if complex.block.is_some() {
        unimplemented!("Block types");
    }

    let default_attributes_apply = complex.default_attributes_apply;
    if !default_attributes_apply {
        unimplemented!("Non-default attributes");
    }

    for content in &complex.content {
        if let Some(group) = get_complex_group(content) {
            generator.groups.push(group);
        }
        if let Some(attribute) = get_complex_attributes(content) {
            generator.attributes.push(attribute);
        }
    }

    generator
}

pub(crate) fn fetch_types(schemas: &Schemas) -> Vec<TypeGenerator> {
    let mut types = vec![];
    for (_schema_id, schema_info) in schemas.schemas() {
        let schema = &schema_info.schema;
        for content in &schema.content {
            let data_type = fetch_type(content);
            if data_type.is_some() {
                types.push(data_type.unwrap());
            }
        }
    }

    types
}
