use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyRuntimeError};
use pyo3::prelude::*;
use std::any::Any;
use std::panic;
use std::path::PathBuf;
use xmlgenerator::error::XMLGeneratorError;
use xmlgenerator::XMLGenerator;

create_exception!(pyxmlgenerator, InvalidPathError, PyException);
create_exception!(pyxmlgenerator, XSDValidatorError, PyException);
create_exception!(pyxmlgenerator, DataTypeInformationError, PyException);
create_exception!(pyxmlgenerator, DataTypeNotFoundError, PyException);
create_exception!(pyxmlgenerator, XSDParserError, PyException);
create_exception!(pyxmlgenerator, DataTypesFormatError, PyException);
create_exception!(pyxmlgenerator, XMLBuilderError, PyException);
create_exception!(pyxmlgenerator, InvalidXSDVersionError, PyException);
create_exception!(pyxmlgenerator, InfiniteRecursionError, PyException);
create_exception!(pyxmlgenerator, NoElementsError, PyException);
create_exception!(pyxmlgenerator, InvalidXSDError, PyException);
create_exception!(pyxmlgenerator, NoIndependentElementsError, PyException);
create_exception!(pyxmlgenerator, MultipleXSDRootsError, PyException);
create_exception!(pyxmlgenerator, TypeGenerationError, PyException);
create_exception!(pyxmlgenerator, ImplementationError, PyException);
create_exception!(pyxmlgenerator, RegexError, PyException);
create_exception!(pyxmlgenerator, IncompatiblePatternError, PyException);
create_exception!(pyxmlgenerator, InvalidXSDNameError, PyException);
create_exception!(pyxmlgenerator, LineEndingsError, PyException);
create_exception!(pyxmlgenerator, XSDEncodingError, PyException);

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
        ImplementationError::new_err(msg)
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

/// Return the version of the XMLGenerator Rust crate
///
/// Returns
/// -------
/// str
///    The version of the Rust crate
///
/// Raises
/// ------
/// None
///   This function does not raise any exceptions
///
#[pyfunction]
fn version() -> String {
    format!("{}", env!("CARGO_PKG_VERSION"))
}

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

    fn validate(&self, filepath: String) -> PyResult<()> {
        let path_buf = handle_input(filepath)?;
        match panic::catch_unwind(|| self.inner.validate(&path_buf)) {
            Ok(result) => handle_result(result),
            Err(error) => Err(handle_panic(error)),
        }
    }

    #[pyo3(signature = (filepath, seed = None))]
    fn generate(&self, filepath: String, seed: Option<u64>) -> PyResult<String> {
        let path_buf = handle_input(filepath)?;

        match panic::catch_unwind(|| self.inner.generate(&path_buf, seed)) {
            Ok(result) => handle_result(result),
            Err(error) => Err(handle_panic(error)),
        }
    }
}

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
