use crate::XMLGeneratorError;
use chrono::Duration;
use fake::{Fake, Faker, faker};
use num_traits::Signed;
use rand::prelude::{IndexedRandom, ThreadRng};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use regex::Regex;
use time::{Date, Time, format_description};

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

fn make_fake<Output: fake::Dummy<Faker> + ToString>() -> Option<String> {
    Some(Faker.fake::<Output>().to_string())
}

fn make_fake_signed<Output: fake::Dummy<Faker> + ToString + Signed>(
    positive: bool,
    zero_allowed: bool,
) -> Option<String> {
    let val: Output = Faker.fake::<Output>();

    let abs_val = val.abs();

    if !zero_allowed && (abs_val == Output::zero()) {
        return make_fake_signed::<Output>(positive, zero_allowed);
    }

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

pub(crate) struct Generator {
    rng: ThreadRng,
    xor: XorShiftRng,
    ignore: Vec<String>,
    max_depth: u16,
}

impl Generator {
    pub fn new(max_depth: u16) -> Generator {
        let rng = rand::rng();
        let xor = XorShiftRng::from_os_rng();
        let ignore = get_ignore_types();
        Generator {
            rng,
            xor,
            ignore,
            max_depth,
        }
    }

    pub(crate) fn validate(input_str: &str, pattern: &str) -> Result<bool, XMLGeneratorError> {
        let regex = match Regex::new(input_str) {
            Ok(re) => re,
            Err(_) => return Err(XMLGeneratorError::RegexError(input_str.to_string())),
        };

        let result = regex.is_match(pattern);

        Ok(result)
    }

    fn sample(&mut self, regex: rand_regex::Regex) -> Vec<String> {
        (&mut self.xor)
            .sample_iter(&regex)
            .take(1000)
            .collect::<Vec<String>>()
    }

    pub(crate) fn generate_regex(&mut self, pattern: &str) -> Option<String> {
        let regex = match rand_regex::Regex::compile(pattern, 100) {
            Ok(regex) => regex,
            Err(_) => return None,
        };

        let mut samples = self.sample(regex);

        samples.sort();

        samples.first().map(|s| s.to_string())
    }

    fn generate_list(&mut self, date_type: &str) -> Option<String> {
        let n = self.range();

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
            let item = self.generate_type(item_type)?;

            out.push_str(&item);
        }

        Some(out)
    }

    pub(crate) fn generate_type(&mut self, type_name: &str) -> Option<String> {
        match type_name {
            // Numeric Data Types
            "byte" => make_fake::<i8>(),     // A signed 8-bit integer
            "decimal" => make_fake::<f32>(), // A decimal value
            "int" => make_fake::<i32>(),     // A signed 32-bit integer
            "integer" => make_fake::<i32>(), // An integer value
            "long" => make_fake::<i64>(),    // A signed 64-bit integer
            "negativeInteger" => make_fake_signed::<i32>(false, false), // A signed negative integer
            "nonNegativeInteger" => make_fake_signed::<i32>(true, true), // A signed positive integer
            "nonPositiveInteger" => make_fake_signed::<i32>(false, true), // A non-positive integer
            "positiveInteger" => make_fake_signed::<i32>(true, false),   // A positive integer
            "short" => make_fake::<i16>(),                               // A signed 16-bit integer
            "unsignedLong" => make_fake::<u64>(), // An unsigned 64-bit integer
            "unsignedInt" => make_fake::<u32>(),  // An unsigned 32-bit integer
            "unsignedShort" => make_fake::<u16>(), // An unsigned 16-bit integer
            "unsignedByte" => make_fake::<u8>(),  // An unsigned 8-bit integer

            // String data types
            // TODO Add patterns for string types
            "ENTITIES" => make_fake::<String>(), // ENTITIES
            "ENTITY" => make_fake::<String>(),   // ENTITY
            "ID" => make_fake::<String>(),       // A string that represents the ID attribute
            "IDREF" => make_fake::<String>(),    // A string that represents the IDREF attribute
            "language" => make_fake::<String>(), // A string that contains a valid language id
            "Name" => self.generate_regex(r"[:A-Z_a-z][-.0-9:A-Z_a-z]*"), // A string that contains a valid XML name r"\i\c*"
            "NCName" => self.generate_regex(r"[A-Z_a-z][-.0-9A-Z_a-z]*"), // NCName
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
            "NMTOKENS" => self.generate_list(type_name),
            "IDREFS" => self.generate_list(type_name),

            // Just use a string for any type
            "anyType" => make_fake::<String>(),
            "anySimpleType" => make_fake::<String>(),

            _ => None,
        }
    }

    pub(crate) fn range(&mut self) -> u8 {
        self.rng.random_range(1..=10)
    }

    pub(crate) fn generate_enumeration(&mut self, enumerations: &[String]) -> Option<String> {
        enumerations.choose(&mut self.rng).cloned()
    }

    pub(crate) fn sample_output(
        &mut self,
        output: &str,
        pattern: &str,
        name: &str,
        depth: u16,
    ) -> Option<String> {
        if depth >= self.max_depth {
            return self.generate_regex(pattern);
        }

        match pattern.find(output) {
            None => {
                let next_output = self.generate_type(name).expect("No type generated");
                self.sample_output(&next_output, pattern, name, depth + 1)
            }
            Some(mat) => Some(mat.to_string()),
        }
    }

    pub(crate) fn generate_pattern(&mut self, pattern: &str, name: &str) -> Option<String> {
        match self.generate_type(name) {
            Some(output) => {
                if self.ignore.contains(&name.to_string()) {
                    self.generate_regex(pattern)
                } else {
                    self.sample_output(&output, pattern, name, 0)
                }
            }
            None => self.generate_regex(pattern),
        }
    }
}
