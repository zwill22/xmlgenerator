use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::any::Any;
use std::collections::HashSet;
use std::panic;
use std::path::PathBuf;
use xsdtestdata::get_test_data;

fn handle_panic(error: Box<dyn Any>) -> PyErr {
    if let Some(s) = error.downcast_ref::<&str>() {
        let msg = format!("{}", s);
        PyRuntimeError::new_err(msg)
    } else if let Some(s) = error.downcast_ref::<String>() {
        let msg = format!("{}", s);
        PyRuntimeError::new_err(msg)
    } else {
        PyRuntimeError::new_err("XMLGenerator: unknown error")
    }
}

#[pyfunction]
fn get_xsd_test_data(
    database_dir: PathBuf,
    archive_path: PathBuf,
    include_extra_files: bool,
) -> PyResult<HashSet<(PathBuf, bool)>> {
    let result =
        panic::catch_unwind(|| get_test_data(&database_dir, &archive_path, include_extra_files));

    match result {
        Ok(result) => Ok(result),
        Err(e) => Err(handle_panic(e)),
    }
}

#[pymodule]
fn pyxsdtestdata(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_xsd_test_data, m)?)?;

    Ok(())
}
