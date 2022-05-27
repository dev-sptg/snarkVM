extern crate libloading as lib;
use snarkvm_ir::{InputData, Value};
use std::path::{Path, PathBuf};
use std::io;
use std::sync::{Arc, Condvar, mpsc, Mutex};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char};
use std::ptr::{copy_nonoverlapping};
use std::env;
use std::sync::mpsc::{Receiver};
use std::thread;
use std::slice::from_raw_parts_mut;
use libc::c_void;
//use std::alloc::{alloc, dealloc, Layout};
use libloading::Library;
use snarkvm_debugdata::{DebugData, DebugFunction, DebugVariable, DebugVariableType};
use std::mem::size_of;
//use std::slice::from_raw_parts;
//use indexmap::IndexMap;
use snarkvm_debugdata::DebugVariableType::Circuit;
//use snarkvm_fields::PrimeField;
use std::process;
use snarkvm_gadgets::Boolean;


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
    pub file_path: *mut c_char,
}

#[repr(C)]
struct StackExp {
    pub stack: *mut StackFrameExp,
    pub stack_count: i32,
}

extern "C" {
    //fn printf(fmt: *const c_char, ...) -> c_int;
    //fn strlen(arr: *const c_char) -> usize;
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
}


type RunServer = fn(port: u32) -> i32;
type RegisterCallback = fn (target: *mut RustObject, cb: extern fn(target: *mut RustObject, *mut  c_char, i32));
type RegisterNextStep = fn (target: *mut RustObject, cb: extern fn(target: *mut RustObject));
type RegisterStepIn = fn (target: *const Debugger, cb: extern fn(target: *mut Debugger));
type RegisterStepOut = fn (target: *const Debugger, cb: extern fn(target: *mut Debugger));
type RegisterGetStackCallback = fn (target: *const Debugger, cb: extern fn(target: *mut Debugger));
type RegisterTerminateDebug = fn (target: *const Debugger, cb: extern fn(target: *mut Debugger));

type RegisterAddBreakpointCallback = fn (target: *const Debugger, cb: extern fn(target: *mut Debugger, src_path: *mut  c_char, line: u32));
type RegisterClearAllBreakpointsCallback = fn (target: *const Debugger, cb: extern fn(target: *mut Debugger, src_path: *mut  c_char));
type RegisterBreakpointHit = fn (target: *const Debugger, cb: extern fn(target: *mut Debugger));

type SetBreakpointLines = fn(src_id: u32, lines: *const u32, count: u32);
type AddStack = fn (stack: *mut StackExp);
type AddVariables = fn (variables_reference: u32, variables: *mut VariableExp, count: u32);
type NextStepResponse = fn ();
type BreakpointHitResponse = fn ();

extern "C" fn callback(target: *mut RustObject, src_path: *mut  c_char, _sz: i32) {
    //println!("Rust: I'm called from C");

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
    //println!("Rust:next_step : I'm called from C");
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
    //println!("Rust:step_in : I'm called from C");
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

extern "C" fn step_out(target: *mut Debugger) {
    //println!("Rust:step_in : I'm called from C");
    let debugger = unsafe {
        assert!(!target.is_null());
        &mut *target
    };

   debugger.is_step_out = true;

    let (lock, cvar) = &*debugger.pair;
    let mut started = lock.lock().unwrap();
    *started = true;
    cvar.notify_one();
}

extern "C" fn get_stack_callback(target: *mut Debugger) {
    //println!("Rust:get_stack_callback : I'm called from C");
    let debugger = unsafe {
        assert!(!target.is_null());
        &mut *target
    };

    debugger.send_stack_frame();
}


extern "C" fn add_breakpoint_callback(target: *mut Debugger,  src_path: *mut  c_char, line: u32) {
    //println!("Rust:get_stack_callback : I'm called from C");
    let debugger = unsafe {
        assert!(!target.is_null());
        &mut *target
    };

    let path = unsafe {CStr::from_ptr(src_path)};
    let string = String::from(path.to_str().unwrap());
    let file_path = string.clone();
    let path = Path::new(&string);
    if path.exists() {
        debugger.breakpoints.push(Breakpoint {
            file_path,//: string,//format!("{}", path.canonicalize().unwrap().display()),
            line
        });
    }
}

extern "C" fn clear_all_breakpoints_callback(target: *mut Debugger,  src_path: *mut  c_char) {
    //println!("Rust:get_stack_callback : I'm called from C");
    let debugger = unsafe {
        assert!(!target.is_null());
        &mut *target
    };

    let path = unsafe {CStr::from_ptr(src_path)};
    let file_path = String::from(path.to_str().unwrap());
    loop {
        match debugger.breakpoints.iter_mut().position(|r| r.file_path == file_path) {
            Some(index) => {
                debugger.breakpoints.remove(index);
            }
            None => {
                break;
            }
        }
    }

}

extern "C" fn breakpoint_hit_callback(target: *mut Debugger) {
    let debugger = unsafe {
        assert!(!target.is_null());
        &mut *target
    };

    let (lock, cvar) = &*debugger.pair;
    let mut started = lock.lock().unwrap();
    *started = true;
    debugger.is_breakpoint_hit = true;
    cvar.notify_one();
}



extern "C" fn terminate_debug(_target: *mut Debugger) {
    //println!("Rust:step_in : I'm called from C");
    /*let debugger = unsafe {
        assert!(!target.is_null());
        &mut *target
    };*/
    println!("Leo: !!!!!!!!!!!!process::exit!!!!!!!!!!!!!!!!!!");
    process::exit(0x0);
}

#[derive(Clone, Debug)]
enum DebugEvent {
    Stack,
    NextStep,
    BreakpointHit
}

#[derive(Clone, Debug)]
struct StrackEvent {
    pub event: DebugEvent,
    pub debug_data: DebugData,
}


#[derive(Clone, Debug)]
pub struct Breakpoint {
    file_path: String,
    line: u32,
}



//#[derive(Debug)]
pub struct Debugger {
    pair:Arc<(Mutex<bool>, Condvar)>,
    pub debug_data: DebugData,
    pub is_debug_mode: bool,
    cur_stack_frame_id: u32,
    cur_variables_reference_id: u32,
    cur_function_id: u32,
    tx: mpsc::Sender<StrackEvent>,
    rx: Arc<Mutex<Receiver<StrackEvent>>>,
    lib_main: Library,
    call_depth: u32,
    cur_program_call_depth: u32,
    pub is_step_into: bool,
    pub is_step_out: bool,
    pub is_call_instruction: bool,
    pub is_breakpoint_hit: bool,
    pub breakpoints: Vec<Breakpoint>
}

#[repr(C)]
struct RustObject {
    main_file_path: PathBuf,
    mutex_pair:Arc<(Mutex<bool>, Condvar)>,
}


impl Debugger {
    pub fn new(debug_data: DebugData) -> Self {
        let (tx, rx) = mpsc::channel();

        let mut dap_lib = "".to_string();
        if cfg!(windows) {
            println!("Leo: load library debugger.dll");
            dap_lib = "debugger.dll".to_string();
            println!("Leo: this is windows");
        } else if cfg!(unix) {
            println!("Leo: oad library libdebugger.so");
            dap_lib = "libdebugger.so".to_string();
            println!("Leo: this is unix");
        }

        let path_so = Self::inner_main(dap_lib.as_str()).expect("Couldn't");
        let lib_main = lib::Library::new(path_so).unwrap();
        let debug_mode = debug_data.debug;
        Self {
            pair: Arc::new((Mutex::new(false), Condvar::new())),
            debug_data: debug_data,
            is_debug_mode: debug_mode,
            cur_stack_frame_id: 200,
            cur_variables_reference_id: 300,
            cur_function_id: 0,
            tx: tx,
            rx: Arc::new(Mutex::new(rx)),
            lib_main: lib_main,
            call_depth: 1,
            cur_program_call_depth: 1,
            is_step_into: false,
            is_step_out: false,
            is_call_instruction: false,
            is_breakpoint_hit: false,
            breakpoints: Vec::new()
        }
    }

    fn inner_main(str: &str) -> io::Result<PathBuf> {
        let mut dir = env::current_exe()?;
        dir.pop();
        dir.push(str);
        Ok(dir)
    }

    pub fn wait_for_next_step(&mut self) {
        if !self.is_debug_mode {
            return;
        }

        let (lock, cvar) = &*self.pair;
        let started = lock.lock().unwrap();
        cvar.wait(started);
    }

    pub fn update_position(&mut self, current_line: u32, line_end: u32) {
        if !self.is_debug_mode {
            return;
        }

        match self.debug_data.stack.last_mut() {
            Some(func) => {
                func.current_line = current_line;
                func.line_end = line_end;
            }
            None =>{}
        }
    }

    pub fn set_self_reference(&mut self, self_circuit_id: u32) {
        if !self.is_debug_mode {
            return;
        }

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

    pub fn step_out(&mut self) {
        if self.is_step_out {
            self.is_step_out = false;

            let debug_call_depth =  self.get_debug_call_depth();
            self.pop_stack(debug_call_depth);
            self.send_next_step_response();
            self.wait_for_next_step();
        }
    }

    pub fn evaluate_instruction(&mut self, function_index: u32,  instruction_index: u32 ) {
        if !self.is_debug_mode {
            return;
        }

        if self.call_depth == (self.debug_data.stack.len() as u32) && self.call_depth == self.cur_program_call_depth {

            match self.debug_data.functions.get_mut(&function_index) {
                Some(func) => {
                    let line_start = func.line_start;
                    let line_end = func.line_end;

                    if self.is_step_into && self.is_call_instruction {
                        self.update_position(line_start, line_end);

                        self.send_next_step_response();
                        self.wait_for_next_step();

                        self.step_out();

                    }
                }
                None => {
                }
            };

            self.is_step_into = false;
            self.is_call_instruction = false;
            if instruction_index == std::u32::MAX {
                return;
            }
            match self.debug_data.functions.get_mut(&function_index) {
                Some(func) => {
                    let file_path = func.file_path.clone();
                    match func.instructions.get_mut(&instruction_index) {
                        Some(instruction) => {
                            let instruction_line_start = instruction.line_start;
                            let instruction_line_end = instruction.line_end;


                            if self.is_breakpoint_hit {
                                match self.breakpoints.iter_mut().position(|r| {
                                    let path_func  = Path::new(&file_path);
                                    let path_breakpoint = Path::new(&r.file_path);
                                    let path_func = format!("{}", path_func.canonicalize().unwrap().display());
                                    let path_breakpoint = format!("{}", path_breakpoint.canonicalize().unwrap().display());

                                    //print!("{} : {}", path_func, path_breakpoint);

                                    path_func == path_breakpoint && r.line == instruction_line_start
                                }
                                ) {
                                    Some(_) => {

                                        self.update_position(instruction_line_start, instruction_line_end);
                                        self.send_breakpoint_hit_response();
                                        self.wait_for_next_step();
                                        self.step_out();
                                    }
                                    None => {

                                    }
                                }
                            } else {
                                self.update_position(instruction_line_start, instruction_line_end);
                                self.send_next_step_response();
                                self.wait_for_next_step();
                                self.step_out();
                            }

                        }
                        None => {}
                    }

                }
                None =>{
                }
            }
        }
    }

    pub fn resolve_variable(&mut self, var_id: u32, value: &Value, variable: Option< &mut DebugVariable>) {
        if !self.is_debug_mode {
            return;
        }

        match value {
            Value::Address(_bytes) => {  }
            Value::Boolean(_value) => {  },
            Value::Field(_limbs) => {  },
            Value::Char(_c) => {  },
            Value::Group(_g) => {  },
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
                                        is_argument: false,
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
                                is_argument: false,
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
        if !self.is_debug_mode {
            return;
        }

        self.resolve_variable(var_id, value, None);
    }

    pub fn set_sub_variable_values(&mut self, var_id: u32, values: Vec<String>) {
        if !self.is_debug_mode {
            return;
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

    pub fn set_debug_call_depth(&mut self, call_depth:u32) {
        if !self.is_debug_mode {
            return;
        }

        self.call_depth = call_depth;
    }

    pub fn get_debug_call_depth(&mut self) -> u32 {
        self.call_depth
    }

    pub fn set_program_call_depth(&mut self, call_depth:u32) {
        if !self.is_debug_mode {
            return;
        }
        self.cur_program_call_depth = call_depth;
    }

    pub fn get_program_call_depth(&mut self) -> u32 {
        self.cur_program_call_depth
    }

    pub fn add_to_stack(&mut self, func_id: u32) {
        if !self.is_debug_mode {
            return;
        }

        if self.call_depth > (self.debug_data.stack.len() as u32) {
            match self.debug_data.functions.get(&func_id) {
                Some(func) => {
                    let mut new_func = func.clone();
                    new_func.current_line = new_func.line_start;
                    self.debug_data.stack.push(new_func);
                    self.debug_data.call_dept = self.debug_data.stack.len() as u32;
                }
                None => {}
            }
        }
    }

    pub fn pop_stack(&mut self, call_depth: u32) {
        if !self.is_debug_mode {
            return;
        }

        self.cur_program_call_depth = self.cur_program_call_depth - 1;
        if self.cur_program_call_depth < 1 {
            self.cur_program_call_depth = 1;
            return;
        }

        if call_depth > 1 && call_depth == (self.debug_data.stack.len() as u32) {
            self.debug_data.stack.pop();
            self.set_debug_call_depth(call_depth - 1);
        }
    }

    pub fn send_stack_frame(&mut self) {
        if !self.is_debug_mode {
            return;
        }

        let stack = StrackEvent {
            event: DebugEvent::Stack,
            debug_data: self.debug_data.clone()
        };
        self.tx.send(stack).unwrap();
    }

    pub fn send_next_step_response(&mut self) {
        if !self.is_debug_mode {
            return;
        }

       let stack = StrackEvent {
            event: DebugEvent::NextStep,
            debug_data: self.debug_data.clone()
        };
        self.tx.send(stack).unwrap();
    }

    pub fn send_breakpoint_hit_response(&mut self) {
        if !self.is_debug_mode {
            return;
        }

        self.is_breakpoint_hit = false;

        let stack = StrackEvent {
            event: DebugEvent::BreakpointHit,
            debug_data: self.debug_data.clone()
        };
        self.tx.send(stack).unwrap();
    }

    unsafe fn free_variables(ptr_variables: *mut VariableExp, count: u32) {
        let arr_variables = from_raw_parts_mut(ptr_variables as *mut VariableExp, count as usize);
        /*for variable in arr_variables.iter() {
            libc::free(variable.name as *mut c_void);
            libc::free(variable.type_ as *mut c_void);
            libc::free(variable.value as *mut c_void);
        }*/

        libc::free(ptr_variables  as *mut c_void);
    }


    unsafe fn generate_variables(is_argument: bool, add_variables: &lib::Symbol<AddVariables>, debug_event: &StrackEvent, variables: Option<&Vec<DebugVariable>>, func: &DebugFunction, ptr_variables: *mut VariableExp, count: u32, variables_reference: u32) -> u32 {
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
                            if is_argument != variable.is_argument {
                                continue;
                            }

                            let type_ = "u32".to_string();
                            arr_variables[index].name = libc::malloc(size_of::<c_char>() * (variable.name.len() + 1)) as *mut c_char;
                            arr_variables[index].type_ = libc::malloc(size_of::<c_char>() * (type_.len() + 1)) as *mut c_char;
                            arr_variables[index].value = libc::malloc(size_of::<c_char>() * (variable.value.len() + 1)) as *mut c_char;

                            let str_name = CString::new(variable.name.clone()).unwrap().into_raw();
                            let str_value = CString::new(variable.value.clone()).unwrap().into_raw();
                            let str_type = CString::new(type_).unwrap().into_raw();
                            //strcpy(arr_variables[index].name, str_name);
                            //strcpy(arr_variables[index].type_, str_type);
                            //strcpy(arr_variables[index].value, str_value);

                            strcpy(arr_variables[index].name, str_name);
                            strcpy(arr_variables[index].type_, str_type);
                            strcpy(arr_variables[index].value, str_value);
                            arr_variables[index].variables_reference = 0;

                            if variable.sub_variables.len() > 0 {
                                cur_variables_reference += 1;
                                arr_variables[index].variables_reference = cur_variables_reference;
                                let sub_var_count = variable.sub_variables.len();
                                let ptr_sub_variables = libc::malloc(size_of::<VariableExp>() * sub_var_count)  as *mut VariableExp;
                                cur_variables_reference = Debugger::generate_variables(is_argument, add_variables, debug_event, Some(&variable.sub_variables), func, ptr_sub_variables, sub_var_count as u32, arr_variables[index].variables_reference);
                            }

                            index += 1;
                        }
                        None => {}
                    }
                }
                if index < count as usize && !is_argument {
                    let str_self = "self".to_string();
                    let type_ = "u32".to_string();
                    let value = "Circuit".to_string();

                    arr_variables[index].name = libc::malloc(size_of::<c_char>() * (str_self.len() + 1)) as *mut c_char;
                    arr_variables[index].type_ = libc::malloc(size_of::<c_char>() * (type_.len() + 1)) as *mut c_char;
                    arr_variables[index].value = libc::malloc(size_of::<c_char>() * (value.len() + 1)) as *mut c_char;

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
                                cur_variables_reference = Debugger::generate_variables(is_argument, add_variables, debug_event, Some(&variable.sub_variables), func, ptr_sub_variables, sub_var_count as u32, arr_variables[index].variables_reference);
                            }

                        }
                        None => {}
                    }
                }

                add_variables(variables_reference, ptr_variables, count );
                Debugger::free_variables(ptr_variables, count);
            }

            Some(variables) => {
                for variable in  variables {

                    let type_ = "u32".to_string();
                    arr_variables[index].name = libc::malloc(size_of::<c_char>() * (variable.name.len() + 1)) as *mut c_char;
                    arr_variables[index].type_ = libc::malloc(size_of::<c_char>() * (type_.len() + 1)) as *mut c_char;
                    arr_variables[index].value = libc::malloc(size_of::<c_char>() * (variable.value.len() + 1)) as *mut c_char;

                    let str_name = CString::new(variable.name.clone()).unwrap().into_raw();
                    let str_value = CString::new(variable.value.clone()).unwrap().into_raw();
                    let str_type = CString::new(type_).unwrap().into_raw();
                    strcpy(arr_variables[index].name, str_name);
                    strcpy(arr_variables[index].type_, str_type);
                    strcpy(arr_variables[index].value, str_value);

                    //println!("name = {}; val = {}", variable.name, variable.value);

                    arr_variables[index].variables_reference = 0;

                    if variable.sub_variables.len() > 0 {
                        cur_variables_reference += 1;
                        arr_variables[index].variables_reference = cur_variables_reference;
                        let sub_var_count = variable.sub_variables.len();
                        let ptr_sub_variables = libc::malloc(size_of::<VariableExp>() * sub_var_count)  as *mut VariableExp;
                        cur_variables_reference = Debugger::generate_variables(is_argument, add_variables, debug_event, Some(&variable.sub_variables), func, ptr_sub_variables, sub_var_count as u32, arr_variables[index].variables_reference);
                    }
                    index += 1;
                }
                add_variables(variables_reference, ptr_variables, count);
                Debugger::free_variables(ptr_variables, count);
            }
        }
        cur_variables_reference
    }

    pub fn run_debugger(&mut self, input: &InputData) {
        if !self.is_debug_mode {
            return;
        }

        unsafe {
            let register_get_stack_callback: lib::Symbol<RegisterGetStackCallback> = self.lib_main.get(b"register_get_stack_callback").unwrap();
            let register_step_in: lib::Symbol<RegisterStepIn>  = self.lib_main.get(b"register_step_in").unwrap();
            let register_step_out: lib::Symbol<RegisterStepIn>  = self.lib_main.get(b"register_step_out").unwrap();
            let register_terminate_debug: lib::Symbol<RegisterTerminateDebug>  = self.lib_main.get(b"register_terminate_debug").unwrap();
            let set_breakpoint_lines: lib::Symbol<SetBreakpointLines>  = self.lib_main.get(b"set_breakpoint_lines").unwrap();
            let register_add_breakpoint_callback: lib::Symbol<RegisterAddBreakpointCallback>  = self.lib_main.get(b"register_add_breakpoint_callback").unwrap();
            let register_clear_all_breakpoints_callback: lib::Symbol<RegisterClearAllBreakpointsCallback>  = self.lib_main.get(b"register_clear_all_breakpoints_callback").unwrap();
            let register_breakpoint_hit_callback: lib::Symbol<RegisterBreakpointHit> = self.lib_main.get(b"register_breakpoint_hit_callback").unwrap();

            register_get_stack_callback(&* self, get_stack_callback);
            register_step_in(&* self, step_in);
            register_step_out(&* self, step_out);
            register_terminate_debug(&* self, terminate_debug);
            register_add_breakpoint_callback(&* self, add_breakpoint_callback);
            register_clear_all_breakpoints_callback(&* self, clear_all_breakpoints_callback);
            register_breakpoint_hit_callback(&* self, breakpoint_hit_callback);

            let mut instructions: Vec<u32> = Vec::new();
            for (_key, function) in &self.debug_data.functions {
                for (_key, instruction) in &function.instructions {
                    instructions.push(instruction.line_start);
                }
            }
            let mut instructions_mem: Vec<u32> = Vec::with_capacity(instructions.len());
            instructions_mem = instructions.clone();

            set_breakpoint_lines(400, instructions_mem.as_ptr(), instructions_mem.len() as u32);
        }


        //let start = self.start_signal.clone();
        let pair2 = Arc::clone(&self.pair);
        //let (_lock, cvar) = &*pair2;
        let receiver = self.rx.clone();

        //let registers = input.registers.clone();
        //let main_input = input.main.clone();
        let file_path = input.debug_data.clone();
        let mut cur_variables_reference_id = self.cur_variables_reference_id;
        let mut stack_frame_id = self.cur_stack_frame_id;
        let debug_port = self.debug_data.debug_port;

        thread::spawn(move || {
            let mut dap_lib = "".to_string();
            if cfg!(windows) {
                println!("Leo: load library debugger.dll");
                dap_lib = "debugger.dll".to_string();
                println!("Leo: this is windows");
            } else if cfg!(unix) {
                println!("Leo: load library libdebugger.so");
                dap_lib = "libdebugger.so".to_string();
                println!("Leo: this is unix");
            }

            let path_so = Self::inner_main(dap_lib.as_str()).expect("Couldn't");
            let lib = lib::Library::new(path_so).unwrap();

            unsafe {
                let run_server: lib::Symbol<RunServer> =  lib.get(b"run_server").unwrap();
                println!("Leo: run debugger server");
                run_server(debug_port);
            }
        });

        thread::spawn(move || {
            let mut dap_lib = "".to_string();
            if cfg!(windows) {
                println!("Leo: load library debugger.dll");
                dap_lib = "debugger.dll".to_string();
                println!("Leo: this is windows");
            } else if cfg!(unix) {
                println!("Leo: load library libdebugger.so");
                dap_lib = "libdebugger.so".to_string();
                println!("Leo: this is unix");
            }


            let path_so = Self::inner_main(dap_lib.as_str()).expect("Couldn't");
            let lib = lib::Library::new(path_so).unwrap();
            //let debug_data = self.de
            unsafe {
                let register_callback: lib::Symbol<RegisterCallback> = lib.get(b"register_callback").unwrap();
                let register_next_step: lib::Symbol<RegisterNextStep> = lib.get(b"register_next_step").unwrap();


                let add_stack: lib::Symbol<AddStack> = lib.get(b"add_stack").unwrap();
                let next_step_response: lib::Symbol<NextStepResponse> = lib.get(b"next_step_response").unwrap();
                let breakpoint_hit_response: lib::Symbol<BreakpointHitResponse> = lib.get(b"breakpoint_hit_response").unwrap();
                let add_variables: lib::Symbol<AddVariables> = lib.get(b"add_variables").unwrap();

                println!("Leo: register debugger callback");

                let mut rust_object = Box::new(RustObject {
                    main_file_path: file_path.clone(),
                    mutex_pair: pair2,

                });

                register_callback(&mut *rust_object,  callback);
                register_next_step(&mut *rust_object, next_step);

                println!("Leo: run debugger event loop");
                loop {
                    //
                    let debug_event = match receiver.lock() {
                        Ok(res) => {
                            match res.recv() {
                                Ok(rc) => {
                                    rc
                                }
                                Err(_e) => {
                                    return;
                                }
                            }
                        }
                        Err(_e) => {
                            return;
                        }
                    };

                    //let debug_event = receiver.lock().unwrap().recv().unwrap();
                    //println!("receiver - debug_event.event_id = {}", debug_event.event_id);

                    match debug_event.event {
                        DebugEvent::Stack => {
                            let mut frame_id = stack_frame_id;
                            let mut stack_index = 0;
                            let ptr_stack_frame = libc::malloc(size_of::<StackFrameExp>() * debug_event.debug_data.stack.len() ) as *mut StackFrameExp;
                            let arr_stack_frame = from_raw_parts_mut(ptr_stack_frame as *mut StackFrameExp, debug_event.debug_data.stack.len());
                            for func in debug_event.debug_data.stack.iter() {
                                let mut variables_count = func.get_variables_count(false, &debug_event.debug_data.variables);
                                let mut arguments_count = func.get_variables_count(true, &debug_event.debug_data.variables);
                                if func.self_circuit_id != 0 {
                                    variables_count += 1; // need for self
                                }

                                let ptr_variables = libc::malloc(size_of::<VariableExp>() * variables_count as usize)  as *mut VariableExp;
                                let variables_reference_id = Debugger::generate_variables(false, &add_variables, &debug_event, None, func, ptr_variables, variables_count, cur_variables_reference_id);

                                let scope_count = if arguments_count > 0 {2} else {1};
                                let ptr_scope = libc::malloc(size_of::<ScopeExp>() *  scope_count) as *mut ScopeExp;
                                let ptr_scopes = from_raw_parts_mut(ptr_scope as *mut ScopeExp, scope_count as usize);

                                let name = "Variables".to_string();
                                let presentation_hint =  "locals".to_string();

                                (ptr_scopes[0]).name = libc::malloc(size_of::<c_char>() * name.len()) as *mut c_char;
                                (ptr_scopes[0]).presentation_hint = libc::malloc(size_of::<c_char>() * presentation_hint.len()) as *mut c_char;

                                let name = CString::new(name).unwrap().into_raw();
                                let presentation_hint =  CString::new(presentation_hint).unwrap().into_raw();

                                strcpy((ptr_scopes[0]).name, name);
                                strcpy((ptr_scopes[0]).presentation_hint, presentation_hint);
                                (ptr_scopes[0]).variables_reference = cur_variables_reference_id;
                                cur_variables_reference_id = variables_reference_id + 1;

                                if arguments_count > 0 {
                                    let ptr_arguments = libc::malloc(size_of::<VariableExp>() * arguments_count as usize)  as *mut VariableExp;
                                    let variables_reference_id = Debugger::generate_variables(true, &add_variables, &debug_event, None, func, ptr_arguments, arguments_count, cur_variables_reference_id);

                                    let name = "Arguments".to_string();
                                    let presentation_hint =  "arguments".to_string();

                                    (ptr_scopes[1]).name = libc::malloc(size_of::<c_char>() * name.len()) as *mut c_char;
                                    (ptr_scopes[1]).presentation_hint = libc::malloc(size_of::<c_char>() * presentation_hint.len()) as *mut c_char;

                                    let name = CString::new(name).unwrap().into_raw();
                                    let presentation_hint =  CString::new(presentation_hint).unwrap().into_raw();

                                    strcpy((ptr_scopes[1]).name, name);
                                    strcpy((ptr_scopes[1]).presentation_hint, presentation_hint);
                                    (ptr_scopes[1]).variables_reference = cur_variables_reference_id;
                                    cur_variables_reference_id = variables_reference_id + 1;
                                }

                                let ptr_scopes = libc::malloc(size_of::<ScopesMapExp>() ) as *mut ScopesMapExp;
                                (*ptr_scopes).scopes = ptr_scope;
                                (*ptr_scopes).count = scope_count as i32;

                                let str_file_path = CString::new(func.file_path.clone()).unwrap().into_raw();
                                let str_func_name = CString::new(func.name.clone()).unwrap().into_raw();
                                arr_stack_frame[stack_index].name = libc::malloc(size_of::<c_char>() * func.name.len()) as *mut c_char;
                                arr_stack_frame[stack_index].file_path = libc::malloc(size_of::<c_char>() * func.file_path.len()) as *mut c_char;

                                arr_stack_frame[stack_index].id = frame_id as i32;
                                arr_stack_frame[stack_index].scopes_map = ptr_scopes;
                                arr_stack_frame[stack_index].scopes_count = 1;
                                arr_stack_frame[stack_index].line = func.current_line as i32;
                                arr_stack_frame[stack_index].column = 1;
                                strcpy(arr_stack_frame[stack_index].name, str_func_name);
                                strcpy(arr_stack_frame[stack_index].file_path, str_file_path);

                                frame_id +=1;
                                stack_index += 1;

                            }

                            let mut stack = StackExp {
                                stack: ptr_stack_frame,
                                stack_count: debug_event.debug_data.stack.len() as i32,
                            };

                            println!("Leo: add debug stack");
                            add_stack(&mut stack);

                            for i in 0..stack.stack_count  as usize {
                                let scopes_map = from_raw_parts_mut(arr_stack_frame[i].scopes_map as *mut ScopesMapExp, arr_stack_frame[i].scopes_count as usize);
                                for j in 0..arr_stack_frame[i].scopes_count as usize {
                                    let scopes = from_raw_parts_mut(scopes_map[j].scopes as *mut ScopeExp, scopes_map[j].count as usize);
                                    for x in 0..scopes_map[j].count as usize {
                                        libc::free(scopes[x].name as *mut c_void);
                                        libc::free(scopes[x].presentation_hint as *mut c_void);
                                    }
                                    libc::free(scopes_map[j].scopes as *mut c_void);
                                }
                                libc::free(arr_stack_frame[i].name as *mut c_void);
                                libc::free(arr_stack_frame[i].file_path as *mut c_void);
                                libc::free(arr_stack_frame[i].scopes_map as *mut c_void);
                            }

                            libc::free(stack.stack as *mut c_void);
                        }
                        DebugEvent::NextStep => {
                            next_step_response();
                        }
                        DebugEvent::BreakpointHit => {
                            breakpoint_hit_response();
                        }
                    }
                }


            }
        });
    }
}
