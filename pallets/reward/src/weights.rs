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

//! Autogenerated weights for reward
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-13, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/metaverse-node
// benchmark
// pallet
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// reward
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --template=./template/weight-template.hbs
// --output
// ./pallets/reward/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for reward.
pub trait WeightInfo {	fn create_campaign() -> Weight;	fn create_nft_campaign() -> Weight;	fn claim_reward() -> Weight;	fn claim_nft_reward() -> Weight;	fn set_reward() -> Weight;	fn set_nft_reward() -> Weight;	fn close_campaign() -> Weight;	fn close_nft_campaign() -> Weight;	fn cancel_campaign() -> Weight;	fn cancel_nft_campaign() -> Weight;	fn add_set_reward_origin() -> Weight;	fn remove_set_reward_origin() -> Weight;	fn on_finalize() -> Weight;}

/// Weights for reward using the for collator node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {	fn create_campaign() -> Weight {
		(40_885_000 as Weight)			.saturating_add(T::DbWeight::get().reads(4 as Weight))			.saturating_add(T::DbWeight::get().writes(5 as Weight))	}	fn create_nft_campaign() -> Weight {
		(38_036_000 as Weight)			.saturating_add(T::DbWeight::get().reads(5 as Weight))			.saturating_add(T::DbWeight::get().writes(6 as Weight))	}	fn claim_reward() -> Weight {
		(35_175_000 as Weight)			.saturating_add(T::DbWeight::get().reads(4 as Weight))			.saturating_add(T::DbWeight::get().writes(4 as Weight))	}	fn claim_nft_reward() -> Weight {
		(36_767_000 as Weight)			.saturating_add(T::DbWeight::get().reads(6 as Weight))			.saturating_add(T::DbWeight::get().writes(4 as Weight))	}	fn set_reward() -> Weight {
		(19_834_000 as Weight)			.saturating_add(T::DbWeight::get().reads(4 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}	fn set_nft_reward() -> Weight {
		(18_319_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}	fn close_campaign() -> Weight {
		(40_841_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(4 as Weight))	}	fn close_nft_campaign() -> Weight {
		(34_413_000 as Weight)			.saturating_add(T::DbWeight::get().reads(4 as Weight))			.saturating_add(T::DbWeight::get().writes(5 as Weight))	}	fn cancel_campaign() -> Weight {
		(79_924_000 as Weight)			.saturating_add(T::DbWeight::get().reads(3 as Weight))			.saturating_add(T::DbWeight::get().writes(3 as Weight))	}	fn cancel_nft_campaign() -> Weight {
		(33_118_000 as Weight)			.saturating_add(T::DbWeight::get().reads(4 as Weight))			.saturating_add(T::DbWeight::get().writes(4 as Weight))	}	fn add_set_reward_origin() -> Weight {
		(11_644_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn remove_set_reward_origin() -> Weight {
		(12_048_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))			.saturating_add(T::DbWeight::get().writes(2 as Weight))	}	fn on_finalize() -> Weight {
		(13_786_000 as Weight)			.saturating_add(T::DbWeight::get().reads(2 as Weight))	}}

// For backwards compatibility and tests
impl WeightInfo for () {	fn create_campaign() -> Weight {
		(40_885_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(4 as Weight))			.saturating_add(RocksDbWeight::get().writes(5 as Weight))	}	fn create_nft_campaign() -> Weight {
		(38_036_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(5 as Weight))			.saturating_add(RocksDbWeight::get().writes(6 as Weight))	}	fn claim_reward() -> Weight {
		(35_175_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(4 as Weight))			.saturating_add(RocksDbWeight::get().writes(4 as Weight))	}	fn claim_nft_reward() -> Weight {
		(36_767_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(6 as Weight))			.saturating_add(RocksDbWeight::get().writes(4 as Weight))	}	fn set_reward() -> Weight {
		(19_834_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(4 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn set_nft_reward() -> Weight {
		(18_319_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn close_campaign() -> Weight {
		(40_841_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(4 as Weight))	}	fn close_nft_campaign() -> Weight {
		(34_413_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(4 as Weight))			.saturating_add(RocksDbWeight::get().writes(5 as Weight))	}	fn cancel_campaign() -> Weight {
		(79_924_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(3 as Weight))			.saturating_add(RocksDbWeight::get().writes(3 as Weight))	}	fn cancel_nft_campaign() -> Weight {
		(33_118_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(4 as Weight))			.saturating_add(RocksDbWeight::get().writes(4 as Weight))	}	fn add_set_reward_origin() -> Weight {
		(11_644_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn remove_set_reward_origin() -> Weight {
		(12_048_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))			.saturating_add(RocksDbWeight::get().writes(2 as Weight))	}	fn on_finalize() -> Weight {
		(13_786_000 as Weight)			.saturating_add(RocksDbWeight::get().reads(2 as Weight))	}}
