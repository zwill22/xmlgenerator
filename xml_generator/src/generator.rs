use crate::XMLGeneratorError;
use crate::datetime::Datetime;
use crate::element::Element;
use crate::pattern::Pattern;
use crate::whitespace::check_line_endings;
use crate::xsd_type::XsdType;
use chrono::Duration;
use fake::{Fake, Faker};
use rand::prelude::{IndexedRandom, ThreadRng};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use regex::Regex;
use std::collections::HashSet;
use xsd_parser::models::schema::SchemaInfo;
use xsd_parser::models::schema::xs::FormChoiceType;

fn make_fake<Output: fake::Dummy<Faker> + ToString>() -> Option<Output> {
    Some(Faker.fake::<Output>())
}

fn fake<Input: fake::Dummy<Faker> + ToString>() -> Option<String> {
    make_fake::<Input>().map(|output| output.to_string())
}

pub(crate) struct Generator {
    rng: ThreadRng,
    xor: XorShiftRng,
    max_depth: usize,
    max_repeat: u32,
    regex_patterns: usize,
    namespaces: Vec<String>,
    tracker: HashSet<String>,
    references: Vec<String>,
    element_qualified: bool,
    attribute_qualified: bool,
}

impl Generator {
    pub(crate) fn new(regex_patterns: usize, max_repeat: u32, max_depth: usize) -> Generator {
        let rng = rand::rng();
        let xor = XorShiftRng::from_os_rng();
        Generator {
            rng,
            xor,
            regex_patterns,
            max_depth,
            max_repeat,
            namespaces: vec![],
            tracker: HashSet::new(),
            references: vec![],
            element_qualified: false,
            attribute_qualified: false,
        }
    }

    pub(crate) fn is_root(&self) -> bool {
        self.tracker.is_empty()
    }

    pub(crate) fn track_ref(&mut self, reference: &str) {
        self.references.push(reference.to_string());
    }

    pub(crate) fn untrack_ref(&mut self, reference: &str) -> Result<(), XMLGeneratorError> {
        match self.references.pop() {
            Some(value) => {
                if reference != value {
                    return Err(XMLGeneratorError::InvalidXSDError(
                        "Unknown reference".to_string(),
                    ));
                }

                Ok(())
            }
            None => Err(XMLGeneratorError::InvalidXSDError(
                "Reference not found".to_string(),
            )),
        }
    }

    pub(crate) fn set_qualification(&mut self, schema_info: &SchemaInfo) {
        let schema = &schema_info.schema;

        match schema.element_form_default {
            FormChoiceType::Qualified => self.element_qualified = true,
            FormChoiceType::Unqualified => {}
        }

        match schema.attribute_form_default {
            FormChoiceType::Qualified => self.attribute_qualified = true,
            FormChoiceType::Unqualified => {}
        }
    }

    pub(crate) fn attributes_qualified(&mut self) -> bool {
        self.attribute_qualified
    }

    pub(crate) fn requires_full_name(&self, name: &str) -> bool {
        if self.element_qualified {
            return true;
        }
        let reference = match self.references.last() {
            Some(reference) => reference,
            None => return false,
        };

        if reference == name {
            return true;
        }

        false
    }

    fn includes(&self, element: &Element) -> bool {
        let id = element.get_id();
        self.tracker.contains(&id)
    }

    pub(crate) fn track(&mut self, element: &Element) -> Result<(), XMLGeneratorError> {
        if self.includes(element) {
            return Err(XMLGeneratorError::InfiniteRecursionError);
        }

        let id = element.get_id();
        self.tracker.insert(id);

        Ok(())
    }

    pub(crate) fn untrack(&mut self, element: &Element) {
        let id = element.get_id();
        let result = self.tracker.remove(&id);
        if !result {
            panic!("Element not in hierarchy");
        }
    }

    fn sample(&mut self, regex: rand_regex::Regex) -> HashSet<String> {
        (&mut self.xor)
            .sample_iter(&regex)
            .take(self.regex_patterns)
            .collect::<HashSet<String>>()
    }

    fn check_valid(&mut self, sample: &str) -> bool {
        match check_line_endings(sample) {
            Ok(()) => true,
            Err(_) => false,
        }
    }

    fn regex_samples(&mut self, pattern: &Pattern, ascii: bool) -> HashSet<String> {
        let re = pattern.get_pattern(ascii);

        let regex = match rand_regex::Regex::compile(re, self.max_repeat) {
            Ok(regex) => regex,
            Err(_) => return HashSet::new(),
        };

        let samples = self.sample(regex);

        samples
            .iter()
            .filter(|s| self.check_valid(s))
            .cloned()
            .collect()
    }

    fn regex(&mut self, pattern: &Pattern, ascii: bool) -> Option<String> {
        let samples = self.regex_samples(pattern, ascii);

        if samples.len() == 1 {
            return samples.into_iter().next();
        }

        for sample in &samples {
            if sample.is_empty() {
                continue;
            }

            return Some(sample.clone());
        }

        None
    }

    pub(crate) fn generate_regex(&mut self, pattern: &Pattern) -> Option<String> {
        self.regex(pattern, true)
    }

    fn fake_datetime(&mut self, datetime: &Datetime) -> Option<String> {
        datetime.generate(&mut self.rng)
    }

    fn fake_string(&mut self, pattern: &Pattern, ascii: bool) -> Option<String> {
        if pattern.is_empty() {
            return fake::<String>();
        }

        self.regex(pattern, ascii)
    }

    fn type_generate(&mut self, xsd_type: &XsdType, ascii: bool) -> Option<String> {
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
            XsdType::Uri => fake::<http::Uri>(),
            // XsdType::Base64Binary => fake_base64(),
            // XsdType::HexBinary => fake_hex(),
            XsdType::Duration => fake::<Duration>(),
            XsdType::DateTime(datetime) => self.fake_datetime(datetime),
            XsdType::String(pattern) => self.fake_string(pattern, ascii),
            XsdType::None => None,
        }
    }

    pub(crate) fn generate_type(&mut self, xsd_type: &XsdType) -> Option<String> {
        self.type_generate(xsd_type, true)
    }

    pub(crate) fn choose<'a, Item>(&mut self, vec: &'a [Item]) -> Option<&'a Item> {
        vec.choose(&mut self.rng)
    }

    fn find_match(pattern: &Pattern, input: &str, ascii: bool) -> Option<String> {
        let re = pattern.get_pattern(ascii);
        for mat in Regex::new(re).unwrap().find_iter(input) {
            let string = mat.as_str();

            if string.is_empty() {
                continue;
            }

            return Some(string.to_string());
        }

        None
    }

    fn generate_pattern(
        &mut self,
        xsd_type: &XsdType,
        pattern: &Pattern,
        ascii: bool,
    ) -> Option<String> {
        let samples = self.regex_samples(pattern, ascii);
        for sample in samples {
            if xsd_type.validate(&sample) {
                return Some(sample);
            }
        }

        let output = self
            .type_generate(xsd_type, ascii)
            .expect("No type generated");

        Generator::find_match(pattern, &output, ascii)
    }

    fn cross_match(
        &mut self,
        pattern1: &Pattern,
        pattern2: &Pattern,
        ascii: bool,
    ) -> Option<String> {
        let samples1 = self.regex_samples(pattern1, ascii);
        for sample in samples1 {
            if let Some(output) = Generator::find_match(pattern2, &sample, ascii) {
                return Some(output);
            }
        }

        None
    }

    fn generate_date_with_pattern(
        &mut self,
        datetime: &Datetime,
        pattern: &Pattern,
    ) -> Option<String> {
        if pattern.is_empty() {
            return datetime.generate(&mut self.rng);
        }

        for _ in 1..self.regex_patterns {
            if let Some(out_date) = datetime.generate(&mut self.rng)
                && let Some(output) = Generator::find_match(pattern, &out_date, true)
                && datetime.matches(&output)
            {
                return Some(output);
            }
        }

        self.generate_date_with_pattern(datetime, pattern)
    }

    fn generate_two_patterns(
        &mut self,
        base_pattern: &Pattern,
        specific_pattern: &Pattern,
        depth: usize,
    ) -> Option<String> {
        // TODO Implement proper error
        if depth > self.max_depth {
            panic!("No pattern generated after {} attempts", self.max_depth);
        }

        if base_pattern.is_empty() {
            return self.regex(specific_pattern, true);
        }

        if specific_pattern.is_empty() {
            return self.regex(base_pattern, true);
        }

        for ascii in [true, false] {
            if let Some(output) = self.cross_match(base_pattern, specific_pattern, ascii) {
                return Some(output);
            };

            if let Some(output) = self.cross_match(specific_pattern, base_pattern, ascii) {
                return Some(output);
            };
        }

        self.generate_two_patterns(base_pattern, specific_pattern, depth + 1)
    }

    pub(crate) fn generate_type_pattern(
        &mut self,
        xsd_type: &XsdType,
        pattern: &Pattern,
    ) -> Option<String> {
        const ASCII: bool = true;
        match xsd_type {
            XsdType::Byte => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::Short => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::Int => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::Long => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::UnsignedByte => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::UnsignedShort => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::UnsignedInt => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::UnsignedLong => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::Float => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::Double => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::Uri => self.generate_pattern(xsd_type, pattern, ASCII),
            // XsdType::Base64Binary => self.generate_pattern(xsd_type, pattern, ASCII),
            // XsdType::HexBinary => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::Duration => self.generate_pattern(xsd_type, pattern, ASCII),
            XsdType::DateTime(datetime) => self.generate_date_with_pattern(datetime, pattern),
            XsdType::String(string) => self.generate_two_patterns(string, pattern, 0),
            XsdType::None => self.regex(pattern, ASCII),
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
