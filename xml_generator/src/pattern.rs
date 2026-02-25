use crate::XMLGeneratorError;
use crate::whitespace::WhiteSpace;
use regex_intersect::non_empty;
use regextranslator::RegexTranslator;

#[derive(PartialEq, Default)]
pub(crate) struct Pattern {
    original: String,
    whitespace: WhiteSpace,
    ascii: Option<String>,
    full: Option<String>,
}

impl Pattern {
    pub(crate) fn new(
        regex_translator: &RegexTranslator,
        whitespace: &WhiteSpace,
        input: &str,
    ) -> Result<Self, XMLGeneratorError> {
        let mut pattern = Self::default();

        let original = whitespace.handle(input)?;

        pattern.original = original.clone();
        pattern.whitespace = whitespace.clone();

        let full_translation = regex_translator.translate(&original, false)?;
        if full_translation == original {
            return Ok(pattern);
        }

        pattern.full = Some(full_translation.clone());

        let ascii_translation = regex_translator.translate(&original, true)?;
        if ascii_translation == original {
            return Ok(pattern);
        }

        if ascii_translation.eq(&full_translation) {
            return Ok(pattern);
        }

        pattern.ascii = Some(ascii_translation.clone());

        Ok(pattern)
    }

    pub(crate) fn get_whitespace(&self) -> WhiteSpace {
        self.whitespace.clone()
    }

    pub(crate) fn get_pattern(&self, ascii: bool) -> &str {
        if ascii {
            if let Some(ascii) = &self.ascii {
                return ascii;
            }

            if let Some(full) = &self.full {
                return full;
            }

            &self.original
        } else {
            if let Some(full) = &self.full {
                return full;
            }

            &self.original
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.original.is_empty()
    }

    pub(crate) fn from_string(s: &str, whitespace: &WhiteSpace) -> Result<Self, XMLGeneratorError> {
        let mut pattern = Self::default();

        let original = whitespace.handle(s)?;
        pattern.original = original.clone();
        pattern.whitespace = whitespace.clone();

        const I: &str = r"\i";
        const I_ASCII: &str = r"[:A-Z_a-z]";
        const I_FULL: &str = r"[:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]";

        const C: &str = r"\c";
        const C_ASCII: &str = r"[-.0-9:A-Z_a-z]";
        const C_FULL: &str = r"[-.0-9:A-Z_a-z\u00B7\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u037D\u037F-\u1FFF\u200C-\u200D\u203F\u2040\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\x{10000}-\x{EFFFF}]";

        let output = s.replace(C, C_FULL).replace(I, I_FULL);
        if output == s {
            return Ok(pattern);
        }

        pattern.full = Some(output.clone());

        let ascii = s.replace(I, I_ASCII).replace(C, C_ASCII);

        if ascii == s {
            return Ok(pattern);
        }

        if ascii == output {
            return Ok(pattern);
        }

        pattern.ascii = Some(ascii.clone());

        Ok(pattern)
    }

    pub(crate) fn check_intersection(&self, other: &Pattern) -> Result<(), XMLGeneratorError> {
        let pattern1 = self.get_pattern(true);
        let pattern2 = other.get_pattern(true);

        let result = non_empty(pattern1, pattern2)?;
        if result {
            return Ok(());
        }

        let full_pattern1 = self.get_pattern(false);
        let full_pattern2 = other.get_pattern(false);

        let full_result = non_empty(pattern1, pattern2)?;
        if !full_result {
            return Err(XMLGeneratorError::RegexMismatchError(
                full_pattern1.to_string(),
                full_pattern2.to_string(),
            ));
        }

        Ok(())
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.original)
    }
}
