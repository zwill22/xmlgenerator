use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::any::Any;
use std::panic;
use std::path::PathBuf;
use xsdtestdata::{ XsdData, XsdTestData, XsdTestDataIntoIterator };

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
pub struct PyXsdData {
    data: XsdData,
}

#[pymethods]
impl PyXsdData {
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
pub struct PyXsdTestDataIter {
    iter: XsdTestDataIntoIterator,
}

#[pymethods]
impl PyXsdTestDataIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyXsdData> {
        let data = slf.iter.next()?;

        Some(PyXsdData { data })
    }
}

#[pyclass(name = "XSDTestData")]
pub struct PyXsdTestData {
    inner: XsdTestData,
}

#[pymethods]
impl PyXsdTestData {
    #[new]
    fn new(database_dir: PathBuf, archive_path: PathBuf) -> PyResult<Self> {
        match panic::catch_unwind(|| XsdTestData::new(&database_dir, &archive_path)) {
            Ok(result) => Ok(PyXsdTestData { inner: result }),
            Err(error) => Err(handle_panic(error)),
        }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<PyXsdTestDataIter>> {
        let iter = PyXsdTestDataIter {
            iter: slf.inner.clone().into_iter(),
        };

        Py::new(slf.py(), iter)
    }

    fn get(&self, key: &str) -> Option<PyXsdData> {
        let data = self.inner.get(key)?;

        Some(PyXsdData { data: data.clone() })
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

#[pymodule]
fn pyxsdtestdata(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyXsdTestData>()?;
    m.add_class::<PyXsdData>()?;

    Ok(())
}
