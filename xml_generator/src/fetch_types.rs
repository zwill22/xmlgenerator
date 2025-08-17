use crate::attribute::AttributeInfo;
use crate::element_generator::ElementGenerator;
use crate::group::GroupInfo;
use crate::restriction::RestrictionInfo;
use crate::type_generator::TypeGenerator;
use xsd_parser::Schemas;
use xsd_parser::models::schema::xs::{
    AttributeType, ComplexBaseType, ComplexBaseTypeContent, ElementType, ElementTypeContent, Facet,
    FacetType, GroupType, GroupTypeContent, Restriction, RestrictionContent, SchemaContent,
    SimpleBaseType, SimpleBaseTypeContent,
};
use xsd_parser::models::schema::{MaxOccurs, QName};

fn get_qname(qname: QName) -> String {
    String::from_utf8(qname.local_name().to_vec()).unwrap()
}

fn get_facet_type(facet_type: &FacetType) -> String {
    if facet_type.fixed {
        unimplemented!("Fixed facet type");
    }

    if facet_type.annotation.is_some() {
        unimplemented!("Annotation");
    }

    facet_type.value.clone()
}

fn get_facet(facet: &Facet) -> String {
    match facet {
        Facet::MinExclusive(x) => get_facet_type(x),
        Facet::MinInclusive(x) => get_facet_type(x),
        Facet::MaxExclusive(x) => get_facet_type(x),
        Facet::MaxInclusive(x) => get_facet_type(x),
        Facet::TotalDigits(x) => get_facet_type(x),
        Facet::FractionDigits(x) => get_facet_type(x),
        Facet::Length(x) => get_facet_type(x),
        Facet::MinLength(x) => get_facet_type(x),
        Facet::MaxLength(x) => get_facet_type(x),
        Facet::Enumeration(x) => get_facet_type(x),
        Facet::WhiteSpace(x) => get_facet_type(x),
        Facet::Pattern(x) => get_facet_type(x),
        Facet::Assertion(_) => unimplemented!("Assertion"),
        Facet::ExplicitTimezone(x) => get_facet_type(x),
    }
}

fn get_restriction_content(content: &RestrictionContent) -> String {
    match content {
        RestrictionContent::Annotation(_) => unimplemented!("Annotation"),
        RestrictionContent::SimpleType(_) => unimplemented!("SimpleType"),
        RestrictionContent::Facet(x) => get_facet(x),
    }
}

fn get_restriction(restriction: &Restriction) -> RestrictionInfo {
    let mut info = RestrictionInfo::new();
    if restriction.base.is_some() {
        info.name = get_qname(restriction.base.clone().unwrap());
    }

    for content in &restriction.content {
        let facet = get_restriction_content(content);
        info.facets.push(facet);
    }

    info
}

fn get_content_restriction(content: &SimpleBaseTypeContent) -> RestrictionInfo {
    match content {
        SimpleBaseTypeContent::Annotation(_) => unimplemented!("Annotation"),
        SimpleBaseTypeContent::Restriction(x) => get_restriction(x),
        SimpleBaseTypeContent::List(_) => unimplemented!("List"),
        SimpleBaseTypeContent::Union(_) => unimplemented!("Union"),
    }
}

fn get_simple_type(simple: &SimpleBaseType) -> TypeGenerator {
    let mut type_generator = TypeGenerator::new();
    type_generator.name = simple.name.clone().unwrap_or("".to_string());
    if type_generator.name.is_empty() {
        unimplemented!("Empty type");
    }

    if simple.final_.is_some() {
        unimplemented!("Final");
    }

    let mut restrictions = vec![];
    for content in &simple.content {
        let restriction = get_content_restriction(content);
        restrictions.push(restriction);
    }

    if restrictions.is_empty() {
        type_generator.type_info.push("string".to_string());

        return type_generator;
    }

    for restriction in &restrictions {
        type_generator.type_info.push(restriction.name.clone());
        for facet in &restriction.facets {
            type_generator.type_info.push(facet.clone());
        }
    }

    type_generator
}

fn fetch_type(content: &SchemaContent) -> Option<TypeGenerator> {
    match content {
        SchemaContent::Include(_) => unimplemented!("Include"),
        SchemaContent::Import(_) => unimplemented!("Import"),
        SchemaContent::Redefine(_) => unimplemented!("Redefine"),
        SchemaContent::Override(_) => unimplemented!("Override"),
        SchemaContent::Annotation(_) => unimplemented!("Annotation"),
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

    if element.ref_.is_some() {
        if generator.name.is_some() {
            panic!("Name already defined.");
        }
        let element_ref = element.ref_.clone().unwrap();
        let reference = Some(get_qname(element_ref));
        if let Some(name) = reference {
            generator.reference = Some(name);
        }
    }

    if element.type_.is_some() {
        let element_type = element.type_.clone().unwrap();
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

fn get_group(group: &GroupType) -> GroupInfo {
    let mut group_info = GroupInfo::new();

    if group.name.is_some() {
        unimplemented!("Named groups");
    }

    if group.ref_.is_some() {
        unimplemented!("Group references");
    }

    group_info.min = group.min_occurs;

    group_info.max = match group.max_occurs {
        MaxOccurs::Unbounded => None,
        MaxOccurs::Bounded(x) => Some(x),
    };

    for content in &group.content {
        let element = get_group_content(content);
        group_info.elements.push(element);
    }

    group_info
}

fn get_complex_group(content: &ComplexBaseTypeContent) -> Option<GroupInfo> {
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

fn get_attribute(attribute: &AttributeType) -> AttributeInfo {
    let mut attribute_info = AttributeInfo::new();
    attribute_info.name = attribute.name.clone().unwrap_or("".to_string());

    if attribute.ref_.is_some() {
        let attribute_ref = attribute.ref_.clone().unwrap();
        attribute_info.ref_name = Some(get_qname(attribute_ref));
    }

    if attribute.type_.is_some() {
        let attribute_type = attribute.type_.clone().unwrap();
        attribute_info.type_name = Some(get_qname(attribute_type));
    }

    attribute_info.attribute_type = attribute.use_.clone();

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

    attribute_info
}

fn get_complex_attributes(content: &ComplexBaseTypeContent) -> Option<AttributeInfo> {
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
    let mut type_generator = TypeGenerator::new();
    type_generator.name = complex.name.clone().unwrap_or("".to_string());

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
        let group = get_complex_group(content);
        if group.is_some() {
            type_generator.groups.push(group.unwrap());
        }
        let attribute = get_complex_attributes(content);
        if attribute.is_some() {
            type_generator.attributes.push(attribute.unwrap());
        }
    }

    type_generator
}

pub(crate) fn fetch_types(schemas: &Schemas) -> Vec<TypeGenerator> {
    let mut types = vec![];
    for (_schema_id, schema) in schemas.schemas() {
        for content in &schema.content {
            let data_type = fetch_type(content);
            if data_type.is_some() {
                types.push(data_type.unwrap());
            }
        }
    }

    types
}
