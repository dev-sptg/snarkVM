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

use crate::{prelude::*, Parameters, Transaction};
use snarkvm_algorithms::CRH;
use snarkvm_utilities::{to_bytes_le, FromBytes, ToBytes};

use anyhow::Result;
use std::io::{Read, Result as IoResult, Write};

/// The transaction kernel contains core (public) transaction components,
/// A signed transaction kernel implies the caller has authorized the execution
/// of the transaction, and implies these values are admissibleby the ledger.
#[derive(Derivative)]
#[derivative(
    Clone(bound = "C: Parameters"),
    Debug(bound = "C: Parameters"),
    PartialEq(bound = "C: Parameters"),
    Eq(bound = "C: Parameters")
)]
pub struct TransactionKernel<C: Parameters> {
    /// The network ID.
    pub network_id: u8,
    /// The serial numbers of the input records.
    pub serial_numbers: Vec<C::AccountSignaturePublicKey>,
    /// The commitments of the output records.
    pub commitments: Vec<C::RecordCommitment>,
    /// A value balance is the difference between the input and output record values.
    pub value_balance: AleoAmount,
    /// Publicly-visible data associated with the transaction.
    pub memo: <Transaction<C> as TransactionScheme>::Memo,
}

impl<C: Parameters> TransactionKernel<C> {
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.network_id == C::NETWORK_ID
            && self.serial_numbers.len() == C::NUM_INPUT_RECORDS
            && self.commitments.len() == C::NUM_OUTPUT_RECORDS
    }

    #[inline]
    pub fn to_signature_message(&self) -> Result<Vec<u8>> {
        match self.is_valid() {
            true => self.to_bytes_le(),
            false => {
                Err(DPCError::InvalidKernel(self.network_id, self.serial_numbers.len(), self.commitments.len()).into())
            }
        }
    }

    #[inline]
    pub fn to_output_serial_number_nonces(&self) -> Result<Vec<C::SerialNumberNonce>> {
        // Ensure the kernel is well-formed before computing the output serial number nonces.
        if !self.is_valid() {
            return Err(
                DPCError::InvalidKernel(self.network_id, self.serial_numbers.len(), self.commitments.len()).into(),
            );
        }

        // Compute the joint serial numbers.
        let mut joint_serial_numbers = vec![];
        for serial_number in self.serial_numbers.iter().take(C::NUM_INPUT_RECORDS) {
            joint_serial_numbers.extend_from_slice(&to_bytes_le![serial_number]?);
        }

        // Compute the output serial number nonces.
        let mut output_serial_number_nonces = Vec::with_capacity(C::NUM_OUTPUT_RECORDS);
        for i in 0..C::NUM_OUTPUT_RECORDS {
            let position = (C::NUM_INPUT_RECORDS + i) as u8;
            let serial_number_nonce =
                C::serial_number_nonce_crh().hash(&to_bytes_le![position, joint_serial_numbers]?)?;
            output_serial_number_nonces.push(serial_number_nonce);
        }

        Ok(output_serial_number_nonces)
    }
}

impl<C: Parameters> ToBytes for TransactionKernel<C> {
    #[inline]
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        // Ensure the correct number of serial numbers and commitments are provided.
        if !self.is_valid() {
            return Err(
                DPCError::InvalidKernel(self.network_id, self.serial_numbers.len(), self.commitments.len()).into(),
            );
        }

        self.network_id.write_le(&mut writer)?;
        self.serial_numbers.write_le(&mut writer)?;
        self.commitments.write_le(&mut writer)?;
        self.value_balance.write_le(&mut writer)?;
        self.memo.write_le(&mut writer)
    }
}

impl<C: Parameters> FromBytes for TransactionKernel<C> {
    #[inline]
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let network_id: u8 = FromBytes::read_le(&mut reader)?;

        let mut serial_numbers = Vec::<C::AccountSignaturePublicKey>::with_capacity(C::NUM_INPUT_RECORDS);
        for _ in 0..C::NUM_INPUT_RECORDS {
            serial_numbers.push(FromBytes::read_le(&mut reader)?);
        }

        let mut commitments = Vec::<C::RecordCommitment>::with_capacity(C::NUM_OUTPUT_RECORDS);
        for _ in 0..C::NUM_OUTPUT_RECORDS {
            commitments.push(FromBytes::read_le(&mut reader)?);
        }

        let value_balance: AleoAmount = FromBytes::read_le(&mut reader)?;
        let memo: <Transaction<C> as TransactionScheme>::Memo = FromBytes::read_le(&mut reader)?;

        Ok(Self {
            network_id,
            serial_numbers,
            commitments,
            value_balance,
            memo,
        })
    }
}