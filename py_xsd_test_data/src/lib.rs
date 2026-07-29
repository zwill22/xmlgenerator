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

/// XSD Data Class containing information about a single XSD file
///
/// Includes data about an XSD file, including:
///
/// * Path to file
/// * Data set it belongs to
/// * Validity of XSD
///
/// No constructor is included, instances are created by the :py:class:`XSDTestData` class.
///
#[pyclass(name = "XSDData")]
pub struct PyXSDData {
    data: XSDData,
}

#[pymethods]
impl PyXSDData {
    /// Get the unique key associated with the XSD file
    ///
    /// Returns
    /// -------
    /// str
    ///     Unique key for the file in the test set
    ///
    pub fn key(&self) -> &str {
        self.data.get_key()
    }

    /// Get the data set to which the XSD file belongs
    ///
    /// Returns
    /// -------
    /// str
    ///     Data set name
    ///
    pub fn data_set(&self) -> &str {
        self.data.get_data_set()
    }

    /// Get the filepath of the XSD file
    ///
    /// Returns
    /// -------
    /// pathlib.Path
    ///     Path to XSD file
    ///
    pub fn path(&self) -> &PathBuf {
        self.data.get_path()
    }

    /// Whether the XSD file is listed as valid
    ///
    /// Returns
    /// -------
    /// bool
    ///     Validity of XSD
    ///
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

/// Class for managing XSD Test Data from the `xsdtests`_ database.
///  
/// The class manages the data in the `xsdtests`_ database.
/// When initialised, the class searches for the database at the :py:data:`database_dir`. If found, it builds the object using the contents of this directory.
///
/// If the :py:data:`database_dir`` does not exist, the class searches for an archive file at
/// :py:data:`archive_path`. If this is found, it extracts the contents to :py:data:`database_dir`.
/// Otherwise, it downloads the archive from the `xsdtests`_ repo.
///
/// .. _xsdtests: https://github.com/w3c/xsdtests
///
/// Arguments
/// ---------
/// database_dir: pathlib.Path
///     Path to the (current or target) directory for the database.
/// archive_path: pathlib.Path
///     Path to the zip file containing the database
///
/// Returns
/// -------
/// XSDTestData
///     Data about all XSD files in the database
///
/// Raises
/// ------
/// RuntimeError
///     Error while setting up the database
///
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

    /// Get the :py:class:`XSDData` for the XSD file with :py:data:`key`
    ///
    /// Arguments
    /// ---------
    /// key: str
    ///     Key for the required XSD file
    ///
    /// Returns
    /// -------
    /// XSDData | None
    ///     If the key is present, returns the :py:class:`XSDData` for the specified XSD.
    ///     Otherwise returns `None`.
    /// 
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

/// Module for managing XSD test data
/// 
/// This module provides methods for managing test data from the `xsdtests`_ database.
/// The :py:class:`XSDTestData` class is used to initialise and manage the datasets.
///
/// .. _xsdtests: https://github.com/w3c/xsdtests
#[pymodule]
fn pyxsdtestdata(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyXSDTestData>()?;
    m.add_class::<PyXSDData>()?;

    m.add_function(wrap_pyfunction!(version, m)?)?;

    Ok(())
}
