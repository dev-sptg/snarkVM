extern crate libloading as lib;
use snarkvm_ir::{InputData, Program};
use std::path::PathBuf;
use std::io;
use indexmap::IndexMap;
use std::sync::{Arc, Condvar, mpsc, Mutex};
use std::ffi::{c_void, CString};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr::{copy_nonoverlapping, null, null_mut};
use std::env;
use std::fmt::Display;
use std::ptr;
use std::sync::mpsc::{Receiver, Sender, sync_channel};
use std::sync::mpsc::SyncSender;

use std::thread;
use std::time::Duration;
use snarkvm_debugdata::{DebugData, DebugVariable};
use snarkvm_debugdata::DebugItem::{Function, Variable};


#[repr(C)]
struct RustObject {
    main_file_path: PathBuf,
    mutex_pair:Arc<(Mutex<bool>, Condvar)>
}

type RunServer = fn() -> i32;
type RegisterCallback = fn (target: *mut RustObject, cb: extern fn(target: *mut RustObject, *mut  c_char, i32));
type RegisterNextStep = fn (target: *mut RustObject, cb: extern fn(target: *mut RustObject));
type RegisterGetStackCallback = fn (target: *mut RustObject, cb: extern fn(target: *mut RustObject));
type AddStackframe = fn (stack_frame: *mut StackFrameExp);

#[repr(C)]
struct VariableExp {
    pub str_name: *mut c_char,
    pub name_len: u16,

    pub str_type: *mut c_char,
    pub type_len: u16,

    pub str_value: *mut c_char,
    pub value_len: u16,

    pub variables_reference: i32
}


#[repr(C)]
struct ScopeExp {
    pub str_name: *mut c_char,
    pub name_len: u16,

    pub presentation_hint: *mut c_char,
    pub presentation_hint_len: u16,

    pub variables: *mut VariableExp,
    pub variables_len: u16,

    pub variables_reference: i32
}

#[repr(C)]
struct ScopesExp {
    pub scopes: *mut ScopeExp,
    pub count: i32
}


#[repr(C)]
struct StackFrameExp {
    pub id: i32,
    pub scopes: *mut ScopesExp,
    pub scopes_count: i32,

    pub line: i32,
    pub column: i32,
    pub str_name: *mut c_char,
    pub name_len: u16,
}

#[repr(C)]
struct TestExp {
    pub id: i32,
}

extern "C" {
    fn printf(fmt: *const c_char, ...) -> c_int;
    fn strlen(arr: *const c_char) -> usize;
}

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

extern "C" fn get_stack_callback(target: *mut RustObject) {
    println!("Rust:get_stack_callback : I'm called from C");
    let robject = unsafe {
        assert!(!target.is_null());
        &mut *target
    };

    let (lock, cvar) = &*robject.mutex_pair;
    let mut started = lock.lock().unwrap();
    *started = true;
    cvar.notify_one();
}

#[derive(Clone, Debug)]
struct StrackEvend {
    pub cur_functionID: u32,
    pub debug_data: DebugData,
}


#[derive(Clone, Debug)]
pub struct Debugger<'a> {
    pub program: &'a Program,
    pair:Arc<(Mutex<bool>, Condvar)>,
    pub debug_data: DebugData,
    cur_stack_frameID: u32,
    cur_variables_referenceID: u32,
    cur_functionID: u32,
    tx: mpsc::Sender<StrackEvend>,
    rx: Arc<Mutex<Receiver<StrackEvend>>>,
    //txrx:Arc< (Sender<u32>, Receiver<u32>)>
    // txrx:(SyncSender<u32>, Receiver<u32>)
}



impl <'a> Debugger<'a> {
    pub fn new(debug_data: DebugData,  program: &'a Program) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            program: program,
            pair: Arc::new((Mutex::new(false), Condvar::new())),
            debug_data: debug_data,
            cur_stack_frameID: 200,
            cur_variables_referenceID: 300,
            cur_functionID: 0,
            tx: tx,
            rx: Arc::new(Mutex::new(rx))
            //txrx: Arc::new(mpsc::channel())
            //txrx: sync_channel(1)
        }
    }

    pub fn send_variable_data(variables: &IndexMap<u32, DebugVariable>) {

    }

    fn inner_main(str: &str) -> io::Result<PathBuf> {
        let mut dir = env::current_exe()?;
        dir.pop();
        dir.push(str);
        Ok(dir)
    }

    pub fn wait_for_next_step(&mut self) {
        let (lock, cvar) = &* self.pair;
        let mut started = lock.lock().unwrap();
        cvar.wait(started);
    }

    pub fn set_variable_value(&mut self, funcID: u32, varID: u32, value: String) {
        match self.debug_data.data.get_mut(&funcID).expect("unresolved function")  {
            Variable(var) => {

            }
            Function(func) => {
                let variable = func.variables.get_mut(&varID).expect("unresolved function");
                variable.value = value.clone();
                //func.add_variable(funID, dbg_var);
            }
        }
        //tx.send(123).unwrap();
    }

    pub fn send_stack_frame(&mut self, funcID: u32) {
        let stack = StrackEvend{
            cur_functionID: funcID,
            debug_data: self.debug_data.clone()
        };
        self.tx.send(stack).unwrap();
        //tx.send(123).unwrap();
    }

    pub fn run_debugger(&mut self, input: &InputData) {

        //let start = self.start_signal.clone();
        let pair2 = Arc::clone(&self.pair);
        let (lock, cvar) = &*pair2;
        let receiver = self.rx.clone();

        let registers = input.registers.clone();
        let main_input = input.main.clone();
        let file_path = input.debug_data.clone();
        let mut cur_variables_referenceID = self.cur_variables_referenceID;
        let mut frameID = self.cur_stack_frameID;

        thread::spawn(move || {
            println!("Load library hello_debugger.dll");
            let pathSo = Self::inner_main("debugger.dll").expect("Couldn't");
            let lib = lib::Library::new(pathSo).unwrap();

            unsafe {
                let run_server: lib::Symbol<RunServer> =  lib.get(b"run_server").unwrap();
                println!("Rust: run_server");
                run_server();
            }
        });

        thread::spawn(move || {
            println!("Load library hello_debugger.dll");
            let pathSo = Self::inner_main("debugger.dll").expect("Couldn't");
            let lib = lib::Library::new(pathSo).unwrap();
            //let debug_data = self.de
            unsafe {
                let register_callback: lib::Symbol<RegisterCallback> = lib.get(b"register_callback").unwrap();
                let register_next_step: lib::Symbol<RegisterNextStep>  = lib.get(b"register_next_step").unwrap();
                let register_get_stack_callback: lib::Symbol<RegisterGetStackCallback> = lib.get(b"register_get_stack_callback").unwrap();
                let add_stackframe: lib::Symbol<AddStackframe> = lib.get(b"add_stackframe").unwrap();

                /*
                let variables_reference_registers: i32 = 300;
                let variables_reference_main_input: i32 = 301;
                let mut vec:Vec<VariableExp> = Vec::with_capacity(registers.len());
                let mut vec_main:Vec<VariableExp> = Vec::with_capacity(main_input.len());
                for (key, val) in registers.iter() {
                    let name = CString::new(key.clone()).unwrap().into_raw();
                    let str_val = CString::new(format!("{}", val).clone()).unwrap().into_raw();

                    let mut variable = VariableExp {
                        str_name: name,
                        name_len: key.len() as u16,
                        str_type: ptr::null_mut(),
                        type_len: 0,
                        str_value: str_val,
                        value_len: strlen(str_val) as u16,
                        variables_reference: variables_reference_registers
                    };

                    vec.push(variable);
                }

                for (key, val) in main_input.iter() {
                    let name = CString::new(key.clone()).unwrap().into_raw();
                    let str_val = CString::new(format!("{}", val).clone()).unwrap().into_raw();

                    let mut variable = VariableExp {
                        str_name: name,
                        name_len: key.len() as u16,
                        str_type: ptr::null_mut(),
                        type_len: 0,
                        str_value: str_val,
                        value_len: strlen(str_val) as u16,
                        variables_reference: variables_reference_main_input
                    };

                    vec_main.push(variable);
                }

                let reg_name = "Registers";
                let name = CString::new(reg_name).unwrap().into_raw();
                let presentation_hint =  CString::new("register").unwrap().into_raw();
                //let name = name.to;//.into_raw();

                let mut scope_reg = ScopeExp {
                    str_name: name,
                    name_len: strlen(name) as u16,
                    presentation_hint: presentation_hint,
                    presentation_hint_len: strlen(presentation_hint) as u16,
                    variables:  vec.as_mut_ptr(),
                    variables_len: vec.len() as u16,
                    variables_reference: variables_reference_registers
                };

                let reg_name = "Variables";
                let name = CString::new(reg_name).unwrap().into_raw();
                let presentation_hint =  CString::new("main inputs").unwrap().into_raw();
                //let name = name.to;//.into_raw();

                let mut scope_main = ScopeExp {
                    str_name: name,
                    name_len: strlen(name) as u16,
                    presentation_hint: presentation_hint,
                    presentation_hint_len: strlen(presentation_hint) as u16,
                    variables:  vec_main.as_mut_ptr(),
                    variables_len: vec_main.len() as u16,
                    variables_reference: variables_reference_main_input
                };

                let mut vec:Vec<ScopeExp> = Vec::with_capacity(2);
                vec.push(scope_reg);
                vec.push(scope_main);

                let mut scopes = ScopesExp{
                    scopes:  vec.as_mut_ptr(),
                    count: vec.len() as i32,
                };*/

                /*let mut scope = ScopeExp {
                    name: [0; 255],
                    presentation_hint: [0; 255],
                    variables_reference: 0
                };

                let name = CString::new("Locals").expect("CString::new failed");
                let presentation_hint = CString::new("locals").expect("CString::new failed");

                copy_nonoverlapping(name.as_ptr(), scope.name.as_mut_ptr(), name.as_bytes_with_nul().len());
                copy_nonoverlapping(presentation_hint.as_ptr(), scope.presentation_hint.as_mut_ptr(), presentation_hint.as_bytes_with_nul().len());
                scope.variables_reference = 100;


                let mut scope1 = ScopeExp {
                    name: [0; 255],
                    presentation_hint: [0; 255],
                    variables_reference: 0
                };

                let name1 = CString::new("Globals").expect("CString::new failed");
                let presentation_hint1 = CString::new("globals").expect("CString::new failed");

                copy_nonoverlapping(name1.as_ptr(), scope1.name.as_mut_ptr(), name1.as_bytes_with_nul().len());
                copy_nonoverlapping(presentation_hint1.as_ptr(), scope1.presentation_hint.as_mut_ptr(), presentation_hint1.as_bytes_with_nul().len());
                scope1.variables_reference = 101;



                let mut scope2 = ScopeExp {
                    name: [0; 255],
                    presentation_hint: [0; 255],
                    variables_reference: 0
                };

                let name2 = CString::new("Static").expect("CString::new failed");
                let presentation_hint2 = CString::new("static").expect("CString::new failed");

                copy_nonoverlapping(name2.as_ptr(), scope2.name.as_mut_ptr(), name2.as_bytes_with_nul().len());
                copy_nonoverlapping(presentation_hint2.as_ptr(), scope2.presentation_hint.as_mut_ptr(), presentation_hint2.as_bytes_with_nul().len());
                scope2.variables_reference = 102;*/

                println!("Rust: register_callback");

                let mut rust_object = Box::new(RustObject {
                    main_file_path: file_path,
                    mutex_pair: pair2,
                });

                register_callback(&mut *rust_object,  callback);
                register_next_step(&mut *rust_object, next_step);
                register_get_stack_callback(&mut *rust_object, get_stack_callback);
                //add_scopes(&mut scopes);

                loop {
                    println!("Loop");
                    let debug_event = receiver.lock().unwrap().recv().unwrap();


                    let mut vec_main:Vec<VariableExp> = Vec::with_capacity(main_input.len());

                    match debug_event.debug_data.data.get(&debug_event.cur_functionID).expect("unresolved function")  {
                        Variable(var) => {

                        }
                        Function(func) => {
                            let mut vec:Vec<VariableExp> = Vec::with_capacity(func.variables.len());
                            for (key, val) in func.variables.iter() {
                                let str_name = CString::new(val.name.clone()).unwrap().into_raw();
                                let str_value = CString::new(val.value.clone()).unwrap().into_raw();
                                let str_type = CString::new(val.type_.clone()).unwrap().into_raw();

                                let mut variable = VariableExp {
                                    str_name: str_name,
                                    name_len: strlen(str_name) as u16,
                                    str_type: str_type,
                                    type_len: strlen(str_type) as u16,
                                    str_value: str_value,
                                    value_len: strlen(str_value) as u16,
                                    variables_reference: 0
                                };

                                vec.push(variable);
                            }

                            let name = CString::new("Variables").unwrap().into_raw();
                            let presentation_hint =  CString::new("variables").unwrap().into_raw();

                            let mut scope_func = ScopeExp {
                                str_name: name,
                                name_len: strlen(name) as u16,
                                presentation_hint: presentation_hint,
                                presentation_hint_len: strlen(presentation_hint) as u16,
                                variables:  vec.as_mut_ptr(),
                                variables_len: vec.len() as u16,
                                variables_reference: cur_variables_referenceID as i32
                            };

                            cur_variables_referenceID += 1;

                            let mut vec_scope:Vec<ScopeExp> = Vec::with_capacity(1);
                            vec_scope.push(scope_func);

                            let mut vec_scopes:Vec<ScopesExp> = Vec::with_capacity(1);
                            let mut scopes = ScopesExp{
                                scopes:  vec_scope.as_mut_ptr(),
                                count: vec_scope.len() as i32,
                            };
                            vec_scopes.push(scopes);

                            let str_func_name = CString::new(func.name.clone()).unwrap().into_raw();
                            let mut stack = StackFrameExp {
                                id: frameID as i32,
                                scopes: vec_scopes.as_mut_ptr(),
                                scopes_count: vec_scopes.len() as i32,
                                line: func.line_start as i32,
                                column: 0,
                                str_name: str_func_name,
                                name_len:  strlen(str_func_name) as u16,
                            };
                            frameID +=1;

                            add_stackframe(&mut stack);
                        }
                    }

                }


            }
        });

        //tx.send(123).unwrap();
    }


}
