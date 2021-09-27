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

use super::*;

pub const TO_BYTES_CORE: &str = "to_bytes";

impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_to_bytes(&mut self, arguments: &[ConstrainedValue<F, G>]) -> Result<ConstrainedValue<F, G>> {
        let bytes = match arguments.get(0) {
            None => Err(anyhow!("illegal `to_bytes` call, expected call on target")),
            Some(value) => value.to_bytes(),
        }?;

        Ok(ConstrainedValue::Array(
            bytes
                .into_iter()
                .map(Integer::U8)
                .map(ConstrainedValue::Integer)
                .collect(),
        ))
    }
}

pub const FROM_ADDRESS_BYTES_CORE: &str = "address_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_address_from_bytes(
        &mut self,
        arguments: &[ConstrainedValue<F, G>],
    ) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_BOOL_BYTES_CORE: &str = "bool_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_bool_from_bytes(
        &mut self,
        arguments: &[ConstrainedValue<F, G>],
    ) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_CHAR_BYTES_CORE: &str = "char_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_char_from_bytes(
        &mut self,
        arguments: &[ConstrainedValue<F, G>],
    ) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_FIELD_BYTES_CORE: &str = "field_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_field_from_bytes(
        &mut self,
        arguments: &[ConstrainedValue<F, G>],
    ) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_GROUP_BYTES_CORE: &str = "group_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_group_from_bytes(
        &mut self,
        arguments: &[ConstrainedValue<F, G>],
    ) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_I8_BYTES_CORE: &str = "i8_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_i8_from_bytes(&mut self, arguments: &[ConstrainedValue<F, G>]) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_I16_BYTES_CORE: &str = "i16_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_i16_from_bytes(&mut self, arguments: &[ConstrainedValue<F, G>]) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}
pub const FROM_I32_BYTES_CORE: &str = "i32_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_i32_from_bytes(&mut self, arguments: &[ConstrainedValue<F, G>]) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_I64_BYTES_CORE: &str = "i64_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_i64_from_bytes(&mut self, arguments: &[ConstrainedValue<F, G>]) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_I128_BYTES_CORE: &str = "i128_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_i128_from_bytes(
        &mut self,
        arguments: &[ConstrainedValue<F, G>],
    ) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_U8_BYTES_CORE: &str = "u8_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_u8_from_bytes(&mut self, arguments: &[ConstrainedValue<F, G>]) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_U16_BYTES_CORE: &str = "u16_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_u16_from_bytes(&mut self, arguments: &[ConstrainedValue<F, G>]) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_U32_BYTES_CORE: &str = "u32_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_u32_from_bytes(&mut self, arguments: &[ConstrainedValue<F, G>]) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_U64_BYTES_CORE: &str = "u64_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_u64_from_bytes(&mut self, arguments: &[ConstrainedValue<F, G>]) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}

pub const FROM_U128_BYTES_CORE: &str = "u128_from_bytes";
impl<'a, F: PrimeField, G: GroupType<F>, CS: ConstraintSystem<F>> EvaluatorState<'a, F, G, CS> {
    pub fn call_core_u128_from_bytes(
        &mut self,
        arguments: &[ConstrainedValue<F, G>],
    ) -> Result<ConstrainedValue<F, G>> {
        let arg = match arguments.get(0) {
            None => Err(anyhow!("illegal `from_bytes` call, expected 1 argument")),
            Some(value) => Ok(value),
        }?;
        let bytes = unwrap_u8_array_argument(arg);

        ConstrainedValue::from_bytes(&bytes)
    }
}
