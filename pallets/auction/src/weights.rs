// This file is part of Metaverse.Network & Bit.Country.

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for auction
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-31, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/metaverse-node
// benchmark
// --pallet=auction
// --extrinsic=*
// --steps=20
// --repeat=10
// --execution=wasm
// --wasm-execution=compiled
// --template=./template/weight-template.hbs
// --output
// ./pallets/auction/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for auction.
pub trait WeightInfo {	fn create_new_auction() -> Weight;	fn create_new_buy_now() -> Weight;	fn bid() -> Weight;	fn buy_now() -> Weight;	fn authorise_metaverse_collection() -> Weight;	fn remove_authorise_metaverse_collection() -> Weight;	fn on_finalize() -> Weight;}

/// Weights for auction using the for collator node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn create_new_auction() -> Weight {
		(306_300_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(9 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	fn create_new_buy_now() -> Weight {
		(385_600_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(9 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	fn bid() -> Weight {
		(256_800_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn buy_now() -> Weight {
		(593_400_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(10 as Weight))
	}
	fn authorise_metaverse_collection() -> Weight {
		(91_900_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_authorise_metaverse_collection() -> Weight {
		(116_300_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn on_finalize() -> Weight {
		(50_100_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {	fn create_new_auction() -> Weight {
		(306_300_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(9 as Weight))			.saturating_add(RocksDbWeight::get().writes(7 as Weight))	}	fn create_new_buy_now() -> Weight {
		(385_600_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(9 as Weight))			.saturating_add(RocksDbWeight::get().writes(7 as Weight))	}	fn bid() -> Weight {
		(256_800_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn buy_now() -> Weight {
		(593_400_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(8 as Weight))			.saturating_add(RocksDbWeight::get().writes(10 as Weight))	}	fn authorise_metaverse_collection() -> Weight {
		(91_900_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn remove_authorise_metaverse_collection() -> Weight {
		(116_300_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(1 as Weight))	}	fn on_finalize() -> Weight {
		(50_100_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(1 as Weight))	}}
