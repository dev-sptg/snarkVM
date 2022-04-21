extern crate libloading as lib;
use snarkvm_ir::{InputData, Value};
use std::path::PathBuf;
use std::io;
use std::sync::{Arc, Condvar, mpsc, Mutex};
use std::ffi::{CString};
use std::os::raw::{c_char, c_int};
use std::ptr::{copy_nonoverlapping};
use std::env;
use std::sync::mpsc::{Receiver};
use std::thread;
use std::alloc::{alloc, dealloc, Layout};
use libloading::Library;
use snarkvm_debugdata::{DebugData, DebugFunction, DebugInstruction, DebugItem, DebugVariable, DebugVariableType};
use snarkvm_debugdata::DebugItem::{ Function, Variable};
use std::mem::size_of;
use std::slice::from_raw_parts;
use std::slice::from_raw_parts_mut;
use indexmap::IndexMap;
use libc::c_void;
use snarkvm_debugdata::DebugVariableType::Circuit;
use snarkvm_fields::PrimeField;
use snarkvm_ir::Value::Array;
use std::process;
use crate::{ConstrainedValue, GroupType};

#[repr(C)]
struct VariableExp {
    pub name: *mut c_char,
    pub type_: *mut c_char,
    pub value: *mut c_char,
    pub variables_reference: u32
}


#[repr(C)]
struct ScopeExp {
    pub name: *mut c_char,
    pub presentation_hint: *mut c_char,
    pub variables_reference: u32
}

#[repr(C)]
struct ScopesMapExp {
    pub scopes: *mut ScopeExp,
    pub count: i32
}


#[repr(C)]
struct StackFrameExp {
    pub id: i32,
    pub scopes_map: *mut ScopesMapExp,
    pub scopes_count: i32,

    pub line: i32,
    pub column: i32,
    pub name: *mut c_char,
}

#[repr(C)]
struct StackExp {
    pub stack: *mut StackFrameExp,
    pub stack_count: i32,
}

extern "C" {
    fn printf(fmt: *const c_char, ...) -> c_int;
    fn strlen(arr: *const c_char) -> usize;
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
}


type RunServer = fn(port: u32) -> i32;
type RegisterCallback = fn (target: *mut RustObject, cb: extern fn(target: *mut RustObject, *mut  c_char, i32));
type RegisterNextStep = fn (target: *mut RustObject, cb: extern fn(target: *mut RustObject));
type RegisterStepIn = fn (target: *const Debugger, cb: extern fn(target: *mut Debugger));
type RegisterGetStackCallback = fn (target: *const Debugger, cb: extern fn(target: *mut Debugger));
type RegisterTerminateDebug = fn (target: *const Debugger, cb: extern fn(target: *mut Debugger));
type AddStack = fn (stack: *mut StackExp);
type AddVariables = fn (variables_reference: u32, variables: *mut VariableExp, count: u32);
type NextStepResponse = fn ();

extern "C" fn callback(target: *mut RustObject, src_path: *mut  c_char, _sz: i32) {
    println!("Rust: I'm called from C");

    /*let path = CString::new("foo").expect("CString::new failed");
    let len = path.as_bytes_with_nul().len();
    let ptr = path.into_raw();
    unsafe {
        copy_nonoverlapping(ptr, src_path, len);

        let _ = CString::from_raw(ptr);
    }*/

    let path = unsafe {
        assert!(!target.is_null());
        &mut *target
    };

    let path = path.main_file_path.as_path().display().to_string();

    let path = CString::new(path).expect("CString::new failed");
    let len = path.as_bytes_with_nul().len();
    let ptr = path.into_raw();
    unsafe {
        copy_nonoverlapping(ptr, src_path, len);
        let _ = CString::from_raw(ptr);
    }
}

extern "C" fn next_step(target: *mut RustObject) {
    println!("Rust:next_step : I'm called from C");
    let robject = unsafe {
        assert!(!target.is_null());
        &mut *target
    };

    let (lock, cvar) = &*robject.mutex_pair;
    let mut started = lock.lock().unwrap();
    *started = true;
    cvar.notify_one();
}

extern "C" fn step_in(target: *mut Debugger) {
    println!("Rust:step_in : I'm called from C");
    let debugger = unsafe {
        assert!(!target.is_null());
        &mut *target
    };
    
    debugger.is_step_into = true;

    let (lock, cvar) = &*debugger.pair;
    let mut started = lock.lock().unwrap();
    *started = true;
    cvar.notify_one();
}

extern "C" fn get_stack_callback(target: *mut Debugger) {
    println!("Rust:get_stack_callback : I'm called from C");
    let debugger = unsafe {
        assert!(!target.is_null());
        &mut *target
    };
    debugger.send_stack_frame();
}





extern "C" fn terminate_debug(target: *mut Debugger) {
    println!("Rust:step_in : I'm called from C");
    let debugger = unsafe {
        assert!(!target.is_null());
        &mut *target
    };
    process::exit(0x0);
}

#[derive(Clone, Debug)]
struct StrackEvent {
    pub event_id: u32,
    pub debug_data: DebugData,
}


//#[derive(Clone, Debug)]

#[derive(Debug)]
pub struct Debugger {
    pair:Arc<(Mutex<bool>, Condvar)>,
    pub debug_data: DebugData,
    cur_stack_frameID: u32,
    cur_variables_referenceID: u32,
    cur_functionID: u32,
    tx: mpsc::Sender<StrackEvent>,
    rx: Arc<Mutex<Receiver<StrackEvent>>>,
    lib_main: Library,
    call_depth: u32,
    cur_program_call_depth: u32,
    pub is_step_into: bool
}

#[repr(C)]
struct RustObject {
    main_file_path: PathBuf,
    mutex_pair:Arc<(Mutex<bool>, Condvar)>,
}


impl Debugger {
    pub fn new(debug_data: DebugData) -> Self {
        let (tx, rx) = mpsc::channel();        
        let pathSo = Self::inner_main("debugger.dll").expect("Couldn't");
        let lib_main = lib::Library::new(pathSo).unwrap();
        Self {
            pair: Arc::new((Mutex::new(false), Condvar::new())),
            debug_data: debug_data,
            cur_stack_frameID: 200,
            cur_variables_referenceID: 300,
            cur_functionID: 0,
            tx: tx,
            rx: Arc::new(Mutex::new(rx)),
            lib_main: lib_main,
            call_depth: 1,
            cur_program_call_depth: 1,
            is_step_into: false
        }
    }

    fn inner_main(str: &str) -> io::Result<PathBuf> {
        let mut dir = env::current_exe()?;
        dir.pop();
        dir.push(str);
        Ok(dir)
    }

    pub fn wait_for_next_step(&mut self) {
        let (lock, cvar) = &*self.pair;
        let started = lock.lock().unwrap();
        cvar.wait(started);
    }

    pub fn update_position(&mut self, line_start: u32, line_end: u32) {
        match self.debug_data.stack.last_mut() {
            Some(func) => {
                func.line_start = line_start;
                func.line_end = line_end;
            }
            None =>{}
        }
    }

    pub fn set_self_reference(&mut self, self_circuit_id: u32) {
        match self.debug_data.stack.last_mut() {
            Some(func) => {
                match self.debug_data.variables.get_mut(&self_circuit_id) {
                    Some(variable) => {
                        match variable.type_ {
                            Circuit => {
                                func.self_circuit_id = self_circuit_id;
                            }
                            _=>{}
                        }
                    }
                    None =>{}
                }

            }
            None =>{}
        }
    }


    pub fn evaluate_instruction(&mut self, function_index: u32,  instruction_index: u32, ) {
        if self.call_depth == (self.debug_data.stack.len() as u32) && self.call_depth == self.cur_program_call_depth {

            match  self.debug_data.functions.get_mut(&function_index) {
                Some(func) => {
                    let line_start = func.line_start;
                    let line_end = func.line_end;

                    if self.is_step_into {
                        self.is_step_into = false;

                        self.update_position(line_start, line_end);

                        self.send_next_step_response();
                        self.wait_for_next_step();

                    }
                }
                None => {
                }
            };


            match self.debug_data.functions.get_mut(&function_index) {
                Some(func) => {
                    match func.instructions.get_mut(&instruction_index) {
                        Some(instruction) => {
                            let instruction_line_start = instruction.line_start;
                            let instruction_line_end = instruction.line_end;

                            self.update_position(instruction_line_start, instruction_line_end);
                            self.send_next_step_response();
                            self.wait_for_next_step();
                        }
                        None => {}
                    }

                }
                None =>{
                }
            }

            /*let func = self.debug_data.functions.get_mut(&function_index).expect("return in non-function");
            let instruction = func.instructions.get_mut(&instruction_index).expect("return in non-function");

            let line_start = func.line_start;
            let line_end = func.line_end;

            if self.is_step_into {
                self.is_step_into = false;

                self.update_position(line_start, line_end);
                let dept = self.get_debug_call_depth() + 1;
                self.set_debug_call_depth(dept);

                self.send_next_step_response();
                self.wait_for_next_step();

            }*/

            /*let instruction = match func.instructions.get_mut(&instruction_index) {
                Some(instruction) => {
                    instruction
                }
                None => {
                    return;
                }
            };*/


           /* match func.instructions.get_mut(&instruction_index) {
                Some(instruction) => {
                    //let instruction_line_start = instruction.line_start;
                    //let instruction_line_end = instruction.line_end;

                    //self.update_position(instruction_line_start, instruction_line_end);
                    //self.send_next_step_response();
                    //self.wait_for_next_step();
                }
                None => {}
            }*/

            /*
             match self.debug_data.functions.get(&function_index) {
                Some(func) => {
                    let f = func.clone();
                    let line_start = f.line_start;
                    let line_end = f.line_end;
                    if self.is_step_into {
                        self.is_step_into = false;

                        self.update_position(line_start, line_end);
                        let dept = self.get_debug_call_depth() + 1;
                        self.set_debug_call_depth(dept);

                        self.send_next_step_response();
                        self.wait_for_next_step();

                    }

                    match func.instructions.get_mut(&instruction_index) {
                        Some(instruction) => {
                            //let instruction_line_start = instruction.line_start;
                            //let instruction_line_end = instruction.line_end;

                            //self.update_position(instruction_line_start, instruction_line_end);
                            //self.send_next_step_response();
                            //self.wait_for_next_step();
                        }
                        None => {}
                    }

                }
                None =>{
                }
            }
*/

            /*match self.debug_data.functions.get_mut(&function_index) {
                Some(func) => {
                    match func.instructions.get_mut(&instruction_index) {
                        Some(instruction) => {
                            //self.update_position(instruction.line_start, instruction.line_end);
                            self.send_next_step_response();
                            self.wait_for_next_step();

                            if self.is_step_into {
                                self.is_step_into = false;
                                let line_start = func.line_start;
                                let line_end = func.line_end;


                                //self.update_position(0, 0);
                                //self.update_position(func.line_start, func.line_end);

                                let dept = self.get_debug_call_depth() + 1;
                                self.set_debug_call_depth(dept);

                                self.send_next_step_response();
                                self.wait_for_next_step();
                            }
                        }
                        None => {}
                    }
                }
                None => {}
            }*/

            /*match self.debug_data.instructions.get(&id) {
                Some(item) => {
                    self.update_position(item.line_start, item.line_end);
                    self.send_next_step_response();
                    self.wait_for_next_step();

                    if self.is_step_into {
                        self.is_step_into = false;
                        match self.debug_data.functions.get(&index)  {
                            Some(func) => {
                                self.update_position(func.line_start, func.line_end);

                                let dept = self.get_debug_call_depth() + 1;
                                self.set_debug_call_depth(dept);

                                self.send_next_step_response();
                                self.wait_for_next_step();
                            }
                            None => {}
                        }

                    }
                }
                None => {}
            }*/
        }
    }

    pub fn set_variable_value_array(&mut self, func_id: u32, varID: u32, value: String) {

    }

    pub fn resolve_variable(&mut self, var_id: u32, value: &Value, variable: Option< &mut DebugVariable>) {
        match value {
            Value::Address(bytes) => {  }
            Value::Boolean(value) => {  },
            Value::Field(limbs) => {  },
            Value::Char(c) => {  },
            Value::Group(g) => {  },
            Value::Integer(i) => {
                match variable {
                    None => {
                        match self.debug_data.variables.get_mut(&var_id) {
                            Some(variable) => {
                                variable.value = format!("{}", i);
                            }
                            None => {}
                        }
                    }
                    Some(var) => {
                        var.type_ = DebugVariableType::Integer;
                        var.value = format!("{}", i);
                    }
                }
            },
            Value::Array(items) => {
                match variable {
                    None => {
                        match self.debug_data.variables.get_mut(&var_id) {
                            Some(variable) => {
                                let mut variable = variable.clone();
                                variable.type_ = DebugVariableType::Array;
                                variable.value = "Array".to_string();

                                let mut index = 0;
                                for item in items {
                                    //out.push(self.resolve(item, cs)?.into_owned());
                                    let mut dbg_var = DebugVariable{
                                        name: format!("[{}]", index),
                                        type_: DebugVariableType::Array,
                                        value: "".to_string(),
                                        circuit_id: 0,
                                        mutable: false,
                                        const_: false,
                                        line_start: 0,
                                        line_end: 0,
                                        sub_variables: vec![]
                                    };
                                    self.resolve_variable(var_id, item, Some(&mut dbg_var));
                                    variable.sub_variables.push(dbg_var);
                                    index += 1;
                                }

                                self.debug_data.variables.insert(var_id, variable);
                            }
                            None => {
                                return;
                            }
                        };
                    }
                    Some(var) => {
                        let mut index = 0;
                        for item in items {
                            let mut dbg_var = DebugVariable{
                                name: format!("[{}]", index),
                                type_: DebugVariableType::Array,
                                value: "".to_string(),
                                circuit_id: 0,
                                mutable: false,
                                const_: false,
                                line_start: 0,
                                line_end: 0,
                                sub_variables: vec![]
                            };

                            self.resolve_variable(var_id, item, Some(&mut dbg_var));
                            var.sub_variables.push(dbg_var);
                            index += 1;
                        }
                    }
                }
            }
            Value::Tuple(items) => {
                let mut values: Vec<String> = Vec::new();
                for item in items {
                    match item {
                        Value::Integer(int) => {
                            values.push(int.to_string());
                        }
                        _=> {}
                    }
                }

                match self.debug_data.variables.get_mut(&var_id) {
                    Some(variable) => {
                        let mut index: usize = 0;
                        for item in &mut variable.sub_variables {
                            item.value = values.get(index).unwrap().clone();
                            index += 1;
                        }
                    }
                    None =>{}
                }
            }
            Value::Str(_) =>{},
            Value::Ref(i) => {
                match self.debug_data.variables.get_mut(&var_id) {
                    Some(variable) => {
                        variable.value =  format!("{}", i);
                    }
                    None => {}
                }
            }
        }
    }

    pub fn resolve_debug_variable(&mut self, var_id: u32, value: &Value) {
        self.resolve_variable(var_id, value, None);
    }


    /*pub fn set_variable_value_string(&mut self, func_id: u32, varID: u32, value: String) {
        match self.debug_data.variables.get_mut(&varID) {
            Some(variable) => {
                variable.value = value.clone();
            }
            None => {}
        }
    }*/

    //pub fn set_variable_value_string(&mut self, func_id: u32, varID: u32, value: String) {
    pub fn set_variable_value(&mut self, func_id: u32, varID: u32, value: Value) {

        /*match self.debug_data.variables.get_mut(&varID) {
            Some(variable) => {
                variable.value = value.clone();
            }
            None =>{}
        }*/


        /*match self.debug_data.stack.last_mut() {
            Some(func) => {
                match func.variables.get_mut(&varID) {
                    Some(dbg_item) => {
                        match dbg_item {
                            Variable(variable) => {
                                variable.value = value.clone();
                            }
                            Function(_) => {}
                        }
                    }
                    None =>{}

                }
            }
            None =>{}
        }*/
    }

    pub fn set_sub_variable_values(&mut self, varID: u32, values: Vec<String>) {
        match self.debug_data.variables.get_mut(&varID) {
            Some(variable) => {
                let mut index: usize = 0;
                for item in &mut variable.sub_variables {
                    item.value = values.get(index).unwrap().clone();
                    index += 1;
                }
            }
            None =>{}
        }
    }

    pub fn set_debug_call_depth(&mut self, call_depth:u32) {
        self.call_depth = call_depth;
    }

    pub fn get_debug_call_depth(&mut self) -> u32 {
        self.call_depth
    }

    pub fn set_program_call_depth(&mut self, call_depth:u32) {
        self.cur_program_call_depth = call_depth;
    }

    pub fn get_program_call_depth(&mut self) -> u32 {
        self.cur_program_call_depth
    }

    pub fn add_to_stack(&mut self, funcID: u32) {
        if self.call_depth > (self.debug_data.stack.len() as u32) {
            match self.debug_data.functions.get(&funcID) {
                Some(func) => {
                    self.debug_data.stack.push(func.clone());
                    self.debug_data.call_dept = self.debug_data.stack.len() as u32;
                }
                None => {}
            }
        }
    }

    pub fn pop_stack(&mut self, call_depth: u32) {
        self.cur_program_call_depth = self.cur_program_call_depth - 1;
        if call_depth > 1 && call_depth == (self.debug_data.stack.len() as u32) {
            self.debug_data.stack.pop();
            self.set_debug_call_depth(call_depth - 1);
        }
    }

    pub fn send_stack_frame(&mut self) {
        let stack = StrackEvent {
            event_id: 0,
            debug_data: self.debug_data.clone()
        };
        self.tx.send(stack).unwrap();
    }

    pub fn send_next_step_response(&mut self) {
       let stack = StrackEvent {
            event_id: 1,
            debug_data: self.debug_data.clone()
        };
        self.tx.send(stack).unwrap();
    }



    unsafe fn generate_variables(add_variables: &lib::Symbol<AddVariables>, debug_event: &StrackEvent, variables: Option<&Vec<DebugVariable>>, func: &DebugFunction, ptr_variables: *mut VariableExp, count: u32, variables_reference: u32) -> u32 {
        let mut index = 0;
        let mut cur_variables_reference= variables_reference;
        let arr_variables = from_raw_parts_mut(ptr_variables as *mut VariableExp, count as usize);
        match variables {
            None => {
                for val in func.variables.iter() {
                    let item = debug_event.debug_data.variables.get(val);
                    match item {
                        Some(variable) =>{
                            //let layout = Layout::new::<VariableExp>();
                            //let variable = alloc(layout)  as *mut VariableExp;

                            let type_ = "u32".to_string();
                            arr_variables[index].name = libc::malloc(size_of::<c_char>() * variable.name.len()) as *mut c_char;
                            arr_variables[index].type_ = libc::malloc(size_of::<c_char>() * type_.len()) as *mut c_char;
                            arr_variables[index].value = libc::malloc(size_of::<c_char>() * variable.value.len()) as *mut c_char;

                            let str_name = CString::new(variable.name.clone()).unwrap().into_raw();
                            let str_value = CString::new(variable.value.clone()).unwrap().into_raw();
                            let str_type = CString::new(type_).unwrap().into_raw();
                            strcpy(arr_variables[index].name, str_name);
                            strcpy(arr_variables[index].type_, str_type);
                            strcpy(arr_variables[index].value, str_value);
                            arr_variables[index].variables_reference = 0;

                            if variable.sub_variables.len() > 0 {
                                cur_variables_reference += 1;
                                arr_variables[index].variables_reference = cur_variables_reference;
                                let sub_var_count = variable.sub_variables.len();
                                let ptr_sub_variables = libc::malloc(size_of::<VariableExp>() * sub_var_count)  as *mut VariableExp;
                                cur_variables_reference = Debugger::generate_variables(add_variables, debug_event, Some(&variable.sub_variables), func, ptr_sub_variables, sub_var_count as u32, arr_variables[index].variables_reference);
                            }

                            index += 1;
                        }
                        None => {}
                    }
                }
                if index < count as usize {
                    let str_self = "self".to_string();
                    let type_ = "u32".to_string();
                    let value = "Circuit".to_string();

                    arr_variables[index].name = libc::malloc(size_of::<c_char>() * str_self.len()) as *mut c_char;
                    arr_variables[index].type_ = libc::malloc(size_of::<c_char>() * type_.len()) as *mut c_char;
                    arr_variables[index].value = libc::malloc(size_of::<c_char>() * value.len()) as *mut c_char;

                    let str_name = CString::new(str_self.clone()).unwrap().into_raw();
                    let str_value = CString::new(value.clone()).unwrap().into_raw();
                    let str_type = CString::new(type_).unwrap().into_raw();

                    strcpy(arr_variables[index].name, str_name);
                    strcpy(arr_variables[index].type_, str_type);
                    strcpy(arr_variables[index].value, str_value);

                    match debug_event.debug_data.variables.get(&func.self_circuit_id) {
                        Some(variable) =>{
                            if variable.sub_variables.len() > 0 {
                                cur_variables_reference += 1;
                                arr_variables[index].variables_reference = cur_variables_reference;
                                let sub_var_count = variable.sub_variables.len();
                                let ptr_sub_variables = libc::malloc(size_of::<VariableExp>() * sub_var_count)  as *mut VariableExp;
                                cur_variables_reference = Debugger::generate_variables(add_variables, debug_event, Some(&variable.sub_variables), func, ptr_sub_variables, sub_var_count as u32, arr_variables[index].variables_reference);
                            }

                        }
                        None => {}
                    }
                }

                add_variables(variables_reference, ptr_variables, count );
            }

            Some(variables) => {
                for variable in  variables {
                    let type_ = "u32".to_string();
                    arr_variables[index].name = libc::malloc(size_of::<c_char>() * variable.name.len()) as *mut c_char;
                    arr_variables[index].type_ = libc::malloc(size_of::<c_char>() * type_.len()) as *mut c_char;
                    arr_variables[index].value = libc::malloc(size_of::<c_char>() * variable.value.len()) as *mut c_char;

                    let str_name = CString::new(variable.name.clone()).unwrap().into_raw();
                    let str_value = CString::new(variable.value.clone()).unwrap().into_raw();
                    let str_type = CString::new(type_).unwrap().into_raw();
                    strcpy(arr_variables[index].name, str_name);
                    strcpy(arr_variables[index].type_, str_type);
                    strcpy(arr_variables[index].value, str_value);

                    println!("name = {}; val = {}", variable.name, variable.value);

                    arr_variables[index].variables_reference = 0;

                    if variable.sub_variables.len() > 0 {
                        cur_variables_reference += 1;
                        arr_variables[index].variables_reference = cur_variables_reference;
                        let sub_var_count = variable.sub_variables.len();
                        let ptr_sub_variables = libc::malloc(size_of::<VariableExp>() * sub_var_count)  as *mut VariableExp;
                        cur_variables_reference = Debugger::generate_variables(add_variables, debug_event, Some(&variable.sub_variables), func, ptr_sub_variables, sub_var_count as u32, arr_variables[index].variables_reference);
                    }
                    index += 1;
                }
                add_variables(variables_reference, ptr_variables, count);
            }
        }
        cur_variables_reference
    }

    pub fn run_debugger(&mut self, input: &InputData) {


        unsafe {
            let register_get_stack_callback: lib::Symbol<RegisterGetStackCallback> = self.lib_main.get(b"register_get_stack_callback").unwrap();
            let register_step_in: lib::Symbol<RegisterStepIn>  = self.lib_main.get(b"register_step_in").unwrap();
            let register_terminate_debug: lib::Symbol<RegisterTerminateDebug>  = self.lib_main.get(b"register_terminate_debug").unwrap();

            register_get_stack_callback(&* self, get_stack_callback);
            register_step_in(&* self, step_in);
            register_terminate_debug(&* self, terminate_debug);
        }


        //let start = self.start_signal.clone();
        let pair2 = Arc::clone(&self.pair);
        //let (_lock, cvar) = &*pair2;
        let receiver = self.rx.clone();

        //let registers = input.registers.clone();
        let main_input = input.main.clone();
        let file_path = input.debug_data.clone();
        let mut cur_variables_reference_id = self.cur_variables_referenceID;
        let mut frameID = self.cur_stack_frameID;
        let debug_port = self.debug_data.debug_port;

        thread::spawn(move || {
            println!("Load library hello_debugger.dll");
            let path_so = Self::inner_main("debugger.dll").expect("Couldn't");
            let lib = lib::Library::new(path_so).unwrap();

            unsafe {
                let run_server: lib::Symbol<RunServer> =  lib.get(b"run_server").unwrap();
                println!("Rust: run_server");
                run_server(debug_port);
            }
        });

        thread::spawn(move || {
            println!("Load library hello_debugger.dll");
            let pathSo = Self::inner_main("debugger.dll").expect("Couldn't");
            let lib = lib::Library::new(pathSo).unwrap();
            //let debug_data = self.de
            unsafe {
                let register_callback: lib::Symbol<RegisterCallback> = lib.get(b"register_callback").unwrap();
                let register_next_step: lib::Symbol<RegisterNextStep> = lib.get(b"register_next_step").unwrap();


                let add_stack: lib::Symbol<AddStack> = lib.get(b"add_stack").unwrap();
                let next_step_response: lib::Symbol<NextStepResponse> = lib.get(b"next_step_response").unwrap();
                let add_variables: lib::Symbol<AddVariables> = lib.get(b"add_variables").unwrap();

                println!("Rust: register_callback");

                let mut rust_object = Box::new(RustObject {
                    main_file_path: file_path,
                    mutex_pair: pair2,

                });

                register_callback(&mut *rust_object,  callback);
                register_next_step(&mut *rust_object, next_step);


                loop {
                    println!("Loop");
                    let debug_event = receiver.lock().unwrap().recv().unwrap();
                    println!("receiver - debug_event");
                    if debug_event.event_id == 0 {
                        let mut stack_index = 0;
                        let ptr_stack_frame = libc::malloc(size_of::<StackFrameExp>() * debug_event.debug_data.stack.len() ) as *mut StackFrameExp;
                        let arr_stack_frame = from_raw_parts_mut(ptr_stack_frame as *mut StackFrameExp, debug_event.debug_data.stack.len());
                        let mut vec_stack:Vec<StackFrameExp> = Vec::with_capacity(debug_event.debug_data.stack.len());
                        for func in debug_event.debug_data.stack.iter() {

                            let mut vec_scope:Vec<ScopeExp> = Vec::with_capacity(1);
                            let mut vec_scopes:Vec<ScopesMapExp> = Vec::with_capacity(1);
                            let mut vec:Vec<VariableExp> = Vec::with_capacity(func.variables.len());

                            let mut variables_count = func.variables.len();
                            if func.self_circuit_id != 0 {
                                variables_count += 1; // need for self
                            }

                            /*let dbg_func = DebugFunction {
                                name: "test".to_string(),
                                self_circuit_id: 0,
                                variables: Vec::new(),
                                instructions: IndexMap::new(),
                                arguments: Vec::new(),
                                line_start: 0,
                                line_end: 0
                            };

                            let mut sub_variables: Vec<DebugVariable> = Vec::new();
                            let mut var = DebugVariable {
                                name: "arr".to_string(),
                                type_: DebugVariableType::Integer,
                                value: "".to_string(),
                                circuit_id: 0,
                                mutable: false,
                                const_: false,
                                line_start: 0,
                                line_end: 0,
                                sub_variables: Vec::new()
                            };

                            for i in 0..2000{
                                var.sub_variables.push(DebugVariable {
                                    name: format!("[{}]", i),
                                    type_: DebugVariableType::Integer,
                                    value: format!("{}", i),
                                    circuit_id: 0,
                                    mutable: false,
                                    const_: false,
                                    line_start: 0,
                                    line_end: 0,
                                    sub_variables: Vec::new()
                                });
                            }

                            sub_variables.push(var);


                            let ptr_variables = libc::malloc(size_of::<VariableExp>() * 1)  as *mut VariableExp;
                            let variables_reference_id = Debugger::generate_variables(&add_variables, &debug_event, Some(&sub_variables), &dbg_func, ptr_variables, 1, cur_variables_reference_id);
*/

                            let ptr_variables = libc::malloc(size_of::<VariableExp>() * variables_count)  as *mut VariableExp;
                            let variables_reference_id = Debugger::generate_variables(&add_variables, &debug_event, None, func, ptr_variables, variables_count as u32, cur_variables_reference_id);

                            let ptr_scope = libc::malloc(size_of::<ScopeExp>() ) as *mut ScopeExp;
                            let name = "Variables".to_string();
                            let presentation_hint =  "Variables".to_string();

                            (*ptr_scope).name = libc::malloc(size_of::<c_char>() * name.len()) as *mut c_char;
                            (*ptr_scope).presentation_hint = libc::malloc(size_of::<c_char>() * presentation_hint.len()) as *mut c_char;

                            let name = CString::new(name).unwrap().into_raw();
                            let presentation_hint =  CString::new(presentation_hint).unwrap().into_raw();

                            strcpy((*ptr_scope).name, name);
                            strcpy((*ptr_scope).presentation_hint, presentation_hint);
                            (*ptr_scope).variables_reference = cur_variables_reference_id;
                            cur_variables_reference_id = variables_reference_id;


                            let ptr_scopes = libc::malloc(size_of::<ScopesMapExp>() ) as *mut ScopesMapExp;
                            (*ptr_scopes).scopes = ptr_scope;
                            (*ptr_scopes).count = 1;


                            let str_func_name = CString::new(func.name.clone()).unwrap().into_raw();
                            arr_stack_frame[stack_index].name = libc::malloc(size_of::<c_char>() * func.name.len()) as *mut c_char;

                            arr_stack_frame[stack_index].id = frameID as i32;
                            arr_stack_frame[stack_index].scopes_map = ptr_scopes;
                            arr_stack_frame[stack_index].scopes_count = 1;
                            arr_stack_frame[stack_index].line = func.line_start as i32;
                            arr_stack_frame[stack_index].column = 1;
                            strcpy(arr_stack_frame[stack_index].name, str_func_name);

                            frameID +=1;
                            stack_index += 1;
                            //vec_stack.push(stack_frame);

                        }

                        let mut stack = StackExp {
                            stack: ptr_stack_frame,
                            stack_count: debug_event.debug_data.stack.len() as i32,
                        };

                        println!("add_stack");
                        add_stack(&mut stack);

                        for i in 0..stack.stack_count  as usize {
                            let scopes_map = from_raw_parts_mut(arr_stack_frame[i].scopes_map as *mut ScopesMapExp, arr_stack_frame[i].scopes_count as usize);

                            for j in 0..arr_stack_frame[i].scopes_count as usize {
                                libc::free(scopes_map[j].scopes as *mut c_void);
                            }
                            libc::free(arr_stack_frame[i].scopes_map as *mut c_void);
                        }

                        libc::free(stack.stack as *mut c_void);
                    } else if debug_event.event_id == 1 {
                        next_step_response();
                    }
                }


            }
        });
    }
}
