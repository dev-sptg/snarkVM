use std::fmt;
use std::fmt::Display;
use crate::variable::DebugVariable;
use indexmap::IndexMap;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct DebugFunction {
    pub name: String,
    pub variables: IndexMap<u32, DebugVariable>,
    pub line_start: u32,
    pub line_end: u32,
}

impl DebugFunction {
    pub fn new() -> Self {
        Self {
            name: String::from(""),
            variables: IndexMap::new(),
            line_start: 0,
            line_end: 0,
        }
    }

    pub fn add_variable(&mut self, id: u32, variable: DebugVariable) {
        self.variables.insert(id, variable);
    }

}


impl<'a> fmt::Display for DebugFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DebugFunction")
    }
}

impl<'a> fmt::Debug for DebugFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl<'a> PartialEq for DebugFunction {
    fn eq(&self, other: &DebugFunction) -> bool {
        true
    }
}
