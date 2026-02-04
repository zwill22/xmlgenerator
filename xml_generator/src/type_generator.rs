use regextranslator::RegexTranslator;
use crate::XMLGeneratorError;

pub(crate) struct TypeGenerator {
    translator: RegexTranslator
}

impl<'a> TypeGenerator {
    pub(crate) fn new() -> Self {
        let translator = RegexTranslator::new().unwrap();

        Self {
            translator
        }
    }

    pub(crate) fn translate(&self, pattern: &str) -> Result<String, XMLGeneratorError> {
        Ok(self.translator.translate(pattern)?)
    }
}