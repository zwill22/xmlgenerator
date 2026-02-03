use crate::XMLGeneratorError;
use crate::error::unimplemented;
use chrono::Duration;
use fake::faker;
use fake::{Fake, Faker};
use num_traits::Signed;
use rand::seq::IndexedRandom;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use regex::Regex;
use regextranslator::RegexTranslator;
use time::{Date, Time, format_description};
use xsd_parser::models::schema::QName;
use xsd_parser::models::schema::xs::{
    Facet, FacetType, Restriction, RestrictionContent, SimpleBaseTypeContent,
};

fn make_fake<Output: fake::Dummy<Faker> + ToString>() -> Option<String> {
    Some(Faker.fake::<Output>().to_string())
}

fn generate_regex(pattern: &Regex) -> Option<String> {
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

    samples.first().map(|s| s.to_string())
}

fn make_fake_string(pattern: &str) -> Option<String> {
    let regex = Regex::new(pattern).unwrap();
    generate_regex(&regex)
}

fn make_fake_signed<Output: fake::Dummy<Faker> + ToString + Signed>(
    positive: bool,
) -> Option<String> {
    let val: Output = Faker.fake::<Output>();

    let abs_val = val.abs();

    if positive {
        return Some(abs_val.to_string());
    }

    let negative_val = abs_val.neg();

    Some(negative_val.to_string())
}

fn fake_date() -> Date {
    faker::time::en::Date().fake::<Date>()
}

fn fake_date_format(format: &str) -> Option<String> {
    let date = fake_date();
    let format = format_description::parse(format).unwrap();

    date.format(&format).ok()
}

fn generate_list(date_type: &str) -> Option<String> {
    let n = rand::rng().random_range(1..=10);
    let mut out = "".to_string();
    let item_type = match date_type {
        "NMTOKENS" => "NMTOKEN",
        "IDREFS" => "IDREF",
        _ => return None,
    };

    for i in 0..n {
        if i != 0 {
            out.push(' ');
        }
        let item = generate_type(item_type)?;

        out.push_str(&item);
    }

    Some(out)
}

pub(crate) fn generate_type(type_name: &str) -> Option<String> {
    match type_name {
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
        "Name" => make_fake_string(r"[:A-Z_a-z][-.0-9:A-Z_a-z]*"), // A string that contains a valid XML name r"\i\c*"
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
        "gMonthDay" => fake_date_format("--[month]-[day]"), // Defines the month and day (MM-DD)
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

fn generate_enumeration(enumerations: &[String]) -> Option<String> {
    let mut rng = rand::rng();

    enumerations.choose(&mut rng).cloned()
}

fn sample_output(
    output: &str,
    pattern: &Regex,
    name: &String,
    depth: u16,
) -> Option<String> {
    let limit = 100; // TODO make an option
    if depth >= limit {
        return generate_regex(pattern);
    }

    match pattern.find(output) {
        None => {
            let next_output = generate_type(name).expect("No type generated");
            sample_output(&next_output, pattern, name, depth + 1)
        }
        Some(mat) => Some(mat.as_str().to_string()),
    }
}

fn get_ignore_types() -> Vec<String> {
    let strs = vec![
        // numeric types
        "byte",
        "decimal",
        "int",
        "integer",
        "long",
        "negativeInteger",
        "nonNegativeInteger",
        "nonPositiveInteger",
        "positiveInteger",
        "short",
        "unsignedLong",
        "unsignedInt",
        "unsignedShort",
        "unsignedByte",
        "ENTITIES",
        "ENTITY",
        "ID",
        "IDREF",
        "language",
        "Name",
        "NCName",
        "NMTOKEN",
        "normalizedString",
        "QName",
        "string",
        "token",
        "NOTATION",
        "anyType",
        "anySimpleType",
    ];

    let mut ignore_types = Vec::new();
    for str in strs {
        ignore_types.push(str.to_string());
    }

    ignore_types
}

fn generate_pattern(pattern: &Regex, name: &String) -> Option<String> {
    let ignore_types = get_ignore_types();

    match generate_type(name) {
        Some(output) => {
            if ignore_types.contains(name) {
                generate_regex(pattern)
            } else {
                sample_output(&output, pattern, name, 0)
            }
        }
        None => generate_regex(pattern),
    }
}

fn handle_enumeration(
    type_info: &mut TypeInfo,
    enumeration: &FacetType,
) -> Result<(), XMLGeneratorError> {
    let value = &enumeration.value;

    type_info.enumerations.push(value.clone());

    Ok(())
}

fn check_carriage_returns(pattern: &str) -> Result<(), XMLGeneratorError> {
    let stripped = pattern.replace("\\r\\n", "");

    if stripped.contains("\\r") {
        return Err(XMLGeneratorError::UnimplementedFeature(
            "Carriage returns".to_string(),
        ));
    }

    Ok(())
}

fn handle_regex_pattern(
    type_info: &mut TypeInfo,
    pattern: &str,
    translator: &RegexTranslator,
) -> Result<(), XMLGeneratorError> {
    check_carriage_returns(pattern)?;

    match regex::Regex::new(pattern) {
        Ok(regex) => {
            // Generalise `\d` pattern to equal `[0-9]`
            if pattern.contains(r"\d") {
                let new_pattern = pattern.replace(r"\d", r"[0-9]");
                return handle_regex_pattern(type_info, &new_pattern, translator);
            }

            type_info.pattern = Some(regex);
            Ok(())
        }
        Err(pattern_err) => {
            let translation = translator.translate(pattern)?;

            if translation.as_str() == pattern {
                let error = format!("Compilation error: {}", pattern_err);
                return Err(XMLGeneratorError::RegexError(error));
            }

            handle_regex_pattern(type_info, &translation, translator)
        }
    }
}

fn handle_pattern(
    type_info: &mut TypeInfo,
    pattern: &FacetType,
    regex_translator: &RegexTranslator,
) -> Result<(), XMLGeneratorError> {
    let regex_str = pattern.value.as_str();

    handle_regex_pattern(type_info, regex_str, regex_translator)
}

fn handle_facet(
    type_info: &mut TypeInfo,
    facet: &Facet,
    regex_translator: &RegexTranslator,
) -> Result<(), XMLGeneratorError> {
    match facet {
        Facet::MinExclusive(_) => unimplemented("MinExclusive facet"),
        Facet::MinInclusive(_) => unimplemented("MinInclusive facet"),
        Facet::MaxExclusive(_) => unimplemented("MaxExclusive facet"),
        Facet::MaxInclusive(_) => unimplemented("MaxInclusive facet"),
        Facet::TotalDigits(_) => unimplemented("TotalDigits facet"),
        Facet::FractionDigits(_) => unimplemented("FractionDigits facet"),
        Facet::Length(_) => unimplemented("Length facet"),
        Facet::MinLength(_) => unimplemented("MinLength facet"),
        Facet::MaxLength(_) => unimplemented("MaxLength facet"),
        Facet::Enumeration(facet_type) => handle_enumeration(type_info, facet_type),
        Facet::WhiteSpace(_) => unimplemented("WhiteSpace facet"),
        Facet::Pattern(facet_type) => handle_pattern(type_info, facet_type, regex_translator),
        Facet::Assertion(_) => unimplemented("Assertion facet"),
        Facet::ExplicitTimezone(_) => unimplemented("ExplicitTimezone facet"),
    }
}

fn handle_content(
    type_info: &mut TypeInfo,
    content: &RestrictionContent,
    regex_translator: &RegexTranslator,
) -> Result<(), XMLGeneratorError> {
    match content {
        RestrictionContent::Annotation(_) => unimplemented("Annotation"),
        RestrictionContent::SimpleType(_) => unimplemented("SimpleType"),
        RestrictionContent::Facet(facet) => handle_facet(type_info, facet, regex_translator),
    }
}

pub(crate) fn get_qname(qname: &QName) -> String {
    String::from_utf8(qname.local_name().to_vec()).unwrap()
}

fn get_restriction(
    type_info: &mut TypeInfo,
    restriction: &Restriction,
    regex_translator: &RegexTranslator,
) -> Result<(), XMLGeneratorError> {
    if let Some(base) = &restriction.base {
        type_info.name = get_qname(base);
    }

    for content in &restriction.content {
        handle_content(type_info, content, regex_translator)?;
    }

    Ok(())
}

fn parse_restriction(
    type_info: &mut TypeInfo,
    content: &SimpleBaseTypeContent,
    regex_translator: &RegexTranslator,
) -> Result<(), XMLGeneratorError> {
    match content {
        SimpleBaseTypeContent::Annotation(_) => unimplemented("Annotation"),
        SimpleBaseTypeContent::Restriction(x) => get_restriction(type_info, x, regex_translator),
        SimpleBaseTypeContent::List(_) => unimplemented("List"),
        SimpleBaseTypeContent::Union(_) => unimplemented("Union"),
    }
}

#[derive(Default)]
pub(crate) struct TypeInfo {
    pub(crate) name: String,
    pub(crate) pattern: Option<regex::Regex>,
    pub(crate) enumerations: Vec<String>,
}

impl TypeInfo {
    pub(crate) fn new(
        content: &Vec<SimpleBaseTypeContent>,
        regex_translator: &RegexTranslator,
    ) -> Result<TypeInfo, XMLGeneratorError> {
        let mut type_info = TypeInfo::default();
        for item in content {
            parse_restriction(&mut type_info, item, regex_translator)?;
        }

        Ok(type_info)
    }

    pub(crate) fn generate(&self) -> Option<String> {
        let name = &self.name;
        if !self.enumerations.is_empty() {
            if self.pattern.is_some() {
                panic!("Type info includes enumeration and pattern data");
            }

            return generate_enumeration(&self.enumerations);
        }

        if let Some(pattern) = &self.pattern {
            return generate_pattern(pattern, name);
        }

        generate_type(name)
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
