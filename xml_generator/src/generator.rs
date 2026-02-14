use crate::XMLGeneratorError;
use crate::xsd_type::XsdType;
use chrono::{DateTime, Duration, Utc};
use fake::{Fake, Faker};
use num_traits::Signed;
use rand::prelude::{IndexedRandom, ThreadRng};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use regex::Regex;
use std::collections::HashSet;

fn make_fake<Output: fake::Dummy<Faker> + ToString>() -> Option<Output> {
    Some(Faker.fake::<Output>())
}

fn fake<Input: fake::Dummy<Faker> + ToString>() -> Option<String> {
    make_fake::<Input>().map(|output| output.to_string())
}

fn fake_signed<Output: fake::Dummy<Faker> + ToString + Signed>(
    positive: bool,
    zero_allowed: bool,
) -> Option<String> {
    let val: Output = Faker.fake::<Output>();

    let abs_val = val.abs();

    if !zero_allowed && (abs_val == Output::zero()) {
        return fake_signed::<Output>(positive, zero_allowed);
    }

    if positive {
        return Some(abs_val.to_string());
    }

    let negative_val = abs_val.neg();

    Some(negative_val.to_string())
}

fn fake_datetime() -> DateTime<Utc> {
    let epoch = make_fake::<i32>().unwrap();

    match DateTime::from_timestamp(epoch as i64, 0) {
        Some(datetime) => datetime,
        None => {
            fake_datetime()
        }
    }
}

fn fake_date_format(format: &str) -> Option<String> {
    let date = fake_datetime();

    Some(date.format(format).to_string())
}

fn fake_base64() -> Option<String> {
    Some(fake::base64::Base64.fake())
}

fn fake_hex() -> Option<String> {
    let fake_string = fake::<String>().unwrap();
    let fake_hex_from_string = hex::encode(fake_string.as_bytes());

    Some(fake_hex_from_string)
}

pub(crate) struct Generator {
    rng: ThreadRng,
    xor: XorShiftRng,
    max_depth: u16,
    namespaces: Vec<String>,
}

impl Generator {
    pub fn new(max_depth: u16) -> Generator {
        let rng = rand::rng();
        let xor = XorShiftRng::from_os_rng();
        Generator {
            rng,
            xor,
            max_depth,
            namespaces: vec![],
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

    fn sample(&mut self, regex: rand_regex::Regex) -> HashSet<String> {
        (&mut self.xor)
            .sample_iter(&regex)
            .take(1000)
            .collect::<HashSet<String>>()
    }

    pub(crate) fn generate_regex(&mut self, pattern: &str) -> Option<String> {
        let regex = match rand_regex::Regex::compile(pattern, 100) {
            Ok(regex) => regex,
            Err(_) => return None,
        };

        let samples = self.sample(regex);

        if samples.len() == 1 {
            return samples.into_iter().next();
        }

        for sample in samples {
            if sample.is_empty() {
                continue;
            }

            return Some(sample);
        }

        None
    }

    fn generate_xsd_type(&mut self, xsd_type: &XsdType) -> Option<String> {
        match xsd_type {
            XsdType::Byte => fake::<i8>(),
            XsdType::Short => fake::<i16>(),
            XsdType::Int => fake::<i32>(),
            XsdType::Long => fake::<i64>(),
            XsdType::NegativeInt => fake_signed::<i128>(false, false),
            XsdType::NonNegativeInt => fake_signed::<i128>(true, true),
            XsdType::NonPositiveInt => fake_signed::<i128>(false, true),
            XsdType::PositiveInt => fake_signed::<i128>(true, false),
            XsdType::UnsignedByte => fake::<u8>(),
            XsdType::UnsignedShort => fake::<u16>(),
            XsdType::UnsignedInt => fake::<u32>(),
            XsdType::UnsignedLong => fake::<u64>(),
            XsdType::Float => fake::<f32>(),
            XsdType::Double => fake::<f64>(),
            XsdType::URI => fake::<http::Uri>(),
            XsdType::Base64Binary => fake_base64(),
            XsdType::Boolean => fake::<bool>(),
            XsdType::HexBinary => fake_hex(),
            XsdType::Duration => fake::<Duration>(),
            XsdType::String(pattern) => self.generate_regex(pattern.as_str()),
            XsdType::DateTime(strf_time) => fake_date_format(strf_time.as_str()),
            XsdType::None => None,
        }
    }

    pub(crate) fn generate_type(&mut self, type_name: &str) -> Option<String> {
        let xsd_type = XsdType::from(type_name);
        self.generate_xsd_type(&xsd_type)
    }

    pub(crate) fn choose<'a, Item>(&mut self, vec: &'a [Item]) -> Option<&'a Item> {
        vec.choose(&mut self.rng)
    }

    fn sample_output(
        &mut self,
        xsd_type: &XsdType,
        output: &str,
        pattern: &str,
        depth: u16,
    ) -> Option<String> {
        if depth >= self.max_depth {
            return self.generate_regex(pattern);
        }

        for mat in Regex::new(pattern).unwrap().find_iter(output) {
            let string = mat.as_str();

            if string.is_empty() {
                continue;
            }

            return Some(string.to_string());
        }

        let next_output = self.generate_xsd_type(xsd_type).expect("No type generated");
        self.sample_output(xsd_type, &next_output, pattern, depth + 1)
    }

    fn generate_samples(&mut self, xsd_type: &XsdType, pattern: &str) -> Option<String> {
        let output = self.generate_xsd_type(&xsd_type).unwrap();
        self.sample_output(&xsd_type, &output, pattern, 0)
    }

    fn generate_date_pattern(&mut self, pattern: &str) -> Option<String> {
        // TODO Improve pattern
        let new_pattern = pattern
            .replace("[0-9]{4}", "%Y")
            .replace("[0-9][0-9][0-9][0-9]", "%Y")
            .replace("[0-9][0-9]", "%m");

        fake_date_format(&new_pattern)
    }

    fn generate_xsd_type_pattern(&mut self, xsd_type: &XsdType, pattern: &str) -> Option<String> {
        match xsd_type {
            XsdType::Byte => self.generate_samples(xsd_type, pattern),
            XsdType::Short => self.generate_samples(xsd_type, pattern),
            XsdType::Int => self.generate_samples(xsd_type, pattern),
            XsdType::Long => self.generate_samples(xsd_type, pattern),
            XsdType::NegativeInt => self.generate_samples(xsd_type, pattern),
            XsdType::NonNegativeInt => self.generate_samples(xsd_type, pattern),
            XsdType::NonPositiveInt => self.generate_samples(xsd_type, pattern),
            XsdType::PositiveInt => self.generate_samples(xsd_type, pattern),
            XsdType::UnsignedByte => self.generate_samples(xsd_type, pattern),
            XsdType::UnsignedShort => self.generate_samples(xsd_type, pattern),
            XsdType::UnsignedInt => self.generate_samples(xsd_type, pattern),
            XsdType::UnsignedLong => self.generate_samples(xsd_type, pattern),
            XsdType::Float => self.generate_samples(xsd_type, pattern),
            XsdType::Double => self.generate_samples(xsd_type, pattern),
            XsdType::URI => self.generate_samples(xsd_type, pattern),
            XsdType::Base64Binary => self.generate_samples(xsd_type, pattern),
            XsdType::Boolean => self.generate_samples(xsd_type, pattern),
            XsdType::HexBinary => self.generate_samples(xsd_type, pattern),
            XsdType::Duration => self.generate_samples(xsd_type, pattern),
            XsdType::String(_) => self.generate_samples(xsd_type, pattern),
            XsdType::DateTime(_) => self.generate_date_pattern(pattern),
            XsdType::None => self.generate_regex(pattern),
        }
    }

    pub(crate) fn generate_type_pattern(&mut self, pattern: &str, name: &str) -> Option<String> {
        let xsd_type = XsdType::from(name);
        self.generate_xsd_type_pattern(&xsd_type, pattern)
    }

    pub(crate) fn get_current_namespace(&self) -> Option<&String> {
        self.namespaces.last()
    }

    pub(crate) fn add_namespace(&mut self, namespace: String) {
        self.namespaces.push(namespace);
    }

    pub(crate) fn n_namespaces(&self) -> usize {
        self.namespaces.len()
    }

    pub(crate) fn update_namespaces(&mut self, n_namespaces: usize) {
        if n_namespaces == self.namespaces.len() {
            return;
        }

        if n_namespaces == self.namespaces.len() - 1 {
            self.namespaces.truncate(n_namespaces);
            return;
        }

        panic!("Additional namespaces not removed");
    }
}
