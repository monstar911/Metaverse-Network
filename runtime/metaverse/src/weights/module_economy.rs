//! Autogenerated weights for `economy`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-26, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Mac-mini.local`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/metaverse-node
// benchmark
// pallet
// --pallet=economy
// --extrinsic=*
// --steps=20
// --repeat=10
// --execution=wasm
// --wasm-execution=compiled
// --output
// ./pallets/economy/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `economy`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> economy::WeightInfo for WeightInfo<T> {
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Economy BitPowerExchangeRate (r:0 w:1)
	fn set_bit_power_exchange_rate() -> Weight {
		(12_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Economy PowerBalance (r:0 w:1)
	fn set_power_balance() -> Weight {
		(12_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Mining Round (r:1 w:0)
	// Storage: Economy ExitQueue (r:1 w:0)
	// Storage: Economy StakingInfo (r:1 w:1)
	// Storage: Economy TotalStake (r:1 w:1)
	fn stake_a() -> Weight {
		(29_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Mining Round (r:1 w:0)
	// Storage: Economy EstateExitQueue (r:1 w:0)
	// Storage: Estate Estates (r:1 w:0)
	// Storage: Estate EstateOwner (r:1 w:0)
	// Storage: OrmlNFT Tokens (r:1 w:0)
	// Storage: Economy EstateStakingInfo (r:1 w:1)
	// Storage: Economy TotalEstateStake (r:1 w:1)
	fn stake_b() -> Weight {
		(39_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Economy StakingInfo (r:1 w:1)
	// Storage: Mining Round (r:1 w:0)
	// Storage: Economy ExitQueue (r:1 w:1)
	// Storage: Economy TotalStake (r:1 w:1)
	fn unstake_a() -> Weight {
		(20_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Estate Estates (r:1 w:0)
	// Storage: Estate EstateOwner (r:1 w:0)
	// Storage: OrmlNFT Tokens (r:1 w:0)
	// Storage: Economy EstateStakingInfo (r:1 w:1)
	// Storage: Mining Round (r:1 w:0)
	// Storage: Economy EstateExitQueue (r:1 w:1)
	// Storage: Economy TotalEstateStake (r:1 w:1)
	fn unstake_b() -> Weight {
		(30_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
	// Storage: Economy ExitQueue (r:1 w:1)
	fn withdraw_unreserved() -> Weight {
		(22_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
