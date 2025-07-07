// Auto-generated Python bindings for nucleation.
// This file is generated from api_definition.rs - DO NOT EDIT MANUALLY!

#![cfg(feature = "python")]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyBytes};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::{
    UniversalSchematic,
    BlockState,
    formats::{litematic, schematic},
    print_utils::{format_schematic, format_json_schematic},
};

#[pyclass(name = "BlockState")]
#[derive(Clone)]
pub struct PyBlockState {
    pub(crate) inner: BlockState,
}

#[pymethods]
impl PyBlockState {
    #[new]
    fn new(name: String) -> Self {
        Self {
            inner: BlockState::new(name),
        }
    }

    pub fn with_property(&mut self, , key: String, value: String) -> &PyBlockState {
        let new_inner = self.inner.clone().with_property(key, value);
        Self { inner: new_inner }
    }

    #[getter]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    pub fn properties(&self) -> HashMap<String, String> {
        self.inner.properties.clone()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("<BlockState '{}>'>", self.inner.to_string())
    }
}

#[pyclass(name = "Schematic")]
pub struct PySchematic {
    pub(crate) inner: UniversalSchematic,
}

#[pymethods]
impl PySchematic {
    #[new]
    fn new(name: Option<String>) -> Self {
        Self {
            inner: UniversalSchematic::new(name.unwrap_or_else(|| "Default".to_string())),
        }
    }

    pub fn from_data(&mut self, , data: &[u8]) -> PyResult<()> {
        if litematic::is_litematic(data) {
            self.inner = litematic::from_litematic(data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        } else if schematic::is_schematic(data) {
            self.inner = schematic::from_schematic(data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Unknown format"));
        }
        Ok(())
    }

    pub fn from_litematic(&mut self, , data: &[u8]) -> PyResult<()> {
        self.inner = litematic::from_litematic(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn to_litematic(&mut self) -> PyResult<&[u8]> {
        let bytes = litematic::to_litematic(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    pub fn from_schematic(&mut self, , data: &[u8]) -> PyResult<()> {
        self.inner = schematic::from_schematic(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(())
    }

    pub fn to_schematic(&mut self) -> PyResult<&[u8]> {
        let bytes = schematic::to_schematic(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(PyBytes::new(py, &bytes).into())
    }

    pub fn set_block(&mut self, , x: i32, y: i32, z: i32, block_name: String) {
        self.inner.set_block(x, y, z, BlockState::new(block_name.to_string()));
    }

    pub fn set_block_with_properties(&mut self, , x: i32, y: i32, z: i32, block_name: String, properties: HashMap<String, String>) {
        let block_state = BlockState {
            name: block_name.to_string(),
            properties: properties,
        };
        self.inner.set_block(x, y, z, block_state);
    }

    pub fn get_block(&mut self, , x: i32, y: i32, z: i32) -> Option<&PyBlockState> {
        self.inner.get_block(x, y, z).cloned().map(|bs| PySchematic { inner: bs })
    }

    #[getter]
    pub fn dimensions(&self) -> Vec<i32> {
        let (x, y, z) = self.inner.get_dimensions(); vec![x, y, z]
    }

    #[getter]
    pub fn block_count(&self) -> i32 {
        self.inner.total_blocks()
    }

    #[getter]
    pub fn volume(&self) -> i32 {
        self.inner.total_volume()
    }

    #[getter]
    pub fn region_names(&self) -> Vec<String> {
        self.inner.get_region_names()
    }

    fn __str__(&self) -> String {
        format_schematic(&self.inner)
    }

    fn __repr__(&self) -> String {
        format!("<{} '{}, {} blocks'", "Schematic",
                self.inner.metadata.name.as_ref().unwrap_or(&"Unnamed".to_string()),
                self.inner.total_blocks())
    }
}

#[pyfunction]
fn load_schematic(path: String) -> PyResult<&PySchematic> {
    let data = fs::read(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    let mut sch = PySchematic::new(Some(
        Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unnamed")
            .to_owned(),
    ));
    sch.from_data(&data)?;
    Ok(sch)
}

#[pyfunction]
#[pyo3(signature = (schematic, path, format = "auto"))]
fn save_schematic(schematic: &PySchematic, path: String, format: String) -> PyResult<()> {
    let py_bytes = match format {
        "litematic" => schematic.to_litematic(py)?,
        "schematic" => schematic.to_schematic(py)?,
        "auto" => {
            if path.ends_with(".litematic") {
                schematic.to_litematic(py)?
            } else {
                schematic.to_schematic(py)?
            }
        }
        other => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Unknown format '{}'", other)))
    };

    let bytes_obj = py_bytes.bind(py).downcast::<PyBytes>()?;
    let bytes = bytes_obj.as_bytes();

    fs::write(path, bytes)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    Ok(())
}

#[pyfunction]
fn debug_schematic(schematic: &PySchematic) -> String {
    format!("{}\n{}", schematic.debug_info(), format_schematic(&schematic.inner))
}

#[pyfunction]
fn debug_json_schematic(schematic: &PySchematic) -> String {
    format!("{}\n{}", schematic.debug_info(), format_json_schematic(&schematic.inner))
}

#[pymodule]
fn nucleation(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBlockState>()?;
    m.add_class::<PySchematic>()?;
    m.add_function(wrap_pyfunction!(load_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(save_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(debug_schematic, m)?)?;
    m.add_function(wrap_pyfunction!(debug_json_schematic, m)?)?;
    Ok(())
}
