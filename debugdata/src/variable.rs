use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use serde::Serialize;


#[derive(Clone, Serialize)]
pub struct DebugVariable {
    pub name: String,
    pub type_: String,
    pub value: String,
    pub mutable: bool,
    pub const_: bool, // only function arguments, const var definitions NOT included
    pub line_start: u32,
    pub line_end: u32,
    //pub declaration: VariableDeclaration,
    //pub references: Vec<&'a Expression<'a>>, // all Expression::VariableRef or panic
    //pub assignments: Vec<&'a Statement<'a>>, // all Statement::Assign or panic -- must be 1 if not mutable, or 0 if declaration == input | parameter
}



impl<'a> fmt::Display for DebugVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DebugFunction")
    }
}

impl<'a> fmt::Debug for DebugVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl<'a> PartialEq for DebugVariable {
    fn eq(&self, other: &DebugVariable) -> bool {
        true
    }
}
