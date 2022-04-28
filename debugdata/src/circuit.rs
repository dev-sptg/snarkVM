use std::fmt;
use serde::Serialize;


#[derive(Clone, Serialize)]
pub struct DebugCircuit {
    pub name: String,
    pub members: Vec<u32>,
    pub functions: Vec<u32>,
    pub line_start: u32,
    pub line_end: u32,
}



impl<'a> fmt::Display for DebugCircuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DebugFunction")
    }
}

impl<'a> fmt::Debug for DebugCircuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl<'a> PartialEq for DebugCircuit {
    fn eq(&self, _other: &DebugCircuit) -> bool {
        true
    }
}
