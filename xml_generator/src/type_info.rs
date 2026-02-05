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
                SimpleBaseTypeContent::Restriction(restriction) => {
                    type_info.get_restriction(translator, restriction)?
                }
                SimpleBaseTypeContent::Annotation(_) => {
                    unimplemented("SimpleBaseTypeContent::Annotation")?
                }
                SimpleBaseTypeContent::List(_) => unimplemented("SimpleBaseTypeContent::List")?,
                SimpleBaseTypeContent::Union(_) => unimplemented("SimpleBaseTypeContent::Union")?,
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
            Facet::Enumeration(enumeration) => self.handle_enumeration(enumeration),
            Facet::Pattern(pattern) => self.handle_pattern_facet(translator, pattern),
            Facet::MinExclusive(_) => unimplemented("Facet::MinExclusive")?,
            Facet::MinInclusive(_) => unimplemented("Facet::MinInclusive")?,
            Facet::MaxExclusive(_) => unimplemented("Facet::MaxExclusive")?,
            Facet::MaxInclusive(_) => unimplemented("Facet::MaxInclusive")?,
            Facet::TotalDigits(_) => unimplemented("Facet::TotalDigits")?,
            Facet::FractionDigits(_) => unimplemented("Facet::FractionDigits")?,
            Facet::Length(_) => unimplemented("Facet::Length")?,
            Facet::MinLength(_) => unimplemented("Facet::MinLength")?,
            Facet::MaxLength(_) => unimplemented("Facet::MaxLength")?,
            Facet::WhiteSpace(_) => unimplemented("Facet::WhiteSpace")?,
            Facet::Assertion(_) => unimplemented("Facet::Assertion")?,
            Facet::ExplicitTimezone(_) => unimplemented("Facet::ExplicitTimezone")?,
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
                RestrictionContent::Facet(facet) => self.handle_facet(translator, facet)?,
                RestrictionContent::Annotation(_) => {
                    unimplemented("RestrictionContent::Annotation")?
                }
                RestrictionContent::SimpleType(_) => {
                    unimplemented("RestrictionContent::SimpleType")?
                }
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

            return generator.choose(&self.enumerations).cloned();
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
                None => {},
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
