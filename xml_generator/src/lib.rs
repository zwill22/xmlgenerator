use fake::{Fake, Faker};
use std::ops::Deref;
use std::string::String;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};
use xsd_parser::models::schema::xs::{
    AttributeType, AttributeUseType, ComplexBaseType, ComplexBaseTypeContent, ElementType,
    ElementTypeContent, Facet, FacetType, GroupType, GroupTypeContent, QNameList, Restriction,
    RestrictionContent, SchemaContent, SimpleBaseType, SimpleBaseTypeContent,
};
use xsd_parser::models::schema::{MaxOccurs, QName};
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

fn find_root(generators: &Vec<ElementGenerator>) -> Result<&ElementGenerator, XMLGeneratorError> {
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

    println!("{:?}", all_fields);
    println!("{:?}", all_types);

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

fn make_fake<Output: fake::Dummy<Faker> + ToString>() -> Option<String> {
    Option::from(Faker.fake::<Output>().to_string())
}

fn generate(type_name: &String) -> Option<String> {
    match type_name.as_str() {
        "boolean" => make_fake::<bool>(),
        "float" => make_fake::<f32>(),
        "double" => make_fake::<f64>(),
        "integer" => make_fake::<i32>(),
        "positiveInteger" => make_fake::<u32>(),
        "string" => make_fake::<String>(),
        _ => None,
    }
}

fn generate_type_output(
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

    Err(XMLGeneratorError::XMLBuilderError(format!(
        "Cannot find data type: {}",
        type_name
    )))
}

struct TypeGenerator {
    name: String,
    type_info: Vec<String>,
    elements: Vec<ElementGenerator>,
    groups: Vec<GroupInfo>,
    attributes: Vec<AttributeInfo>,
}

impl TypeGenerator {
    fn generate(
        &self,
        xml_element: &mut XMLElement,
        data_types: &Vec<TypeGenerator>,
        elements: &Vec<ElementGenerator>,
    ) -> Result<(), XMLGeneratorError> {
        for type_information in &self.type_info {
            print!("{}\t", type_information);
        }

        for element in self.elements.iter() {
            let child = element.generate(data_types, elements)?;

            let result = xml_element.add_child(child);
            if result.is_err() {
                return Err(XMLGeneratorError::XMLBuilderError(
                    "Unable to add child to element".to_string(),
                ));
            }
        }

        for group in self.groups.iter() {
            for element in group.elements.iter() {
                let child = element.generate(data_types, elements)?;

                let result = xml_element.add_child(child);
                if result.is_err() {
                    return Err(XMLGeneratorError::XMLBuilderError(
                        "Unable to add group child to element".to_string(),
                    ));
                }
            }
        }

        for attribute in self.attributes.iter() {
            println!("Attribute: {}", attribute.name);
        }

        Ok(())
    }
}

impl TypeGenerator {
    fn new() -> Self {
        TypeGenerator {
            name: String::new(),
            type_info: vec![],
            elements: vec![],
            groups: vec![],
            attributes: vec![],
        }
    }
}

impl PartialEq for TypeGenerator {
    fn eq(&self, other: &Self) -> bool {
        if !self.name.eq(&other.name) {
            return false;
        }

        if !self.type_info.eq(&other.type_info) {
            return false;
        }

        if !self.elements.eq(&other.elements) {
            return false;
        }

        if !self.groups.deref().into_iter().eq(&other.groups) {
            return false;
        }

        true
    }
}

struct ElementGenerator {
    name: Option<String>,
    contents: Vec<TypeGenerator>,
    type_info: Option<String>,
    reference: Option<String>,
}

fn get_ref(
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

fn generate_reference(
    xml_element: &mut XMLElement,
    reference: &String,
    data_types: &Vec<TypeGenerator>,
    elements: &Vec<ElementGenerator>,
) -> Result<(), XMLGeneratorError> {
    let child = get_ref(reference, data_types, elements)?;

    match xml_element.add_child(child) {
        Ok(_) => Ok(()),
        Err(err) => Err(XMLGeneratorError::XMLBuilderError(err.to_string())),
    }
}

impl ElementGenerator {
    fn new() -> Self {
        ElementGenerator {
            name: None,
            contents: vec![],
            type_info: None,
            reference: None,
        }
    }

    fn get_name(&self) -> Result<&String, XMLGeneratorError> {
        if let Some(name) = &self.name {
            return Ok(name);
        }

        if let Some(reference) = &self.reference {
            return Ok(reference);
        }

        Err(XMLGeneratorError::DataTypesFormatError(
            "Element does not have a name or a reference".to_string(),
        ))
    }

    fn generate(
        &self,
        data_types: &Vec<TypeGenerator>,
        elements: &Vec<ElementGenerator>,
    ) -> Result<XMLElement, XMLGeneratorError> {
        let name = self.get_name()?;
        let mut root_element = XMLElement::new(name);
        if let Some(reference) = &self.reference {
            if self.type_info.is_some() {
                return Err(XMLGeneratorError::DataTypesFormatError(
                    "Element is a reference and a type".to_string(),
                ));
            }
            if !self.contents.is_empty() {
                return Err(XMLGeneratorError::DataTypesFormatError(
                    "Element references another element an contains content".to_string(),
                ));
            }

            generate_reference(&mut root_element, reference, data_types, elements)?;
        }
        if self.type_info.is_some() {
            if !self.contents.is_empty() {
                return Err(XMLGeneratorError::DataTypesFormatError(
                    "Data has a type and contains type elements".to_string(),
                ));
            }

            let type_info = self.type_info.as_ref().unwrap();

            generate_type_output(&mut root_element, type_info, data_types, elements)?;
        } else {
            for content in self.contents.iter() {
                content.generate(&mut root_element, data_types, elements)?;
            }
        }

        Ok(root_element)
    }
}

impl PartialEq for ElementGenerator {
    fn eq(&self, other: &Self) -> bool {
        if !self.name.eq(&other.name) {
            return false;
        }
        if !self.type_info.eq(&other.type_info) {
            return false;
        }

        if self.contents.len() != other.contents.len() {
            return false;
        }

        if !self.contents.eq(&other.contents) {
            return false;
        }

        for i in 0..self.contents.len() {
            if !self.contents[i].eq(&other.contents[i]) {
                return false;
            }
        }

        true
    }
}

fn get_group(group: &GroupType) -> GroupInfo {
    let name = group.name.clone();
    if name.is_some() {
        unimplemented!("Named groups");
    }

    if group.ref_.is_some() {
        unimplemented!("Group references");
    }

    let min_occurs = group.min_occurs;
    let max_occurs = match group.max_occurs {
        MaxOccurs::Unbounded => None,
        MaxOccurs::Bounded(x) => Some(x),
    };

    let mut elements = vec![];
    for content in &group.content {
        let element = get_group_content(content);
        elements.push(element);
    }

    GroupInfo { elements }
}

fn get_group_content(content: &GroupTypeContent) -> ElementGenerator {
    match content {
        GroupTypeContent::Annotation(_) => unimplemented!("Annotation"),
        GroupTypeContent::Element(x) => get_element_type(x),
        GroupTypeContent::Group(x) => unimplemented!("Embedded groups"),
        GroupTypeContent::All(x) => unimplemented!("Embedded groups"),
        GroupTypeContent::Choice(x) => unimplemented!("Embedded groups"),
        GroupTypeContent::Sequence(x) => unimplemented!("Embedded groups"),
        GroupTypeContent::Any(_) => unimplemented!("Any"),
    }
}

struct Name {
    name: String,
    namespace: Option<String>,
}

fn get_qname(qname: QName) -> Name {
    let name = String::from_utf8(qname.local_name().to_vec()).unwrap();
    let ns = qname.namespace();
    if ns.is_some() {
        return Name {
            name,
            namespace: Option::from(ns.unwrap().to_string()),
        };
    }

    Name {
        name,
        namespace: None,
    }
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

struct AttributeInfo {
    name: String,
}

fn get_attribute(attribute: &AttributeType) -> AttributeInfo {
    let name = attribute.name.clone().unwrap_or("".to_string());

    if attribute.ref_.is_some() {
        let attribute_ref = attribute.ref_.clone().unwrap();
        get_qname(attribute_ref);
    }

    if attribute.type_.is_some() {
        let attribute_type = attribute.type_.clone().unwrap();
        get_qname(attribute_type);
    }

    match attribute.use_ {
        AttributeUseType::Prohibited => println!("AttributeUseType:\tProhibited"),
        AttributeUseType::Optional => println!("AttributeUseType:\tOptional"),
        AttributeUseType::Required => println!("AttributeUseType:\tRequired"),
    }

    let default = attribute.default.clone().unwrap_or("".to_string());

    let fixed = attribute.fixed.clone().unwrap_or("".to_string());

    if attribute.form.is_some() {
        println!("Form");
    }

    let namespace = attribute.target_namespace.clone().unwrap_or("".to_string());

    if attribute.inheritable.is_some() {
        println!("Inheritable");
    }

    if attribute.annotation.is_some() {
        unimplemented!("Annotation");
    }

    if attribute.simple_type.is_some() {
        get_simple_type(&attribute.clone().simple_type.unwrap());
    }

    AttributeInfo { name }
}

struct GroupInfo {
    elements: Vec<ElementGenerator>,
}

impl PartialEq for GroupInfo {
    fn eq(&self, other: &Self) -> bool {
        if !self.elements.eq(&other.elements) {
            return false;
        }

        true
    }
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
        ComplexBaseTypeContent::Attribute(x) => None,
        ComplexBaseTypeContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        ComplexBaseTypeContent::AnyAttribute(_) => unimplemented!("AnyAttribute"),
        ComplexBaseTypeContent::Assert(_) => unimplemented!("Assert"),
    }
}

fn get_complex_attributes(content: &ComplexBaseTypeContent) -> Option<AttributeInfo> {
    match content {
        ComplexBaseTypeContent::Annotation(_) => unimplemented!("Annotation"),
        ComplexBaseTypeContent::SimpleContent(_) => unimplemented!("SimpleContent"),
        ComplexBaseTypeContent::ComplexContent(_) => unimplemented!("ComplexContent"),
        ComplexBaseTypeContent::OpenContent(_) => unimplemented!("OpenContent"),
        ComplexBaseTypeContent::Group(x) => None,
        ComplexBaseTypeContent::All(x) => None,
        ComplexBaseTypeContent::Choice(x) => None,
        ComplexBaseTypeContent::Sequence(x) => None,
        ComplexBaseTypeContent::Attribute(x) => Option::from(get_attribute(x)),
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

    let abstr = complex.abstract_;
    if abstr {
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
        RestrictionContent::SimpleType(x) => unimplemented!("SimpleType"),
        RestrictionContent::Facet(x) => get_facet(x),
    }
}

struct RestrictionInfo {
    name: String,
    namespace: Option<String>,
    facets: Vec<String>,
}

impl RestrictionInfo {
    fn new() -> RestrictionInfo {
        RestrictionInfo {
            name: String::new(),
            namespace: None,
            facets: Vec::new(),
        }
    }
}

fn get_restriction(restriction: &Restriction) -> RestrictionInfo {
    let mut info = RestrictionInfo::new();
    if restriction.base.is_some() {
        let name = get_qname(restriction.base.clone().unwrap());
        info.name = name.name;
        info.namespace = name.namespace;
    }

    for content in &restriction.content {
        let facet = get_restriction_content(content);
        info.facets.push(facet);
    }

    info
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

fn get_qname_list(qname_list: QNameList) -> Vec<Name> {
    let mut qnames = vec![];
    for qname in qname_list.0 {
        let name = get_qname(qname);
        qnames.push(name);
    }

    qnames
}

fn get_element_type(element: &ElementType) -> ElementGenerator {
    let mut generator = ElementGenerator::new();

    generator.name = element.name.clone();

    if element.ref_.is_some() {
        if generator.name.is_some() {
            panic!("Name already defined.");
        }
        let element_ref = element.ref_.clone().unwrap();
        let reference = Option::from(get_qname(element_ref));
        if let Some(name) = reference {
            generator.reference = Some(name.name);
        }
    }

    if element.type_.is_some() {
        let element_type = element.type_.clone().unwrap();
        let type_info = get_qname(element_type);
        generator.type_info = Some(type_info.name);
    }

    if element.substitution_group.is_some() {
        unimplemented!("Element Substitution Groups");
    }

    let min_occurs = element.min_occurs;

    let max_occurs = match element.max_occurs {
        MaxOccurs::Unbounded => None,
        MaxOccurs::Bounded(x) => Option::from(x),
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

fn generate_schema(string: &String) -> Result<Schemas, XMLGeneratorError> {
    let schemas = Parser::new()
        .with_resolver(FileResolver::new())
        .with_default_namespaces()
        .add_schema_from_str(string);

    if let Err(err) = schemas {
        return Err(XMLGeneratorError::XSDParserError(err.to_string()));
    }

    Ok(schemas.unwrap().finish())
}

fn fetch_type(content: &SchemaContent) -> Option<TypeGenerator> {
    match content {
        SchemaContent::Include(_) => unimplemented!("Include"),
        SchemaContent::Import(_) => unimplemented!("Import"),
        SchemaContent::Redefine(_) => unimplemented!("Redefine"),
        SchemaContent::Override(_) => unimplemented!("Override"),
        SchemaContent::Annotation(_) => unimplemented!("Annotation"),
        SchemaContent::DefaultOpenContent(_) => unimplemented!("DefaultOpenContent"),
        SchemaContent::SimpleType(x) => Option::from(get_simple_type(x)),
        SchemaContent::ComplexType(x) => Option::from(get_complex_type(x)),
        SchemaContent::Group(_) => unimplemented!("Top-level group not supported"),
        SchemaContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        SchemaContent::Element(_) => None,
        SchemaContent::Attribute(_) => unimplemented!("Attribute"),
        SchemaContent::Notation(_) => unimplemented!("Notation"),
    }
}

fn fetch_types(schemas: &Schemas) -> Vec<TypeGenerator> {
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

fn fetch_element(content: &SchemaContent) -> Option<ElementGenerator> {
    match content {
        SchemaContent::Include(_) => unimplemented!("Include"),
        SchemaContent::Import(_) => unimplemented!("Import"),
        SchemaContent::Redefine(_) => unimplemented!("Redefine"),
        SchemaContent::Override(_) => unimplemented!("Override"),
        SchemaContent::Annotation(_) => unimplemented!("Annotation"),
        SchemaContent::DefaultOpenContent(_) => unimplemented!("DefaultOpenContent"),
        SchemaContent::SimpleType(_) => None,
        SchemaContent::ComplexType(_) => None,
        SchemaContent::Group(_) => unimplemented!("Top-level group not supported"),
        SchemaContent::AttributeGroup(_) => unimplemented!("AttributeGroup"),
        SchemaContent::Element(x) => Option::from(get_element_type(x)),
        SchemaContent::Attribute(_) => unimplemented!("Attribute"),
        SchemaContent::Notation(_) => unimplemented!("Notation"),
    }
}

fn generate_elements(schemas: &Schemas) -> Vec<ElementGenerator> {
    let mut elements = vec![];
    for (_schema_id, schema) in schemas.schemas() {
        for content in &schema.content {
            let element = fetch_element(content);
            if element.is_some() {
                elements.push(element.unwrap());
            }
        }
    }

    elements
}

fn generate_output(
    generator: &ElementGenerator,
    data_types: &Vec<TypeGenerator>,
    elements: &Vec<ElementGenerator>,
) -> Result<String, XMLGeneratorError> {
    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".into())
        .build();

    let root_element = generator.generate(data_types, elements)?;

    let mut writer: Vec<u8> = Vec::new();
    xml.set_root_element(root_element);
    let result = xml.generate(&mut writer);
    if result.is_err() {
        return Err(XMLGeneratorError::XMLBuilderError(
            result.err().unwrap().to_string(),
        ));
    }

    let output = String::from_utf8(writer).expect("Unable to convert XML output to string");

    Ok(output)
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
    println!("{}", xsd_string);
    println!("=================================================================================\n");
    let schemas = generate_schema(xsd_string)?;

    let data_types = fetch_types(&schemas);

    let elements = generate_elements(&schemas);

    if elements.len() == 0 {
        return Err(XMLGeneratorError::DataTypesFormatError(
            "No elements found".to_string(),
        ));
    }

    if elements.len() > 1 {
        println!("Multiple elements found");
    }

    let root_element = find_root(&elements)?;

    generate_output(root_element, &data_types, &elements)
}
