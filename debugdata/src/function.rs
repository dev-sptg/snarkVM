use std::fmt;
use indexmap::IndexMap;
use serde::Serialize;
use crate::{DebugInstruction, DebugVariable};

#[derive(Clone, Serialize)]
pub struct DebugFunction {
    pub name: String,
    pub file_path: String,
    pub self_circuit_id: u32,
    //pub variables: IndexMap<u32, DebugVariable>,
    pub variables:Vec<u32>,
    pub instructions: IndexMap<u32, DebugInstruction>,
    pub arguments:Vec<u32>,
    pub current_line: u32,
    pub line_start: u32,
    pub line_end: u32,

}

impl DebugFunction {
    pub fn new() -> Self {
        Self {
            name: String::from(""),
            file_path: String::from(""),
            self_circuit_id: 0,
            variables: Vec::new(),
            instructions: IndexMap::new(),
            arguments: Vec::new(),
            current_line: 0,
            line_start: 0,
            line_end: 0,
        }
    }

    pub fn add_variable(&mut self, id: u32) {
        self.variables.push(id);

        /*match self.variables.get(&id) {
            Some(_variable) => {

            }
            None =>{
                self.variables.insert(id, variable);
            }
        }*/
    }

    pub fn get_variables_count(&self, is_argument: bool, variables: &IndexMap<u32, DebugVariable>) -> u32 {
        let mut count: u32 = 0;
        for val in self.variables.iter() {
            let item = variables.get(val);
            match item {
                Some(variable) => {
                    if is_argument == variable.is_argument {
                        count += 1;
                    }
                }
                None => {}
            }
        }
        count
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
    fn eq(&self, _other: &DebugFunction) -> bool {
        true
    }
}