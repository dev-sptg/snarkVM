use crate::function::DebugFunction;
use crate::variable::DebugVariable;
use indexmap::IndexMap;
use serde::Serialize;


use std::fmt;
use std::fmt::Display;

#[derive(Clone, Serialize)]
pub enum DebugItem {
    Variable(DebugVariable),
    Function(DebugFunction)
}

#[derive(Clone, Serialize)]
pub struct DebugData{
    pub data: IndexMap<u32, DebugItem>,
}

impl DebugData {
    pub fn new() -> Self {
        Self {
            data: IndexMap::new()
        }
    }
}

impl<'a> fmt::Display for DebugData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DebugData")
    }
}

impl<'a> fmt::Debug for DebugData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl<'a> PartialEq for DebugData {
    fn eq(&self, other: &DebugData) -> bool {
        true
    }
}