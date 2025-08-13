use crate::XMLGeneratorError::XSDParserError;
use std::string::String;

use xsd_parser::models::schema::QName;
use xsd_parser::models::schema::xs::{
    AttributeType, AttributeUseType, ComplexBaseType, ComplexBaseTypeContent, ElementType,
    ElementTypeContent, Facet, FacetType, GroupType, GroupTypeContent, QNameList, Restriction,
    RestrictionContent, SchemaContent, SimpleBaseType, SimpleBaseTypeContent,
};
use xsd_parser::pipeline::parser::resolver::FileResolver;
use xsd_parser::{Parser, Schemas};

/// XML generator error
///
/// Struct which manages errors in the XMLGenerator crate
#[derive(Debug)]
pub enum XMLGeneratorError {
    /// Error parsing the input XSD file contents
    XSDParserError(String),
    /// Datatypes are in an invalid format
    DataTypesFormatError(String),
    /// Error generating the output XML structure
    XMLBuilderError(String),
}

fn get_group_content(content: &GroupTypeContent, depth: usize) {
    match content {
        GroupTypeContent::Annotation(_) => unimplemented!("Annotation"),
        GroupTypeContent::Element(x) => get_element_type(x, depth),
        GroupTypeContent::Group(x) => get_group(x, depth),
        GroupTypeContent::All(x) => get_group(x, depth),
        GroupTypeContent::Choice(x) => get_group(x, depth),
        GroupTypeContent::Sequence(x) => get_group(x, depth),
        GroupTypeContent::Any(_) => unimplemented!("Any"),
    }
}

fn print_title(title: &str, depth: usize) {
    let indent = std::iter::repeat('\t').take(depth).collect::<String>();
    println!("{}{}", indent, title);
    let l = title.len();
    let banner = std::iter::repeat('-').take(l).collect::<String>();
    println!("{}{}", indent, banner);
}

fn print_val<T: ToString + std::fmt::Display>(name: &str, val: &T, depth: usize) {
    if val.to_string().is_empty() {
        return;
    }

    let n = 32 - name.len();
    let indent = std::iter::repeat('\t').take(depth).collect::<String>();
    let space = std::iter::repeat(" ").take(n).collect::<String>();

    println!("{}{}:{}{}", indent, name, space, val);
}

fn get_qname(title: &str, qname: QName, depth: usize) {
    print_title(title, depth);

    let name = std::string::String::from_utf8(qname.local_name().to_vec()).unwrap();
    print_val("Name", &name, depth);

    let ns = qname.namespace();
    if ns.is_some() {
        print_val("Namespace", &ns.unwrap(), depth);
    }
}

fn get_group(group: &GroupType, depth: usize) {
    print_title("Group", depth);

    let id = group.id.clone().unwrap_or("".to_string());
    print_val("ID", &id, depth);

    let name = group.name.clone().unwrap_or("".to_string());
    print_val("Name", &name, depth);

    if group.ref_.is_some() {
        let group_ref = group.ref_.clone().unwrap();
        get_qname("Reference", group_ref, depth + 1);
    }
    let min_occurs = group.min_occurs;
    print_val("Min Occurs", &min_occurs, depth);
    let max_occurs = group.max_occurs.is_bounded();
    print_val("Max Occurs", &max_occurs, depth);

    for content in &group.content {
        get_group_content(content, depth + 1);
    }
}

fn get_attribute(attribute: &AttributeType, depth: usize) {
    print_title("Attribute", depth);
    let id = attribute.id.clone().unwrap_or("".to_string());
    print_val("ID", &id, depth);

    let name = attribute.name.clone().unwrap_or("".to_string());
    print_val("Name", &name, depth);

    if attribute.ref_.is_some() {
        let attribute_ref = attribute.ref_.clone().unwrap();
        get_qname("Reference", attribute_ref, depth + 1);
    }

    if attribute.type_.is_some() {
        let attribute_type = attribute.type_.clone().unwrap();
        get_qname("Type", attribute_type, depth + 1);
    }

    match attribute.use_ {
        AttributeUseType::Prohibited => print_val("AttributeUseType", &"Prohibited", depth),
        AttributeUseType::Optional => print_val("AttributeUseType", &"Optional", depth),
        AttributeUseType::Required => print_val("AttributeUseType", &"Required", depth),
    }

    let default = attribute.default.clone().unwrap_or("".to_string());
    print_val("Default", &default, depth);

    let fixed = attribute.fixed.clone().unwrap_or("".to_string());
    print_val("Fixed", &fixed, depth);

    if attribute.form.is_some() {
        print_val("Form", &"FormChoiceType", depth);
    }

    let namespace = attribute.target_namespace.clone().unwrap_or("".to_string());
    print_val("TargetNamespace", &namespace, depth);

    if attribute.inheritable.is_some() {
        print_val("Inheritable", &attribute.inheritable.unwrap(), depth);
    }

    if attribute.annotation.is_some() {
        unimplemented!("Annotation");
    }

    if attribute.simple_type.is_some() {
        get_simple_type(&attribute.clone().simple_type.unwrap(), depth + 1);
    }
}

fn get_complex_content(content: &ComplexBaseTypeContent, depth: usize) {
    match content {
        ComplexBaseTypeContent::Annotation(_) => unimplemented!("Annotation"),
        ComplexBaseTypeContent::SimpleContent(_) => unimplemented!("SimpleContent"),
        ComplexBaseTypeContent::ComplexContent(_) => unimplemented!("ComplexContent"),
        ComplexBaseTypeContent::OpenContent(_) => unimplemented!("OpenContent"),
        ComplexBaseTypeContent::Group(x) => get_group(x, depth),
        ComplexBaseTypeContent::All(x) => get_group(x, depth),
        ComplexBaseTypeContent::Choice(x) => get_group(x, depth),
        ComplexBaseTypeContent::Sequence(x) => get_group(x, depth),
        ComplexBaseTypeContent::Attribute(x) => get_attribute(x, depth),
        ComplexBaseTypeContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        ComplexBaseTypeContent::AnyAttribute(_) => unimplemented!("AnyAttribute"),
        ComplexBaseTypeContent::Assert(_) => unimplemented!("Assert"),
    }
}

fn get_complex_type(complex: &ComplexBaseType, depth: usize) -> String {
    print_title("Complex Type", depth);
    let id = complex.id.clone().unwrap_or("".to_string());
    print_val("ID", &id, depth);

    let name = complex.name.clone().unwrap_or("".to_string());
    print_val("Name", &name, depth);

    if complex.mixed.is_some() {
        let mixed = complex.mixed.clone().unwrap();
        print_val("Mixed", &mixed, depth);
    }

    let abstr = complex.abstract_;
    print_val("Abstract", &abstr, depth);

    if complex.final_.is_some() {
        print_val("Final", &"DerivationSetType", depth);
    }

    if complex.block.is_some() {
        print_val("Block", &"BlockSetType", depth);
    }

    let default_attributes_apply = complex.default_attributes_apply;
    print_val("DefaultAttributesApply", &default_attributes_apply, depth);

    for content in &complex.content {
        get_complex_content(content, depth + 1);
    }

    name
}

fn get_facet_type(facet_type: &FacetType, depth: usize) {
    print_title("Facet", depth);
    let id = facet_type.id.clone().unwrap_or("".to_string());
    print_val("\tID", &id, depth);

    let value = facet_type.value.clone();
    print_val("\tValue", &value, depth);

    let fixed = facet_type.fixed.clone();
    print_val("\tFixed", &fixed, depth);

    if facet_type.annotation.is_some() {
        unimplemented!("Annotation");
    }
}

fn get_facet(facet: &Facet, depth: usize) {
    match facet {
        Facet::MinExclusive(x) => get_facet_type(x, depth),
        Facet::MinInclusive(x) => get_facet_type(x, depth),
        Facet::MaxExclusive(x) => get_facet_type(x, depth),
        Facet::MaxInclusive(x) => get_facet_type(x, depth),
        Facet::TotalDigits(x) => get_facet_type(x, depth),
        Facet::FractionDigits(x) => get_facet_type(x, depth),
        Facet::Length(x) => get_facet_type(x, depth),
        Facet::MinLength(x) => get_facet_type(x, depth),
        Facet::MaxLength(x) => get_facet_type(x, depth),
        Facet::Enumeration(x) => get_facet_type(x, depth),
        Facet::WhiteSpace(x) => get_facet_type(x, depth),
        Facet::Pattern(x) => get_facet_type(x, depth),
        Facet::Assertion(_) => unimplemented!("Assertion"),
        Facet::ExplicitTimezone(x) => get_facet_type(x, depth),
    }
}

fn get_restriction_content(content: &RestrictionContent, depth: usize) {
    match content {
        RestrictionContent::Annotation(_) => unimplemented!("Annotation"),
        RestrictionContent::SimpleType(x) => {get_simple_type(x, depth);},
        RestrictionContent::Facet(x) => get_facet(x, depth),
    }
}

fn get_restriction(restriction: &Restriction, depth: usize) {
    print_title("Restriction", depth);
    let id = restriction.id.clone().unwrap_or("".to_string());
    print_val("\tID", &id, depth);

    if restriction.base.is_some() {
        let base = restriction.base.clone().unwrap();
        get_qname("Base", base, depth + 1);
    }

    for content in &restriction.content {
        get_restriction_content(content, depth + 1);
    }
}

fn get_simple_content(content: &SimpleBaseTypeContent, depth: usize) {
    match content {
        SimpleBaseTypeContent::Annotation(_) => unimplemented!("Annotation"),
        SimpleBaseTypeContent::Restriction(x) => get_restriction(x, depth),
        SimpleBaseTypeContent::List(_) => unimplemented!("List"),
        SimpleBaseTypeContent::Union(_) => unimplemented!("Union"),
    }
}

fn get_simple_type(simple: &SimpleBaseType, depth: usize) -> String {
    print_title("Simple Type", depth);
    let id = simple.id.clone().unwrap_or("".to_string());
    print_val("\tID", &id, depth);

    if simple.final_.is_some() {
        print_val("\tFinal", &"SimpleDerivationSetType", depth);
    }

    let name = simple.name.clone().unwrap_or("".to_string());
    print_val("\tName", &name, depth);

    for content in &simple.content {
        get_simple_content(content, depth + 1);
    }

    name
}

fn get_element_content(content: &ElementTypeContent, depth: usize) -> String {
    match content {
        ElementTypeContent::Annotation(_) => unimplemented!("Annotation"),
        ElementTypeContent::SimpleType(x) => get_simple_type(x, depth),
        ElementTypeContent::ComplexType(x) => get_complex_type(x, depth),
        ElementTypeContent::Alternative(_) => unimplemented!("Alternative"),
        ElementTypeContent::Unique(_) => unimplemented!("Unique"),
        ElementTypeContent::Key(_) => unimplemented!("Key"),
        ElementTypeContent::Keyref(_) => unimplemented!("Keyref"),
    }
}

fn get_qname_list(qname_list: QNameList, depth: usize) {
    let mut i = 0;
    for qname in qname_list.0 {
        i = i + 1;
        get_qname("Substitution group {}", qname, depth);
    }
}

fn get_element_type(element: &ElementType, depth: usize) {
    print_title("Element", depth);

    let id = element.id.clone().unwrap_or("".to_string());
    print_val("ID", &id, depth);

    let name = element.name.clone().unwrap_or("".to_string());
    print_val("Name", &name, depth);

    if element.ref_.is_some() {
        let element_ref = element.ref_.clone().unwrap();
        get_qname("Reference", element_ref, depth + 1);
    }

    if element.type_.is_some() {
        let element_type = element.type_.clone().unwrap();
        get_qname("Type", element_type, depth + 1);
    }

    if element.substitution_group.is_some() {
        let element_substitution_group = element.substitution_group.clone().unwrap();
        get_qname_list(element_substitution_group, depth + 1);
    }

    let min_occurs = element.min_occurs;
    print_val("Min Occurs", &min_occurs, depth);

    let max_occurs = element.max_occurs.is_bounded();
    print_val("Max Occurs", &max_occurs, depth);

    let default = element.default.clone().unwrap_or("".to_string());
    print_val("Default", &default, depth);

    let fixed = element.fixed.clone().unwrap_or("".to_string());
    print_val("Fixed", &fixed, depth);

    let nillable = element.nillable.unwrap_or(false);
    print_val("Nillable", &nillable, depth);

    let abstr = element.abstract_;
    print_val("Abstract", &abstr, depth);

    if element.final_.is_some() {
        print_val("Final", &"DerivationSetType", depth);
    }

    if element.block.is_some() {
        print_val("Block", &"BlockSetType", depth);
    }

    if element.form.is_some() {
        print_val("Form", &"FormType", depth);
    }

    let namespace = element.target_namespace.clone().unwrap_or("".to_string());
    print_val("TargetNamespace", &namespace, depth);

    for content in &element.content {
        get_element_content(content, depth + 1);
    }
}

fn fetch_content(content: &SchemaContent) {
    let depth: usize = 0;
    match content {
        SchemaContent::Include(_) => unimplemented!("Include"),
        SchemaContent::Import(_) => unimplemented!("Import"),
        SchemaContent::Redefine(_) => unimplemented!("Redefine"),
        SchemaContent::Override(_) => unimplemented!("Override"),
        SchemaContent::Annotation(_) => unimplemented!("Annotation"),
        SchemaContent::DefaultOpenContent(_) => unimplemented!("DefaultOpenContent"),
        SchemaContent::SimpleType(x) => {get_simple_type(x, depth);},
        SchemaContent::ComplexType(x) => {get_complex_type(x, depth);},
        SchemaContent::Group(x) => get_group(x, depth),
        SchemaContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        SchemaContent::Element(element_type) => get_element_type(element_type, depth),
        SchemaContent::Attribute(_) => unimplemented!("Attribute"),
        SchemaContent::Notation(_) => unimplemented!("Notation"),
    }
}

fn generate_schema(string: &String) -> Result<Schemas, XMLGeneratorError> {
    let schemas = Parser::new()
        .with_resolver(FileResolver::new())
        .with_default_namespaces()
        .add_schema_from_str(string);

    if let Err(err) = schemas {
        return Err(XSDParserError(err.to_string()));
    }

    Ok(schemas.unwrap().finish())
}

fn fetch_type(content: &SchemaContent) -> Option<String> {
    let depth: usize = 0;
    match content {
        SchemaContent::Include(_) => unimplemented!("Include"),
        SchemaContent::Import(_) => unimplemented!("Import"),
        SchemaContent::Redefine(_) => unimplemented!("Redefine"),
        SchemaContent::Override(_) => unimplemented!("Override"),
        SchemaContent::Annotation(_) => unimplemented!("Annotation"),
        SchemaContent::DefaultOpenContent(_) => unimplemented!("DefaultOpenContent"),
        SchemaContent::SimpleType(x) => Option::from(get_simple_type(x, depth)),
        SchemaContent::ComplexType(x) => Option::from(get_complex_type(x, depth)),
        SchemaContent::Group(_) => unimplemented!("Top-level group not supported"),
        SchemaContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        SchemaContent::Element(_) => None,
        SchemaContent::Attribute(_) => unimplemented!("Attribute"),
        SchemaContent::Notation(_) => unimplemented!("Notation"),
    }
}

fn fetch_types(schemas: &Schemas) -> Result<Vec<String>, XMLGeneratorError> {
    let mut types = vec![];
    for (_schema_id, schema) in schemas.schemas() {
        for content in &schema.content {
            let data_type = fetch_type(content);
            if data_type.is_some() {
                types.push(data_type.unwrap());
            }
        }
    }

    Ok(types)
}

/// Generate an XML string containing fake data
///
/// Using an XSD file contents as a string, generate an XML file string of the
/// same format with fake data.
///
/// The function uses the `xsd_parser` crate to parse the input. If this library returns
/// en error, then the function returns an `XMLGeneratorError::XMLParserError`.
/// This crate generates a `data_types` object which the XMLGenerator uses th
/// generate the output xml.
///
/// If the `data_types` contains data which is not in the required format, then an
/// `XMLGeneratorError::DataTypeFormatError` is returned. This includes cases
/// such as multiple root nodes or circular dependencies.
///
/// The function sorts the data into a dependency tree and uses this to generate an
/// `XMLBuilder` object using the `xml_builder` crate. If the `XMLBuilder` returns
/// an error when generating the output xml, then an `XMLGeneratorError::XMLBuilderError`
/// is returned.
pub fn generate_xml(xsd_string: &String) -> Result<String, XMLGeneratorError> {
    println!("\n===============================================================================\n");
    let schemas = generate_schema(xsd_string)?;

    let data_types = fetch_types(&schemas);

    Ok("".to_string())
}
