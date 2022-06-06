use std::fmt;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub enum DebugVariableType {
    Empty,
    Group,
    Char,
    Integer,
    Circuit,
    Array,
    Boolean,
    Field,
    String,
    Tuple
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
    pub is_argument: bool,
    pub const_: bool, // only function arguments, const var definitions NOT included
    pub line_start: u32,
    pub line_end: u32,
    pub sub_variables:Vec<DebugVariable>

}

impl DebugVariable {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            type_: DebugVariableType::Empty,
            value: "".to_string(),
            circuit_id: 0,
            mutable: false,
            is_argument: false,
            const_: false,
            line_start: 0,
            line_end: 0,
            sub_variables: vec![]
        }
    }

    pub fn new_some_variable(type_: DebugVariableType, name: String, value: String, line_start: u32, line_end: u32) -> Self {
        Self {
            name: name,
            type_: type_,
            value: value,
            circuit_id: 0,
            mutable: false,
            is_argument: false,
            const_: false,
            line_start: line_start,
            line_end: line_end,
            sub_variables: vec![]
        }
    }

    /*pub fn new_array(name: String, value: String, line_start: u32, line_end: u32) -> Self {
        Self {
            name: name,
            type_: DebugVariableType::Array,
            value: value,
            circuit_id: 0,
            mutable: false,
            is_argument: false,
            const_: false,
            line_start: line_start,
            line_end: line_end,
            sub_variables: vec![]
        }
    }*/

    pub fn new_circuit(name: String, value: String, circuit_id: u32, line_start: u32, line_end: u32) -> Self {
        Self {
            name: name,
            type_: DebugVariableType::Circuit,
            value: value,
            circuit_id: circuit_id,
            mutable: false,
            is_argument: false,
            const_: false,
            line_start: line_start,
            line_end: line_end,
            sub_variables: vec![]
        }
    }


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
