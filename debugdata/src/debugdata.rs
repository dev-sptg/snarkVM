use crate::function::DebugFunction;
use crate::variable::DebugVariable;
use indexmap::IndexMap;
use serde::Serialize;


use std::fmt;
use crate::DebugCircuit;
use crate::instruction::DebugInstruction;

#[derive(Clone, Serialize)]
pub enum DebugItem {
    Variable(DebugVariable),
    Function(DebugFunction),
    //Circuit(DebugCircuit)
}

#[derive(Clone, Serialize)]
pub struct DebugData{
    pub data: IndexMap<u32, DebugItem>,
    pub stack: Vec<DebugFunction>,
    pub call_dept: u32,
    pub debug: bool,
    pub debug_port: u32,
    pub debug_variable: DebugVariable,
    pub functions: IndexMap<u32, DebugFunction>,
    pub variables: IndexMap<u32, DebugVariable>,
    pub circuits: IndexMap<u32, DebugCircuit>,
    pub last_circuit_id: u32,
}

impl DebugData {
    pub fn new(debug: bool, debug_port: Option<u32>) -> Self {
        let port = match debug_port {
            Some( port) => port,
            None => 50001
        };

        Self {
            data: IndexMap::new(),
            stack: Vec::new(),
            call_dept: 0,
            debug: debug,
            debug_port: port,
            debug_variable: DebugVariable::new(),
            functions: IndexMap::new(),
            variables: IndexMap::new(),
            circuits: IndexMap::new(),
            last_circuit_id: std::u32::MAX
        }
    }

    pub fn add_variable(&mut self, id: u32, variable: DebugVariable) {
        match self.variables.get(&id) {
            Some(_item) => {}
            None => {
                self.variables.insert(id, variable);
            }
        }
    }

    pub fn add_function(&mut self, id: u32, function: DebugFunction) {
        self.functions.insert(id, function);
    }

    pub fn get_function(&mut self, id: u32) -> Option<&mut DebugFunction> {
        let func = self.functions.get_mut(&id);
        func
    }

    pub fn add_variable_to_function(&mut self, function_id: u32, variable_id: u32) {
        match self.functions.get_mut(&function_id) {
            Some(item) => {
                item.variables.push(variable_id);
            }
            None => {}
        }
    }


    pub fn insert_instruction(&mut self, function_index: u32, instruction_index: u32, instruction: DebugInstruction) {
        match self.functions.get_mut(&function_index)  {
            Some(func) => {
                func.instructions.insert(instruction_index, instruction);
            }
            None => {}
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
    fn eq(&self, _other: &DebugData) -> bool {
        true
    }
}