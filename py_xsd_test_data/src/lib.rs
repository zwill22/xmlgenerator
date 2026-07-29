use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::any::Any;
use std::panic;
use std::path::PathBuf;
use xsdtestdata::{XSDData, XSDTestData, XSDTestDataIntoIterator};

fn handle_panic(error: Box<dyn Any>) -> PyErr {
    if let Some(s) = error.downcast_ref::<&str>() {
        let msg = s.to_string();
        PyRuntimeError::new_err(msg)
    } else if let Some(s) = error.downcast_ref::<String>() {
        let msg = s.to_string();
        PyRuntimeError::new_err(msg)
    } else {
        PyRuntimeError::new_err("XMLGenerator: unknown error")
    }
}

#[pyclass(name = "XSDData")]
pub struct PyXSDData {
    data: XSDData,
}

#[pymethods]
impl PyXSDData {
    pub fn key(&self) -> &str {
        self.data.get_key()
    }

    pub fn data_set(&self) -> &str {
        self.data.get_data_set()
    }

    pub fn path(&self) -> &PathBuf {
        self.data.get_path()
    }

    pub fn valid(&self) -> bool {
        self.data.is_valid()
    }
}

#[pyclass(name = "XSDTestDataIter")]
pub struct PyXSDTestDataIter {
    iter: XSDTestDataIntoIterator,
}

#[pymethods]
impl PyXSDTestDataIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyXSDData> {
        let data = slf.iter.next()?;

        Some(PyXSDData { data })
    }
}

#[pyclass(name = "XSDTestData")]
pub struct PyXSDTestData {
    inner: XSDTestData,
}

#[pymethods]
impl PyXSDTestData {
    #[new]
    fn new(database_dir: PathBuf, archive_path: PathBuf) -> PyResult<Self> {
        match panic::catch_unwind(|| XSDTestData::new(&database_dir, &archive_path)) {
            Ok(result) => Ok(PyXSDTestData { inner: result }),
            Err(error) => Err(handle_panic(error)),
        }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<PyXSDTestDataIter>> {
        let iter = PyXSDTestDataIter {
            iter: slf.inner.clone().into_iter(),
        };

        Py::new(slf.py(), iter)
    }

    fn get(&self, key: &str) -> Option<PyXSDData> {
        let data = self.inner.get(key)?;

        Some(PyXSDData { data: data.clone() })
    }
}

#[pyclass]
struct Iterator {
    inner: std::vec::IntoIter<usize>,
}

#[pymethods]
impl Iterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<usize> {
        slf.inner.next()
    }
}

#[pyclass]
struct Container {
    iter: Vec<usize>,
}

#[pymethods]
impl Container {
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<Iterator>> {
        let iter = Iterator {
            inner: slf.iter.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }
}

/// Return the version of the XMLGenerator Rust crate
///
/// Returns
/// -------
/// str
///    The version of the Rust crate
#[pyfunction]
fn version() -> String {
    format!("{}", env!("CARGO_PKG_VERSION"))
}


#[pymodule]
fn pyxsdtestdata(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyXSDTestData>()?;
    m.add_class::<PyXSDData>()?;

    m.add_function(wrap_pyfunction!(version, m)?)?;

    Ok(())
}
