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

// Compilation
#![allow(clippy::module_inception)]
#![cfg_attr(test, allow(deprecated))]
#![deny(unused_import_braces, unused_qualifications, trivial_casts, trivial_numeric_casts)]
#![deny(unused_qualifications, variant_size_differences, stable_features, unreachable_pub)]
#![deny(non_shorthand_field_patterns, unused_attributes, unused_extern_crates)]
#![deny(
    renamed_and_removed_lints,
    stable_features,
    unused_allocation,
    unused_comparisons,
    bare_trait_objects
)]
#![deny(
    const_err,
    unused_must_use,
    unused_mut,
    unused_unsafe,
    private_in_public,
    unsafe_code
)]
#![forbid(unsafe_code)]
// Documentation
//#![cfg_attr(nightly, feature(doc_cfg, external_doc))]
// TODO (howardwu): Reenable after completing documentation in snarkVM-models.
// #![cfg_attr(nightly, warn(missing_docs))]

// once rust 1.54 is release
// #![doc = include_str!("../documentation/the_aleo_curves/00_overview.md")]
#![cfg_attr(nightly, doc(include = "../documentation/the_aleo_curves/00_overview.md"))]

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate thiserror;

pub mod bls12_377;

pub mod bw6_761;

pub mod edwards_bls12;

pub mod edwards_bw6;

pub mod errors;
pub use errors::*;

pub mod templates;

#[cfg_attr(test, macro_use)]
pub mod traits;
pub use traits::*;
