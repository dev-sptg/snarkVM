// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use std::{borrow::Cow, marker::PhantomData};

use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use snarkvm_fields::PrimeField;
use snarkvm_gadgets::{Boolean, CondSelectGadget};
use snarkvm_ir::{Input as IrInput, InputData, Instruction, Program, Type, Value};
use snarkvm_r1cs::ConstraintSystem;

use std::sync::{Arc, Condvar, Mutex};



use crate::{
    bool_from_input,
    errors::{GroupError, ValueError},
    Address,
    Char,
    ConstrainedValue,
    Evaluator,
    FieldType,
    GroupType,
    Integer,
};

mod instruction;
mod state;

pub use instruction::*;
use state::*;

/// An evaluator for filling out a R1CS while also producing an expected output.
pub struct SetupEvaluator<F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> {
    cs: CS,
    _p: PhantomData<(F, G)>,
    pair:Arc<(Mutex<bool>, Condvar)>
    //start_signal: Arc<SignalEvent>//::new(SignalEvent::new(false, SignalKind::Manual));
}

impl<F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> SetupEvaluator<F, G, CS> {
    pub fn new(cs: CS) -> Self {
        Self {
            cs,
            _p: PhantomData,
            //start_signal: Arc::new(SignalEvent::new(false, SignalKind::Manual))
            pair: Arc::new((Mutex::new(false), Condvar::new()))
        }
    }
}


extern crate libloading as lib;
use std::ffi::{c_void, CString};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr::{copy_nonoverlapping, null, null_mut};
use std::io;
use std::env;
use std::fmt::Display;
use std::path::PathBuf;
use std::ptr;


use std::thread;
use std::time::Duration;



#[repr(C)]
struct RustObject {
    main_file_path: PathBuf
}

type RunServer = fn() -> i32;
type RegisterCallback = fn (target: *mut RustObject, cb: extern fn(target: *mut RustObject, *mut  c_char, i32));
type AddScopes = fn (scopes: *mut ScopesExp);

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

impl<F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> Evaluator<F, G> for SetupEvaluator<F, G, CS> {
    type Error = anyhow::Error;
    type Output = ConstrainedValue<F, G>;
    //pair = Arc::new((Mutex::new(false), Condvar::new()));

    fn inner_main(str: &str) -> io::Result<PathBuf> {
        let mut dir = env::current_exe()?;
        dir.pop();
        dir.push(str);
        Ok(dir)
    }

    fn run_dap(&mut self, input: &InputData) -> Boolean {

        //let start = self.start_signal.clone();
        let pair2 = Arc::clone(&self.pair);
        let (lock, cvar) = &*pair2;

        let registers = input.registers.clone();
        let main_input = input.main.clone();
        let debug_data = input.debug_data.clone();

        thread::spawn(move || {
            println!("Load library hello_debugger.dll");
            let pathSo = Self::inner_main("debugger.dll").expect("Couldn't");
            let lib = lib::Library::new(pathSo).unwrap();

            unsafe {
                let run_server: lib::Symbol<RunServer> =  lib.get(b"run_server").unwrap();
                let register_callback: lib::Symbol<RegisterCallback> = lib.get(b"register_callback").unwrap();
                let add_scopes: lib::Symbol<AddScopes> = lib.get(b"add_scopes").unwrap();

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
                };

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
                    main_file_path: debug_data
                });

                register_callback(&mut *rust_object,  callback);


                add_scopes(&mut scopes);

                println!("Rust: run_server");
                run_server();
            }
        });
        Boolean::Constant(true)
    }

    fn evaluate(&mut self, program: &Program, input: &InputData) -> Result<Self::Output, Self::Error> {
        let mut state = EvaluatorState::new(program);
        self.run_dap(input);

        //self.start_signal.wait();
        let (lock, cvar) = &* self.pair;
        let mut started = lock.lock().unwrap();
        cvar.wait(started);
        state.handle_input_block("main", &program.header.main_inputs, &input.main, &mut self.cs)?;
        state.handle_const_input_block(&program.header.constant_inputs, &input.constants, &mut self.cs)?;
        state.handle_input_block(
            "register",
            &program.header.register_inputs,
            &input.registers,
            &mut self.cs,
        )?;
        state.handle_input_block(
            "public_states",
            &program.header.public_states,
            &input.public_states,
            &mut self.cs,
        )?;
        state.handle_input_block(
            "private_record_states",
            &program.header.private_record_states,
            &input.private_record_states,
            &mut self.cs,
        )?;
        state.handle_input_block(
            "private_leaf_states",
            &program.header.private_leaf_states,
            &input.private_leaf_states,
            &mut self.cs,
        )?;
        let function = state.setup_evaluate_function(0, &[])?;
        let output = FunctionEvaluator::evaluate_function(function, state, 0, &mut self.cs)?; // arguments assigned via input system for entrypoint
        Ok(output)
    }
}
