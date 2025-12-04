use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyRuntimeError};
use pyo3::prelude::*;
use std::any::Any;
use std::panic;
use std::path::PathBuf;
use xmlgenerator::XMLGenerator;
use xmlgenerator::error::XMLGeneratorError;

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
create_exception!(pyxmlgenerator, TypeGenerationError, PyException);
create_exception!(pyxmlgenerator, ImplementationError, PyException);
create_exception!(pyxmlgenerator, RegexError, PyException);

fn handle_error(error: XMLGeneratorError) -> PyErr {
    match error {
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
        XMLGeneratorError::TypeGenerationError(e) => TypeGenerationError::new_err(e),
        XMLGeneratorError::RegexError(e) => RegexError::new_err(e),
        XMLGeneratorError::UnimplementedFeature(e) => ImplementationError::new_err(e),
    }
}

fn handle_panic(error: Box<dyn Any>) -> PyErr {
    if let Some(s) = error.downcast_ref::<&str>() {
        let msg = format!("{}", s);
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

#[pyclass(name = "XMLGenerator")]
pub struct PyXMLGenerator {
    inner: XMLGenerator,
}

#[pymethods]
impl PyXMLGenerator {
    #[new]
    fn new() -> PyResult<Self> {
        match panic::catch_unwind(|| XMLGenerator::new()) {
            Ok(generator) => Ok(PyXMLGenerator { inner: generator }),
            Err(error) => Err(handle_panic(error)),
        }
    }

    fn validate(&self, filepath: String) -> PyResult<()> {
        let path_buf = PathBuf::from(filepath);
        match panic::catch_unwind(|| self.inner.validate(&path_buf)) {
            Ok(result) => handle_result(result),
            Err(error) => Err(handle_panic(error)),
        }
    }

    fn generate(&self, filepath: String) -> PyResult<String> {
        let path_buf = PathBuf::from(filepath);
        match panic::catch_unwind(|| self.inner.generate(&path_buf)) {
            Ok(result) => handle_result(result),
            Err(error) => Err(handle_panic(error)),
        }
    }
}

#[pymodule]
fn pyxmlgenerator(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyXMLGenerator>()?;
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
    m.add("TypeGenerationError", _py.get_type::<TypeGenerationError>())?;
    m.add("ImplementationError", _py.get_type::<ImplementationError>())?;
    Ok(())
}
