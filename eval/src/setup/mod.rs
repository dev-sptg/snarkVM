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
use snarkvm_debugger::Debugger;


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
    pair:Arc<(Mutex<bool>, Condvar)>,

    //debugger: Debugger
    //start_signal: Arc<SignalEvent>//::new(SignalEvent::new(false, SignalKind::Manual));
}

impl<F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> SetupEvaluator<F, G, CS> {
    pub fn new(cs: CS) -> Self {
        Self {
            cs,
            _p: PhantomData,
            //start_signal: Arc::new(SignalEvent::new(false, SignalKind::Manual))
            pair: Arc::new((Mutex::new(false), Condvar::new())),
            //debugger: Debugger::new()
        }
    }
}

/*
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
    main_file_path: PathBuf,
    mutex_pair:Arc<(Mutex<bool>, Condvar)>
}

type RunServer = fn() -> i32;
type RegisterCallback = fn (target: *mut RustObject, cb: extern fn(target: *mut RustObject, *mut  c_char, i32));
type RegisterNextStep = fn (target: *mut RustObject, cb: extern fn(target: *mut RustObject));
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
}*/

impl<F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> Evaluator<F, G> for SetupEvaluator<F, G, CS> {
    type Error = anyhow::Error;
    type Output = ConstrainedValue<F, G>;

    fn evaluate(&mut self, program: &Program, input: &InputData) -> Result<Self::Output, Self::Error> {
        let mut debugger = Debugger::new(program.header.debug_data.clone(), program);
        debugger.run_debugger(input);

        let mut state = EvaluatorState::new( program);
        //self.debugger.run_debugger(input);
        //debugger.run_debugger(input);
        //self.run_dap(input);

        //self.start_signal.wait();

        debugger.wait_for_next_step();
        debugger.set_variable_value(0, 18, "123".to_string());
        debugger.send_stack_frame(0);
        debugger.wait_for_next_step();

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
        let output = FunctionEvaluator::evaluate_function(&mut debugger, function, state, 0, &mut self.cs)?; // arguments assigned via input system for entrypoint
        Ok(output)
    }
}
