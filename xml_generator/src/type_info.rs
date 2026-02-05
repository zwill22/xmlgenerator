use crate::XMLGeneratorError;
use crate::error::unimplemented;
use crate::generator::Generator;
use regex::Regex;
use regextranslator::RegexTranslator;
use xsd_parser::models::schema::xs::{
    Facet, FacetType, Restriction, RestrictionContent, SimpleBaseTypeContent,
};

fn check_carriage_returns(pattern: &str) -> Result<(), XMLGeneratorError> {
    let stripped = pattern.replace("\\r\\n", "");

    if stripped.contains("\\r") {
        return Err(XMLGeneratorError::UnimplementedFeature(
            "Carriage returns".to_string(),
        ));
    }

    Ok(())
}

fn handle_pattern(
    translator: &RegexTranslator,
    pattern: &str,
) -> Result<String, XMLGeneratorError> {
    check_carriage_returns(pattern)?;

    match Regex::new(pattern) {
        Ok(regex) => {
            // Generalise `\d` pattern to equal `[0-9]`
            if pattern.contains(r"\d") {
                let new_pattern = pattern.replace(r"\d", r"[0-9]");
                return handle_pattern(translator, &new_pattern);
            }

            Ok(regex.to_string())
        }
        Err(pattern_err) => {
            let translation = translator.translate(pattern)?;

            if translation.as_str() == pattern {
                let error = format!("Compilation error: {}", pattern_err);
                return Err(XMLGeneratorError::RegexError(error));
            }

            handle_pattern(translator, &translation)
        }
    }
}

pub(crate) struct TypeInfo {
    name: String,
    pattern: Option<String>,
    enumerations: Vec<String>,
}

impl TypeInfo {
    pub(crate) fn new(
        translator: &RegexTranslator,
        base_content: &Vec<SimpleBaseTypeContent>,
    ) -> Result<Self, XMLGeneratorError> {
        let mut type_info = TypeInfo {
            name: String::new(),
            pattern: None,
            enumerations: Vec::new(),
        };

        for content in base_content {
            match content {
                SimpleBaseTypeContent::Annotation(_) => unimplemented("Annotation")?,
                SimpleBaseTypeContent::Restriction(x) => {
                    type_info.get_restriction(translator, x)?
                }
                SimpleBaseTypeContent::List(_) => unimplemented("List")?,
                SimpleBaseTypeContent::Union(_) => unimplemented("Union")?,
            }
        }

        Ok(type_info)
    }

    fn handle_enumeration(&mut self, enumeration: &FacetType) -> Result<(), XMLGeneratorError> {
        let value = &enumeration.value;

        self.enumerations.push(value.clone());

        Ok(())
    }

    fn handle_pattern_facet(
        &mut self,
        translator: &RegexTranslator,
        pattern_facet: &FacetType,
    ) -> Result<(), XMLGeneratorError> {
        let pattern = pattern_facet.value.as_str();

        let output = handle_pattern(translator, pattern)?;
        self.pattern = Some(output);
        Ok(())
    }

    fn handle_facet(
        &mut self,
        translator: &RegexTranslator,
        facet: &Facet,
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
            Facet::Enumeration(enumeration) => self.handle_enumeration(enumeration),
            Facet::WhiteSpace(_) => unimplemented("WhiteSpace facet"),
            Facet::Pattern(pattern) => self.handle_pattern_facet(translator, pattern),
            Facet::Assertion(_) => unimplemented("Assertion facet"),
            Facet::ExplicitTimezone(_) => unimplemented("ExplicitTimezone facet"),
        }
    }

    fn get_restriction(
        &mut self,
        translator: &RegexTranslator,
        restriction: &Restriction,
    ) -> Result<(), XMLGeneratorError> {
        if let Some(base) = &restriction.base {
            self.name = String::from_utf8(base.local_name().to_vec()).unwrap()
        }

        for content in &restriction.content {
            match content {
                RestrictionContent::Annotation(_) => unimplemented("Annotation")?,
                RestrictionContent::SimpleType(_) => unimplemented("SimpleType")?,
                RestrictionContent::Facet(facet) => self.handle_facet(translator, facet)?,
            }
        }

        Ok(())
    }

    pub(crate) fn generate(&self, generator: &mut Generator) -> Option<String> {
        let name = &self.name;
        if !self.enumerations.is_empty() {
            if self.pattern.is_some() {
                panic!("Type info includes enumeration and pattern data");
            }

            return generator.generate_enumeration(&self.enumerations);
        }

        if let Some(pattern) = &self.pattern {
            return generator.generate_pattern(pattern, name);
        }

        generator.generate_type(name)
    }

    pub(crate) fn get_name(&self) -> String {
        self.name.clone()
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
