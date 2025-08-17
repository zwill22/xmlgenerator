use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use xmlgenerator::generate_xml;
use xmlgenerator::error::XMLGeneratorError;
use xmlgenerator::error::XMLGeneratorError::XMLBuilderError;

fn generate_parser_error(err_string: String) -> PyErr {
    PyRuntimeError::new_err("XSD Parser encountered an error.\n".to_owned() + err_string.as_str())
}

fn generate_data_format_error(err_string: String) -> PyErr {
    PyRuntimeError::new_err("Input not in valid format:".to_owned() + err_string.as_str())
}

fn generate_data_type_error(err_string: String) -> PyErr {
    PyRuntimeError::new_err("Input contains invalid type:".to_owned() + err_string.as_str())
}

fn generate_xml_builder_error(err_string: String) -> PyErr {
    PyRuntimeError::new_err("XMLBuilder encountered an error\n".to_owned() + err_string.as_str())
}
fn get_error(error: XMLGeneratorError) -> PyErr {
    match error {
        XMLGeneratorError::XSDParserError(e) => generate_parser_error(e),
        XMLGeneratorError::DataTypesFormatError(e) => generate_data_format_error(e),
        XMLGeneratorError::DataTypeError(e) => generate_data_type_error(e),
        XMLBuilderError(e) => generate_xml_builder_error(e),
    }
}


/// Formats the sum of two numbers as string.
#[pyfunction]
fn generate(xsd_string: String) -> PyResult<String> {
    let result = generate_xml(&xsd_string);
    match result {
        Ok(xml_string) => Ok(xml_string),
        Err(e) => {
            let py_error = get_error(e);
            Err(py_error)
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyxmlgenerator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate, m)?)?;
    Ok(())
}
