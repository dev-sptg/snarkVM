use std::fmt;
use serde::Serialize;
use crate::DebugCircuit;

#[derive(Clone, Serialize)]
pub enum DebugVariableType {
    Integer,
    Circuit,
    Array,
    Boolean,
}

#[derive(Clone, Serialize)]
pub struct DebugSomeType {
    pub value: String,
}

impl<'a> fmt::Display for DebugSomeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DebugSomeType")
    }
}

impl<'a> fmt::Debug for DebugSomeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl<'a> PartialEq for DebugSomeType {
    fn eq(&self, _other: &DebugSomeType) -> bool {
        true
    }
}




#[derive(Clone, Serialize)]
pub struct DebugVariable {
    pub name: String,
    pub type_: DebugVariableType,
    pub value: String,
    pub circuit_id: u32,
    pub mutable: bool,
    pub const_: bool, // only function arguments, const var definitions NOT included
    pub line_start: u32,
    pub line_end: u32,
    pub sub_variables:Vec<DebugVariable>

}



impl<'a> fmt::Display for DebugVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DebugVariable")
    }
}

impl<'a> fmt::Debug for DebugVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl<'a> PartialEq for DebugVariable {
    fn eq(&self, _other: &DebugVariable) -> bool {
        true
    }
}
