//! Core logic for a Rubik's Cube simulator.
//! This crate provides the `Cube` struct for maintaining state and `Scramble` for parsing and applying moves.

use pyo3::prelude::*;

mod cube;
mod moves;
pub mod solvers;

pub use cube::Cube;
pub use moves::{MOVE_LIST, Scramble};
pub use solvers::kociemba;

#[pyclass(name = "SingleMove")]
pub struct PySingleMove {
    pub(crate) inner: crate::moves::SingleMove,
}

#[pymethods]
impl PySingleMove {
    #[new]
    fn new(move_str: &str) -> PyResult<Self> {
        crate::moves::SingleMove::new(move_str)
            .map(|mv| Self { inner: mv })
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid move string"))
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }
}

#[pyclass(name = "Scramble")]
pub struct PyScramble {
    pub(crate) inner: Scramble,
}

#[pymethods]
impl PyScramble {
    #[new]
    fn new(move_str: &str) -> Self {
        Self {
            inner: Scramble::new(move_str),
        }
    }

    #[staticmethod]
    fn random(size: usize) -> Self {
        Self {
            inner: Scramble::random(size),
        }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn __str__(&self) -> String {
        self.inner.to_str()
    }

    fn __repr__(&self) -> String {
        format!("Scramble('{}')", self.inner.to_str().trim())
    }
}

#[pyclass(name = "Cube")]
pub struct PyCube {
    inner: Cube,
}

#[pymethods]
impl PyCube {
    #[new]
    fn new() -> Self {
        Self { inner: Cube::new() }
    }

    #[staticmethod]
    fn new_random(size: usize) -> Self {
        Self {
            inner: Cube::new_random(size),
        }
    }

    #[staticmethod]
    fn new_with(scramble: &PyScramble) -> Self {
        Self {
            inner: Cube::new_with(&scramble.inner),
        }
    }

    #[staticmethod]
    fn new_from_minimal(representation: u128) -> Self {
        Self {
            inner: Cube::new_from_minimal(representation),
        }
    }

    fn is_solved(&self) -> bool {
        self.inner.is_solved()
    }

    fn apply(&mut self, scramble: &PyScramble) {
        self.inner.apply(&scramble.inner);
    }

    fn turn(&mut self, move_str: &str) {
        if let Some(mv) = crate::moves::SingleMove::new(move_str) {
            self.inner.turn(&mv);
        }
    }

    fn minimal_representation(&self) -> u128 {
        self.inner.minimal_representation()
    }

    fn net_map(&self) -> String {
        self.inner.net_map()
    }

    fn edges(&self) -> u64 {
        self.inner.edges()
    }

    fn corners(&self) -> u64 {
        self.inner.corners()
    }
}

#[pyfunction]
fn get_move_list() -> Vec<String> {
    crate::moves::MOVE_LIST
        .iter()
        .map(|m| m.to_string())
        .collect()
}

#[pymodule]
fn rustbik_cube(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySingleMove>()?;
    m.add_class::<PyScramble>()?;
    m.add_class::<PyCube>()?;
    m.add_function(wrap_pyfunction!(get_move_list, m)?)?;
    Ok(())
}
