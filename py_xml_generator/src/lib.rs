use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyRuntimeError};
use pyo3::prelude::*;
use std::any::Any;
use std::panic;
use std::path::PathBuf;
use xmlgenerator::error::XMLGeneratorError;
use xmlgenerator::XMLGenerator;

create_exception!(
    pyxmlgenerator,
    InvalidPathError,
    PyException,
    "Invalid filepath or a non-existent file"
);
create_exception!(
    pyxmlgenerator,
    XSDValidatorError,
    PyException,
    "XSDValidator raised an error"
);
create_exception!(
    pyxmlgenerator,
    DataTypeInformationError,
    PyException,
    "No data found for custom type"
);
create_exception!(
    pyxmlgenerator,
    DataTypeNotFoundError,
    PyException,
    "Unable to find a given data type"
);
create_exception!(
    pyxmlgenerator,
    XSDParserError,
    PyException,
    "Error while parsing the XSD file"
);
create_exception!(
    pyxmlgenerator,
    DataTypesFormatError,
    PyException,
    "A data type is in an invalid format"
);
create_exception!(
    pyxmlgenerator,
    XMLBuilderError,
    PyException,
    "Error while building the final XML output"
);
create_exception!(
    pyxmlgenerator,
    InvalidXSDVersionError,
    PyException,
    "XSD specifies an invalid XML version"
);
create_exception!(
    pyxmlgenerator,
    InfiniteRecursionError,
    PyException,
    "Infinite recursion encountered"
);
create_exception!(
    pyxmlgenerator,
    NoElementsError,
    PyException,
    "No elements in XSD"
);
create_exception!(
    pyxmlgenerator,
    InvalidXSDError,
    PyException,
    "XSD is invalid"
);
create_exception!(
    pyxmlgenerator,
    NoIndependentElementsError,
    PyException,
    "No root element found in XSD"
);
create_exception!(
    pyxmlgenerator,
    MultipleXSDRootsError,
    PyException,
    "XSD contains multiple root elements"
);
create_exception!(
    pyxmlgenerator,
    TypeGenerationError,
    PyException,
    "Error generating a specific type"
);
create_exception!(
    pyxmlgenerator,
    ImplementationError,
    PyException,
    "Unimplemented feature in XSD"
);
create_exception!(
    pyxmlgenerator,
    RegexError,
    PyException,
    "Invalid XSD pattern or Regex"
);
create_exception!(
    pyxmlgenerator,
    IncompatiblePatternError,
    PyException,
    "XSD restricts type to incompatible Regex patterns"
);
create_exception!(
    pyxmlgenerator,
    InvalidXSDNameError,
    PyException,
    "XSD contains invalid name"
);
create_exception!(
    pyxmlgenerator,
    LineEndingsError,
    PyException,
    "Invalid line endings in XSD pattern"
);
create_exception!(
    pyxmlgenerator,
    XSDEncodingError,
    PyException,
    "XSD uses invalid encoding"
);

fn handle_error(error: XMLGeneratorError) -> PyErr {
    match error {
        XMLGeneratorError::InvalidPathError(e) => InvalidPathError::new_err(e),
        XMLGeneratorError::XSDValidatorError(e) => XSDValidatorError::new_err(e),
        XMLGeneratorError::DataTypeInformationError(e) => DataTypeInformationError::new_err(e),
        XMLGeneratorError::DataTypeNotFoundError(e) => DataTypeNotFoundError::new_err(e),
        XMLGeneratorError::XSDParserError(e) => XSDParserError::new_err(e),
        XMLGeneratorError::DataTypesFormatError(e) => DataTypesFormatError::new_err(e),
        XMLGeneratorError::XMLBuilderError(e) => XMLBuilderError::new_err(e),
        XMLGeneratorError::InvalidXSDVersionError(e) => InvalidXSDVersionError::new_err(e),
        XMLGeneratorError::InfiniteRecursionError => {
            InfiniteRecursionError::new_err("Infinite Recursion error")
        }
        XMLGeneratorError::NoElementsError => NoElementsError::new_err("No elements found in XSD"),
        XMLGeneratorError::InvalidXSDError(e) => InvalidXSDError::new_err(e),
        XMLGeneratorError::NoIndependentElementsError => {
            NoIndependentElementsError::new_err("No Independent Elements found in XSD")
        }
        XMLGeneratorError::MultipleRootsError => MultipleXSDRootsError::new_err("Multiple Roots"),
        XMLGeneratorError::TypeGenerationError(e) => TypeGenerationError::new_err(e),
        XMLGeneratorError::RegexError(e) => RegexError::new_err(e),
        XMLGeneratorError::RegexMismatchError(p1, p2) => {
            let e = format!("Incompatible patterns: {} {}", p1, p2);
            IncompatiblePatternError::new_err(e)
        }
        XMLGeneratorError::EncodingError => {
            XSDEncodingError::new_err("No encoding specified in XSD")
        }
        XMLGeneratorError::UnimplementedFeature(e) => ImplementationError::new_err(e),
        XMLGeneratorError::InvalidXSDNameError(e) => InvalidXSDNameError::new_err(e),
        XMLGeneratorError::LineEndingsError(e) => LineEndingsError::new_err(e),
    }
}

fn handle_input(input_string: String) -> PyResult<PathBuf> {
    let path = PathBuf::from(input_string);

    match path.try_exists() {
        Ok(exists) => {
            if exists {
                Ok(path)
            } else {
                Err(InvalidPathError::new_err("Input path does not exist"))
            }
        }
        Err(err) => Err(InvalidPathError::new_err(err)),
    }
}

fn handle_panic(error: Box<dyn Any>) -> PyErr {
    if let Some(s) = error.downcast_ref::<&str>() {
        let msg = s.to_string();
        PyRuntimeError::new_err(msg)
    } else if let Some(s) = error.downcast_ref::<String>() {
        let msg = format!("XMLGenerator panic error: {}", s);
        PyRuntimeError::new_err(msg)
    } else {
        PyRuntimeError::new_err("XMLGenerator: unknown error")
    }
}

fn handle_result<T>(result: Result<T, XMLGeneratorError>) -> PyResult<T> {
    match result {
        Ok(output) => Ok(output),
        Err(error) => Err(handle_error(error)),
    }
}

/// Get the version of the :rust:crate:`xmlgenerator` Rust crate
///
/// Returns
/// -------
/// str
///    The version of the Rust crate
///
#[pyfunction]
fn version() -> String {
    format!("{}", env!("CARGO_PKG_VERSION"))
}

/// The main class which manages the XML generator
///
/// A single object should be used for all required validations and generations.
///
/// Example
/// -------
///
/// .. code-block:: python
///
///     from pyxmlgenerator import XMLGenerator
///
///     generator = XMLGenerator()
///
///     schema = "/path/to/schema.xsd"
///
///     # Validates the XSD file and generates an XML instance
///     xml_string = generator.generate(schema)
///
///     print(xml_string)
///
/// This class is a wrapper the :rust:struct:`xmlgenerator::XMLGenerator` Rust struct, which uses `libxml2`_ for validation via `Rust bindings`_.
/// The original ``libxml2`` library is written in C and includes a global initialiser/deinitialiser that should not be initialised more than once at a time.
/// The limitation is that only one instance of the class should be created at any one time
///
/// .. _libxml2: https://gitlab.gnome.org/GNOME/libxml2
/// .. _Rust bindings: https://crates.io/crates/libxml2-rs
///
#[pyclass(name = "XMLGenerator")]
pub struct PyXMLGenerator {
    inner: XMLGenerator,
}

#[pymethods]
impl PyXMLGenerator {
    #[new]
    fn new() -> PyResult<Self> {
        match panic::catch_unwind(XMLGenerator::new) {
            Ok(generator) => Ok(PyXMLGenerator { inner: generator }),
            Err(error) => Err(handle_panic(error)),
        }
    }

    /// XSD file validator
    /// 
    /// This method wraps the :rust:fn:`xmlgenerator::XMLGenerator::validate` method
    ///
    /// Arguments
    /// ---------
    /// filepath: str
    ///     The path to the XSD file to be validated
    ///
    /// Raises
    /// ------
    /// RuntimeError
    ///     If Rust code panics
    /// InvalidPathError
    ///     If ``filepath`` is invalid
    /// XSDValidatorError
    ///     If XSDValidator returns an error
    /// InvalidXSDError
    ///     If recursive elements are found
    ///
    /// Example
    /// -------
    ///
    /// .. code-block:: python
    ///
    ///     from pyxmlgenerator import XMLGenerator
    ///
    ///     generator = XMLGenerator()
    ///
    ///     schema = "/path/to/schema.xsd"
    ///
    ///     generator.validate(schema)
    ///
    /// The validator uses the :rust:struct:`xsdvalidator::XSDValidator` Rust struct.
    /// This struct wraps the `libxml2`_ C library via Rust bindings. 
    /// An XSD file is invalid if this library finds it to be invalid.
    /// The only additional checks track whether there is an infinite loop in the schema.
    ///
    /// .. _libxml2: https://gitlab.gnome.org/GNOME/libxml2
    ///
    fn validate(&self, filepath: String) -> PyResult<()> {
        let path_buf = handle_input(filepath)?;
        match panic::catch_unwind(|| self.inner.validate(&path_buf)) {
            Ok(result) => handle_result(result),
            Err(error) => Err(handle_panic(error)),
        }
    }

    /// XML instance generator
    ///
    /// This method  wraps the :rust:fn:`xmlgenerator::XMLGenerator::generate` method.
    /// 
    /// Arguments
    /// ---------
    /// filepath: str
    ///     The path to the XSD file to be validated
    /// seed : int | None = None
    ///     Seed the random number generator (default: None)
    ///
    /// Raises
    /// ------
    /// RuntimeError
    ///     Rust code panics
    /// InvalidPathError
    ///     Invalid ``filepath``
    /// XSDValidatorError
    ///     XSDValidator returns an error
    /// DataTypeInformationError
    ///     Unable to find any information for a custom type
    /// DataTypeNotFoundError
    ///     Generator cannot find a matching datatype for an attribute
    /// XSDParserError
    ///     The ``xsd-parser`` crate throws an error
    /// DataTypesFormatError
    ///     Data type is in an invalid format
    /// XMLBuilderError
    ///     The ``xml-builder`` crate encounters an error
    /// InvalidXSDVersionError
    ///     XSD has an XML other than 1.0 or 1.1
    /// InfiniteRecursionError
    ///     Infinite recursive element found
    /// NoElementsError
    ///     XSD does not contain any elements
    /// InvalidXSDError
    ///     XSD is invalid
    /// NoIndependentElementsError
    ///     XSD does not contain any root elements
    /// MultipleXSDRootsError
    ///     XSD contains multiple root elements
    /// TypeGenerationError
    ///     Generator is unable to generate a specific type
    /// RegexError
    ///     Regular expression error (XSD pattern or Rust)
    /// IncompatiblePatternError
    ///     Type is constrained to match two incompatible Regex patterns
    /// XSDEncodingError
    ///     XSD uses an unsupported encoding
    /// ImplementationError
    ///     Unimplemented feature
    /// InvalidXSDNameError
    ///     XSD contains an invalid name
    /// LineEndingsError
    ///     Regex contains invalid line endings
    ///
    /// Example
    /// -------
    ///
    /// .. code-block:: python
    ///
    ///     from pyxmlgenerator import XMLGenerator
    ///
    ///     generator = XMLGenerator()
    ///
    ///     schema = "/path/to/schema.xsd"
    ///
    ///     xml_string = generator.generate(schema)
    ///
    ///     print(xml_string)
    ///
    /// Not all features of the XML schema specification are implemented.
    /// Unimplemented features will return an error.
    ///
    #[pyo3(signature = (filepath, seed = None))]
    fn generate(&self, filepath: String, seed: Option<u64>) -> PyResult<String> {
        let path_buf = handle_input(filepath)?;

        match panic::catch_unwind(|| self.inner.generate(&path_buf, seed)) {
            Ok(result) => handle_result(result),
            Err(error) => Err(handle_panic(error)),
        }
    }
}

/// A Python package for generating XML instances from an input XML Schema (XSD)
///
/// This package may be used to read an XML Schema (XSD) file, validate it and use it to generate an XML instance that follows the input schema.
/// The package provides a class :py:class:`XMLGenerator` which may be used to validate an XSD or use it to generate a new instance.
/// The package is a Python wrapper for the :rust:crate:`xmlgenerator` Rust crate.
#[pymodule]
fn pyxmlgenerator(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyXMLGenerator>()?;

    m.add("InvalidPathError", _py.get_type::<InvalidPathError>())?;
    m.add("XSDValidatorError", _py.get_type::<XSDValidatorError>())?;
    m.add(
        "DataTypeInformationError",
        _py.get_type::<DataTypeInformationError>(),
    )?;
    m.add(
        "DataTypeNotFoundError",
        _py.get_type::<DataTypeNotFoundError>(),
    )?;
    m.add("XSDParserError", _py.get_type::<XSDParserError>())?;
    m.add(
        "DataTypesFormatError",
        _py.get_type::<DataTypesFormatError>(),
    )?;
    m.add("XMLBuilderError", _py.get_type::<XMLBuilderError>())?;
    m.add(
        "InvalidXSDVersionError",
        _py.get_type::<InvalidXSDVersionError>(),
    )?;
    m.add(
        "InfiniteRecursionError",
        _py.get_type::<InfiniteRecursionError>(),
    )?;
    m.add("NoElementsError", _py.get_type::<NoElementsError>())?;
    m.add("InvalidXSDError", _py.get_type::<InvalidXSDError>())?;
    m.add(
        "NoIndependentElementsError",
        _py.get_type::<NoIndependentElementsError>(),
    )?;
    m.add(
        "MultipleXSDRootsError",
        _py.get_type::<MultipleXSDRootsError>(),
    )?;
    m.add("TypeGenerationError", _py.get_type::<TypeGenerationError>())?;
    m.add("ImplementationError", _py.get_type::<ImplementationError>())?;
    m.add("RegexError", _py.get_type::<RegexError>())?;
    m.add(
        "IncompatiblePatternError",
        _py.get_type::<IncompatiblePatternError>(),
    )?;
    m.add("XSDEncodingError", _py.get_type::<XSDEncodingError>())?;
    m.add("InvalidXSDNameError", _py.get_type::<InvalidXSDNameError>())?;
    m.add("LineEndingsError", _py.get_type::<LineEndingsError>())?;
    m.add_function(wrap_pyfunction!(version, m)?)?;

    Ok(())
}
