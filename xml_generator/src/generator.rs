use crate::XMLGeneratorError;
use crate::xsd_type::XsdType;
use chrono::Duration;
use fake::{Fake, Faker};
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
    max_repeat: u32,
    regex_patterns: usize,
    namespaces: Vec<String>,
}

impl Generator {
    pub(crate) fn new(regex_patterns: usize, max_repeat: u32) -> Generator {
        let rng = rand::rng();
        let xor = XorShiftRng::from_os_rng();
        Generator {
            rng,
            xor,
            regex_patterns,
            max_repeat,
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
            .take(self.regex_patterns)
            .collect::<HashSet<String>>()
    }

    fn regex_samples(&mut self, pattern: &str) -> HashSet<String> {
        let regex = match rand_regex::Regex::compile(pattern, self.max_repeat) {
            Ok(regex) => regex,
            Err(_) => return HashSet::new(),
        };

        self.sample(regex)
    }

    pub(crate) fn generate_regex(&mut self, pattern: &str) -> Option<String> {
        let samples = self.regex_samples(pattern);

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

    fn fake_string(&mut self, pattern: &str) -> Option<String> {
        if pattern.is_empty() {
            return fake::<String>();
        }

        self.generate_regex(pattern)
    }

    pub(crate) fn generate_type(&mut self, xsd_type: &XsdType) -> Option<String> {
        match xsd_type {
            XsdType::Byte => fake::<i8>(),
            XsdType::Short => fake::<i16>(),
            XsdType::Int => fake::<i32>(),
            XsdType::Long => fake::<i64>(),
            XsdType::UnsignedByte => fake::<u8>(),
            XsdType::UnsignedShort => fake::<u16>(),
            XsdType::UnsignedInt => fake::<u32>(),
            XsdType::UnsignedLong => fake::<u64>(),
            XsdType::Float => fake::<f32>(),
            XsdType::Double => fake::<f64>(),
            XsdType::URI => fake::<http::Uri>(),
            XsdType::Base64Binary => fake_base64(),
            XsdType::HexBinary => fake_hex(),
            XsdType::Duration => fake::<Duration>(),
            XsdType::String(pattern) => self.fake_string(pattern.as_str()),
            XsdType::None => None,
        }
    }

    pub(crate) fn choose<'a, Item>(&mut self, vec: &'a [Item]) -> Option<&'a Item> {
        vec.choose(&mut self.rng)
    }

    fn find_match(pattern: &str, input: &str) -> Option<String> {
        for mat in Regex::new(pattern).unwrap().find_iter(input) {
            let string = mat.as_str();

            if string.is_empty() {
                continue;
            }

            return Some(string.to_string());
        }

        None
    }

    fn generate_pattern(&mut self, xsd_type: &XsdType, pattern: &str) -> Option<String> {
        let samples = self.regex_samples(pattern);
        for sample in samples {
            if xsd_type.is_valid(&sample) {
                return Some(sample);
            }
        }

        let output = self.generate_type(xsd_type).expect("No type generated");

        Generator::find_match(pattern, &output)
    }

    fn cross_match(&mut self, pattern1: &str, pattern2: &str) -> Option<String> {
        let samples1 = self.regex_samples(pattern1);
        for sample in samples1 {
            match Generator::find_match(pattern2, &sample) {
                Some(output) => {
                    return Some(output);
                }
                None => {}
            }
        }

        None
    }

    fn generate_two_patterns(&mut self, pattern1: &str, pattern2: &str) -> Option<String> {
        if pattern1.is_empty() {
            return self.generate_regex(pattern2);
        }

        if pattern2.is_empty() {
            return self.generate_regex(pattern1);
        }

        match self.cross_match(pattern1, pattern2) {
            Some(output) => return Some(output),
            None => {}
        }

        self.cross_match(pattern2, pattern1)
    }

    pub(crate) fn generate_type_pattern(
        &mut self,
        xsd_type: &XsdType,
        pattern: &str,
    ) -> Option<String> {
        match xsd_type {
            XsdType::Byte => self.generate_pattern(&xsd_type, pattern),
            XsdType::Short => self.generate_pattern(&xsd_type, pattern),
            XsdType::Int => self.generate_pattern(&xsd_type, pattern),
            XsdType::Long => self.generate_pattern(&xsd_type, pattern),
            XsdType::UnsignedByte => self.generate_pattern(&xsd_type, pattern),
            XsdType::UnsignedShort => self.generate_pattern(&xsd_type, pattern),
            XsdType::UnsignedInt => self.generate_pattern(&xsd_type, pattern),
            XsdType::UnsignedLong => self.generate_pattern(&xsd_type, pattern),
            XsdType::Float => self.generate_pattern(&xsd_type, pattern),
            XsdType::Double => self.generate_pattern(&xsd_type, pattern),
            XsdType::URI => self.generate_pattern(&xsd_type, pattern),
            XsdType::Base64Binary => self.generate_pattern(&xsd_type, pattern),
            XsdType::HexBinary => self.generate_pattern(&xsd_type, pattern),
            XsdType::Duration => self.generate_pattern(&xsd_type, pattern),
            XsdType::String(string) => self.generate_two_patterns(&string, pattern),
            XsdType::None => self.generate_regex(pattern),
        }
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
