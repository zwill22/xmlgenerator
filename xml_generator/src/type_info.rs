use chrono::Duration;
use fake::faker;
use fake::{Fake, Faker};
use num_traits::Signed;
use rand::seq::IndexedRandom;
use rand::{Rng, SeedableRng};
use rand_regex;
use rand_xorshift::XorShiftRng;
use regex;
use time::{Date, Time, format_description};
use xsd_parser::models::schema::QName;
use xsd_parser::models::schema::xs::{
    Facet, FacetType, Restriction, RestrictionContent, SimpleBaseTypeContent,
};

fn make_fake<Output: fake::Dummy<Faker> + ToString>() -> Option<String> {
    Option::from(Faker.fake::<Output>().to_string())
}

fn make_fake_signed<Output: fake::Dummy<Faker> + ToString + Signed>(
    positive: bool,
) -> Option<String> {
    let val: Output = Faker.fake::<Output>();

    let abs_val = val.abs();

    if positive {
        return Option::from(abs_val.to_string());
    }

    let negative_val = abs_val.neg();

    Option::from(negative_val.to_string())
}

fn fake_date() -> Date {
    faker::time::en::Date().fake::<Date>()
}

fn fake_date_format(format: &str) -> Option<String> {
    let date = fake_date();
    let format = format_description::parse(format).unwrap();

    match date.format(&format) {
        Ok(format) => Some(format),
        Err(_) => None,
    }
}

fn generate_list(date_type: &String) -> Option<String> {
    let n = rand::rng().random_range(1..=10);
    let mut out = "".to_string();
    let item_type = match date_type.as_str() {
        "NMTOKENS" => "NMTOKEN",
        "IDREFS" => "IDREF",
        _ => return None,
    };

    for i in 0..n {
        if i != 0 {
            out.push(' ');
        }
        let item = match generate_type(&item_type.to_string()) {
            Some(item) => item,
            None => return None,
        };

        out.push_str(&item);
    }

    Some(out)
}

pub(crate) fn generate_type(type_name: &String) -> Option<String> {
    match type_name.as_str() {
        // Numeric Data Types
        "byte" => make_fake::<i8>(),     // A signed 8-bit integer
        "decimal" => make_fake::<f32>(), // A decimal value
        "int" => make_fake::<i32>(),     // A signed 32-bit integer
        "integer" => make_fake::<i32>(), // An integer value
        "long" => make_fake::<i64>(),    // A signed 64-bit integer
        "negativeInteger" => make_fake_signed::<i32>(false), // A signed negative integer
        "nonNegativeInteger" => make_fake_signed::<i32>(true), // A signed positive integer
        "nonPositiveInteger" => make_fake_signed::<i32>(false), // A non-positive integer
        "positiveInteger" => make_fake_signed::<i32>(true), // A positive integer
        "short" => make_fake::<i16>(),   // A signed 16-bit integer
        "unsignedLong" => make_fake::<u64>(), // An unsigned 64-bit integer
        "unsignedInt" => make_fake::<u32>(), // An unsigned 32-bit integer
        "unsignedShort" => make_fake::<u16>(), // An unsigned 16-bit integer
        "unsignedByte" => make_fake::<u8>(), // An unsigned 8-bit integer

        // String data types
        // TODO Add patterns for string types
        "ENTITIES" => make_fake::<String>(),         // ENTITIES
        "ENTITY" => make_fake::<String>(),           // ENTITY
        "ID" => make_fake::<String>(),               // A string that represents the ID attribute
        "IDREF" => make_fake::<String>(),            // A string that represents the IDREF attribute
        "language" => make_fake::<String>(),         // A string that contains a valid language id
        "Name" => make_fake::<String>(),             // A string that contains a valid XML name
        "NCName" => make_fake::<String>(),           // NCName
        "NMTOKEN" => make_fake::<String>(), // A string that represents the NMTOKEN attribute
        "normalizedString" => make_fake::<String>(), // A string that does not contain line feeds, carriage returns, or tabs
        "QName" => make_fake::<String>(),            // QName
        "string" => make_fake::<String>(),           // A string
        "token" => make_fake::<String>(), // A string that does not contain line feeds, carriage returns, tabs, leading or trailing spaces, or multiple spaces

        // Date time data types
        "date" => make_fake::<Date>(), // Defines a date value
        "dateTime" => Some(faker::time::en::DateTime().fake()), // Defines a date and time value
        "duration" => make_fake::<Duration>(), // Defines a time interval
        "gDay" => fake_date_format("[day]"), // Defines the day (DD)
        "gMonth" => fake_date_format("[month]"), // Defines the month (MM)
        "gMonthDay" => fake_date_format("[month]-[day]"), // Defines the month and day (MM-DD)
        "gYear" => fake_date_format("[year]"), // Defines the year (YYYY)
        "gYearMonth" => fake_date_format("[year]-[month]"), // Defines the year and month (YYYY-MM)
        "time" => make_fake::<Time>(),

        // Miscellaneous data types
        "anyURI" => make_fake::<http::Uri>(),
        "base64Binary" => Some(fake::base64::Base64.fake()),
        "boolean" => make_fake::<bool>(),
        "float" => make_fake::<f32>(),
        "double" => make_fake::<f64>(),
        "hexBinary" => {
            let fake_string = make_fake::<String>().unwrap();
            let fake_hex_from_string = hex::encode(fake_string.as_bytes());

            Some(fake_hex_from_string)
        }
        "NOTATION" => make_fake::<String>(),

        // List types
        "NMTOKENS" => generate_list(type_name),
        "IDREFS" => generate_list(type_name),

        // Just use a string for any type
        "anyType" => make_fake::<String>(),
        "anySimpleType" => make_fake::<String>(),

        _ => None,
    }
}

fn generate_enumeration(enumerations: &Vec<String>) -> Option<String> {
    let mut rng = rand::rng();

    enumerations.choose(&mut rng).cloned()
}

fn generate_regex(pattern: &regex::Regex) -> Option<String> {
    let seed = 42;
    let mut rng = XorShiftRng::seed_from_u64(seed);

    let regex = match rand_regex::Regex::compile(pattern.as_str(), 100) {
        Ok(regex) => regex,
        Err(_) => return None,
    };

    let mut samples = (&mut rng)
        .sample_iter(&regex)
        .take(1000)
        .collect::<Vec<String>>();

    samples.sort();

    match samples.first() {
        None => None,
        Some(s) => Some(s.to_string()),
    }
}

fn handle_enumeration(type_info: &mut TypeInfo, enumeration: &FacetType) {
    let value = &enumeration.value;

    type_info.enumerations.push(value.clone());
}

fn handle_regex_pattern(type_info: &mut TypeInfo, pattern: &str) {
    match regex::Regex::new(pattern) {
        Ok(regex) => {
            type_info.pattern = Some(regex);
        }
        Err(pattern_err) => {
            let escape = regex::escape(pattern);
            if escape.as_str() != pattern {
                handle_regex_pattern(type_info, escape.as_str());
            } else {
                panic!("Unable to compile regex: {}", pattern_err);
            }
        }
    }
}

fn handle_pattern(type_info: &mut TypeInfo, pattern: &FacetType) {
    let regex_str = pattern.value.as_str();

    handle_regex_pattern(type_info, regex_str);
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
    pub(crate) pattern: Option<regex::Regex>,
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

        match &self.pattern {
            None => match other.pattern {
                None => {}
                Some(_) => return false,
            },
            Some(pattern1) => match &other.pattern {
                None => return false,
                Some(pattern2) => {
                    if pattern1.as_str() != pattern2.as_str() {
                        return false;
                    }
                }
            },
        }

        true
    }
}
