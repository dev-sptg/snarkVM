use std::fmt;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct DebugInstruction {
    pub self_var_id: u32,
    pub line_start: u32,
    pub line_end: u32,
}



impl<'a> fmt::Display for DebugInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DebugFunction")
    }
}

impl<'a> fmt::Debug for DebugInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl<'a> PartialEq for DebugInstruction {
    fn eq(&self, _other: &DebugInstruction) -> bool {
        true
    }
}
