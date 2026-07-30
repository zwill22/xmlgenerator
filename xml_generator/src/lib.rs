//! The XML Generator crate provides methods for generating XML instances from XSD
//!
//! The crate provides the [XMLGenerator](struct@XMLGenerator) struct which manages all the tools needed for XML generation.
//! This includes the [XSDValidator] and [RegexTranslator] structs required to validate and generate XML from an XSD.
//!
//! Example
//!
//! ```rust
//! use std::path::PathBuf;
//! use xmlgenerator::XMLGenerator;
//!
//! fn run(filepath: &PathBuf) {
//!     let generator: XMLGenerator = XMLGenerator::new();
//!
//!     match generator.generate(filepath, None) {
//!         Ok(xml_output) => {
//!             println!("Successfully generated XML output");
//!             println!("{}", xml_output);
//!         },
//!         Err(e) => {
//!             eprintln!("Error generating output");
//!             eprintln!("Error: {:?}", e);
//!         },
//!     };
//! }
//! ```
//!
//! For more information, see: [XMLGenerator](struct@XMLGenerator)
//!
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

/// The main generator, should only be initialised once
///
/// The [XMLGenerator](struct@XMLGenerator) struct contains all the tools necessary to generate an XML string from an XSD file.
/// It uses an [XSDValidator] to validate the input and a [RegexTranslator] to translate `xsd::pattern` to Rust Regex.
/// A single instance of this class should be initialised using the [XMLGenerator::new] method.
/// It can then be used to validate multiple XSD files and generate XML files from them using the [XMLGenerator::validate] and [XMLGenerator::generate] methods respectively.
///
/// # Fields
///
/// - `validator` (`XSDValidator`) - [XSDValidator] to validate input XSD files
/// - `translator` (`RegexTranslator`) - [RegexTranslator] to translate any `xsd:pattern` to Rust Regex
///
/// # Example
///
/// ```rust
/// use std::path::PathBuf;
/// use xmlgenerator::XMLGenerator;
///
/// fn validate_and_generate(filepath: &PathBuf) -> String {
///     let generator: XMLGenerator = XMLGenerator::new();
///
///     // Validate XSD
///     if let Err(e) = generator.validate(filepath) {
///         panic!("Invalid XSD: {:?}", e);
///     }
///
///     // Generate output
///     match generator.generate(filepath, None) {
///         Ok(xml_output) => {
///             println!("Successfully generated XML output");
///             return xml_output;
///         },
///         Err(e) => {
///             panic!("Error generating output: {:?}", e);
///         },
///     };
/// }
/// ```
///
/// The [XMLGenerator::generate] method includes a validation step, so calling the [XMLGenerator::validate] method separately is often unnecessary.
/// However, it can be useful to check the validity of the file before using the generator.
///
pub struct XMLGenerator {
    validator: XSDValidator,
    translator: RegexTranslator,
}

impl Default for XMLGenerator {
    fn default() -> Self {
        let validator = XSDValidator::new(false);
        let translator = RegexTranslator::new();

        XMLGenerator {
            validator,
            translator,
        }
    }
}

impl XMLGenerator {
    /// Constructor for the [XMLGenerator](struct@XMLGenerator) struct.
    ///
    /// # Returns
    ///
    /// - [XMLGenerator](struct@XMLGenerator) - A generator object
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xmlgenerator::XMLGenerator;
    ///
    /// // A new `XMLGenerator` instance
    /// let _ = XMLGenerator::new();
    /// ```
    ///
    pub fn new() -> XMLGenerator {
        XMLGenerator::default()
    }

    /// Validate an XSD file at location `path`.
    ///
    /// If the input file is valid, then the method returns an empty result.
    /// If the file is invalid, then the method throws an [XMLGeneratorError].
    ///
    /// # Arguments
    ///
    /// - `path` (`&PathBuf`) - Path to the input XSD file
    ///
    /// # Returns
    ///
    /// - `Result<(), XMLGeneratorError>` - Error if file is invalid, else nothing
    ///
    /// # Errors
    ///
    /// - [XMLGeneratorError::InvalidPathError] - If `path` is invalid or file does not exist
    /// - [XMLGeneratorError::XSDValidatorError] - If the [XSDValidator] returns an error
    /// - [XMLGeneratorError::InvalidXSDError] - If recursive elements found in XSD
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use xmlgenerator::XMLGenerator;
    ///
    /// fn validate_xsd(filepath: &PathBuf) -> bool {
    ///     let generator = XMLGenerator::new();
    ///
    ///     match generator.validate(filepath) {
    ///         Ok(_) => {
    ///             return true;
    ///         },
    ///         Err(e) => {
    ///             eprintln!("XSDValidator error: {:?}", e);
    ///             return false;
    ///         },
    ///     }
    /// }
    /// ```
    ///
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
    ///
    /// # Arguments
    ///
    /// - `xsd_path` (`&PathBuf`) - Path to XSD file
    /// - `seed` (`Option<u64>`) - Optional unsigned integer to use as a random seed
    ///
    /// # Returns
    ///
    /// - `Result<String, XMLGeneratorError>` - XML string is returned on success, but an [XMLGeneratorError] is returned on failure.
    ///
    /// # Errors
    ///
    /// - [XMLGeneratorError::InvalidPathError] - If `xsd_path` is invalid, or file does not exist
    /// - [XMLGeneratorError::XSDValidatorError] - [XSDValidator] throws an error during validation.
    /// - [XMLGeneratorError::DataTypeInformationError] - A data-type contains invalid information
    /// - [XMLGeneratorError::DataTypeNotFoundError] - A data-type cannot be found
    /// - [XMLGeneratorError::XSDParserError] - Error parsing XSD
    /// - [XMLGeneratorError::DataTypesFormatError] - Data type in an invalid format
    /// - [XMLGeneratorError::XMLBuilderError] - Error while building the final XML output
    /// - [XMLGeneratorError::InvalidXSDVersionError] - Invalid version of the XML Schema (not 1.0 or 1.1)
    /// - [XMLGeneratorError::InfiniteRecursionError] - Infinite recursion found in XSD
    /// - [XMLGeneratorError::NoElementsError] - XSD does not contain any elements
    /// - [XMLGeneratorError::InvalidXSDError] - XSD is invalid
    /// - [XMLGeneratorError::InvalidXSDNameError] - XSD includes an invalid name
    /// - [XMLGeneratorError::NoIndependentElementsError] - XSD does not contain a root element
    /// - [XMLGeneratorError::MultipleRootsError] - XSD contains multiple possible root elements
    /// - [XMLGeneratorError::TypeGenerationError] - Unable to generate a data type
    /// - [XMLGeneratorError::RegexError] - Error processing Regex pattern
    /// - [XMLGeneratorError::RegexMismatchError] - Data-type is constrained to two incompatible regex patterns
    /// - [XMLGeneratorError::LineEndingsError] - Pattern includes invalid line endings
    /// - [XMLGeneratorError::EncodingError] - XSD specified an invalid encoding
    /// - [XMLGeneratorError::UnimplementedFeature] - XSD includes an unimplemented feature
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// use xmlgenerator::XMLGenerator;
    ///
    /// fn generate_xsd(filepath: &PathBuf) {
    ///     let generator: XMLGenerator = XMLGenerator::new();
    ///
    ///     match generator.generate(filepath, None) {
    ///         Ok(xml_output) => {
    ///             println!("Successfully generated XML output");
    ///             println!("{}", xml_output);
    ///         },
    ///         Err(e) => {
    ///             eprintln!("Error generating XML: {:?}", e);
    ///         },
    ///     };
    /// }
    /// ```
    /// 
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
