use fake::{Fake, Faker};
use rand::{Rng, SeedableRng};
use rand::seq::IndexedRandom;
use rand_regex::Regex;
use rand_xorshift::XorShiftRng;
use xsd_parser::models::schema::QName;
use xsd_parser::models::schema::xs::{
    Facet, FacetType, Restriction, RestrictionContent, SimpleBaseTypeContent,
};

fn make_fake<Output: fake::Dummy<Faker> + ToString>() -> Option<String> {
    Option::from(Faker.fake::<Output>().to_string())
}

pub(crate) fn generate_type(type_name: &String) -> Option<String> {
    match type_name.as_str() {
        "boolean" => make_fake::<bool>(),
        "decimal" => make_fake::<f32>(),
        "double" => make_fake::<f64>(),
        "integer" => make_fake::<i32>(),
        "int" => make_fake::<i32>(),
        "positiveInteger" => make_fake::<u32>(),
        "string" => make_fake::<String>(),
        "NMTOKEN" => make_fake::<String>(),
        _ => None,
    }
}

fn generate_enumeration(enumerations: &Vec<String>) -> Option<String> {
    let mut rng = rand::rng();

    enumerations.choose(&mut rng).cloned()
}

fn generate_regex(pattern: &String) -> Option<String> {
    let mut rng = XorShiftRng::from_seed([0; 16]);

    // creates a generator for sampling strings
    let regex_result = Regex::compile(pattern, 1);
    let generator = match regex_result {
        Ok(regex) => regex,
        Err(error) => {
            unimplemented!("Regex pattern: {}\nError: {}", pattern, error);
        }
    };

    let samples = (&mut rng)
        .sample_iter(&generator)
        .take(1)
        .collect::<Vec<String>>();

    if samples.is_empty() {
        return None;
    }

    samples.last().cloned()
}

fn handle_enumeration(type_info: &mut TypeInfo, enumeration: &FacetType) {
    let value = &enumeration.value;

    type_info.enumerations.push(value.clone());
}


fn handle_pattern(type_info: &mut TypeInfo, pattern: &FacetType) {
    type_info.pattern = Some(pattern.value.clone());
}

fn handle_facet(type_info: &mut TypeInfo, facet: &Facet) {
    match facet {
        Facet::MinExclusive(_) => unimplemented!("MinExclusive facet"),
        Facet::MinInclusive(_) => unimplemented!("MinInclusive facet"),
        Facet::MaxExclusive(_) => unimplemented!("MaxExclusive facet"),
        Facet::MaxInclusive(_) => unimplemented!("MaxInclusive facet"),
        Facet::TotalDigits(_) => unimplemented!("TotalDigits facet"),
        Facet::FractionDigits(_) => unimplemented!("FractionDigits facet"),
        Facet::Length(_) => unimplemented!("Length facet"),
        Facet::MinLength(_) => unimplemented!("MinLength facet"),
        Facet::MaxLength(_) => unimplemented!("MaxLength facet"),
        Facet::Enumeration(facet_type) => handle_enumeration(type_info, facet_type),
        Facet::WhiteSpace(_) => unimplemented!("WhiteSpace facet"),
        Facet::Pattern(facet_type) => handle_pattern(type_info, facet_type),
        Facet::Assertion(_) => unimplemented!("Assertion facet"),
        Facet::ExplicitTimezone(_) => unimplemented!("ExplicitTimezone facet"),
    }
}

fn handle_content(type_info: &mut TypeInfo, content: &RestrictionContent) {
    match content {
        RestrictionContent::Annotation(_) => unimplemented!("Annotation"),
        RestrictionContent::SimpleType(_) => unimplemented!("SimpleType"),
        RestrictionContent::Facet(facet) => handle_facet(type_info, facet),
    }
}

pub(crate) fn get_qname(qname: &QName) -> String {
    String::from_utf8(qname.local_name().to_vec()).unwrap()
}

fn get_restriction(type_info: &mut TypeInfo, restriction: &Restriction) {
    if let Some(base) = &restriction.base {
        type_info.name = get_qname(base);
    }

    for content in &restriction.content {
        handle_content(type_info, content);
    }
}

fn parse_restriction(type_info: &mut TypeInfo, content: &SimpleBaseTypeContent) {
    match content {
        SimpleBaseTypeContent::Annotation(_) => unimplemented!("Annotation"),
        SimpleBaseTypeContent::Restriction(x) => get_restriction(type_info, x),
        SimpleBaseTypeContent::List(_) => unimplemented!("List"),
        SimpleBaseTypeContent::Union(_) => unimplemented!("Union"),
    }
}

pub(crate) fn generate_type_info(content: &Vec<SimpleBaseTypeContent>) -> TypeInfo {
    let mut type_info = TypeInfo::new();
    for item in content {
        parse_restriction(&mut type_info, item);
    }

    type_info
}

pub(crate) struct TypeInfo {
    pub(crate) name: String,
    pub(crate) pattern: Option<String>,
    pub(crate) enumerations: Vec<String>,
}

impl TypeInfo {
    pub(crate) fn new() -> Self {
        TypeInfo {
            name: String::new(),
            pattern: None,
            enumerations: Vec::new(),
        }
    }

    pub(crate) fn generate(&self) -> Option<String> {
        if !self.enumerations.is_empty() {
            if self.pattern.is_some() {
                panic!("Type info includes enumeration and pattern data");
            }

            return generate_enumeration(&self.enumerations);
        }

        if let Some(pattern) = &self.pattern {
            return generate_regex(pattern);
        }

        match generate_type(&self.name) {
            Some(name) => Some(name),
            None => None,
        }
    }
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        if self.pattern != other.pattern {
            return false;
        }

        true
    }
}
