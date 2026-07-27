pub use crate::error::XMLGeneratorError;
use crate::generator::Generator;
use crate::schemas::SchemaData;
use crate::xsd::Xsd;
use regextranslator::RegexTranslator;
use std::path::PathBuf;
use xsdvalidator::XSDValidator;

mod attribute;
mod data_type;
mod datetime;
mod element;
mod encoder;
pub mod error;
mod generator;
mod group;
mod name;
mod namespaces;
mod pattern;
mod schema_version;
mod schemas;
mod type_info;
mod whitespace;
mod xsd;
mod xsd_type;

pub struct XMLGenerator {
    validator: XSDValidator,
    translator: RegexTranslator,
}

impl Default for XMLGenerator {
    fn default() -> Self {
        let validator = XSDValidator::new(false);
        let translator = RegexTranslator::new().unwrap();

        XMLGenerator {
            validator,
            translator,
        }
    }
}

impl XMLGenerator {
    pub fn new() -> XMLGenerator {
        XMLGenerator::default()
    }

    pub fn validate(&self, path: &PathBuf) -> Result<(), XMLGeneratorError> {
        let valid = self.validator.validate(path)?;

        if valid {
            return Ok(());
        }

        Err(XMLGeneratorError::XSDValidatorError(
            "Invalid XSD".to_string(),
        ))
    }

    /// Generate an XML string containing fake data
    ///
    /// Using an XSD file contents as a string, generate an XML file string of the
    /// same format with fake data.
    ///
    /// The function uses the `xsd_parser` crate to parse the input. If this library returns
    /// en error, then the function returns an `XMLGeneratorError::XMLParserError`.
    /// This crate generates a `data_types` object which the XMLGenerator uses th
    /// generate the output XML.
    ///
    /// If the `data_types` contains data which is not in the required format, then an
    /// `XMLGeneratorError::DataTypeFormatError` is returned. This includes cases
    /// such as multiple root nodes or circular dependencies.
    ///
    /// The function sorts the data into a dependency tree and uses this to generate an
    /// `XMLBuilder` object using the `xml_builder` crate. If the `XMLBuilder` returns
    /// an error when generating the output XML, then an `XMLGeneratorError::XMLBuilderError`
    /// is returned.
    pub fn generate(
        &self,
        xsd_path: &PathBuf,
        seed: Option<u64>,
    ) -> Result<String, XMLGeneratorError> {
        let schemas = SchemaData::new(xsd_path)?;
        self.validate(xsd_path)?;

        let mut generator = Generator::new(10000, 10, 10000, seed);

        let xsd = Xsd::new(&mut generator, &self.translator, &schemas)?;

        xsd.generate_xml(&mut generator)
    }
}
