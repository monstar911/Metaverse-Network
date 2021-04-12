// This file is part of Bit.Country.

// Copyright (C) 2020-2021 Bit.Country.
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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    traits::Get, StorageMap, StorageValue,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use primitives::{Balance, CountryId, CurrencyId};
use sp_runtime::{
    traits::{AccountIdConversion, One, Zero},
    DispatchError, ModuleId, RuntimeDebug,
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use bc_continuum::{SessionIndex, SpotId, ActiveSessionId};
pub use pallet::*;
use crate::pallet::{Config, Pallet, ActiveAuctionSlots};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ContinuumAuctionSlotStatus {
    /// Accept participation
    AcceptParticipates,
    /// Progressing at Good Neighborhood Protocol
    GNPStarted,
    /// Auction confirmed
    GNPConfirmed,
}

/// Information of EOI on Continuum spot
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct SpotEOI<AccountId> {
    spot_id: SpotId,
    participants: Vec<AccountId>,
}

/// Information of an active auction slot
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct AuctionSlot<BlockNumber, AccountId> {
    spot_id: SpotId,
    participants: Vec<AccountId>,
    active_session_index: BlockNumber,
    status: ContinuumAuctionSlotStatus,
}

#[frame_support::pallet]
pub mod pallet {
    use sp_std::vec::Vec;
    use frame_system::pallet_prelude::*;
    use frame_support::pallet_prelude::*;
    use sp_runtime::traits::Zero;
    use bc_continuum::*;
    use crate::{AuctionSlot, SpotEOI};

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// New Slot Duration
        /// How long the new auction slot will be released. If set to zero, no new auctions are generated
        type SessionDuration: Get<Self::BlockNumber>;
        /// Auction Slot Chilling Duration
        /// How long the participates in the New Auction Slots will get confirmed by neighbours
        type SpotAuctionChillingDuration: Get<Self::BlockNumber>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Initialization
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let auction_duration = T::SessionDuration::get();
            if !auction_duration.is_zero() && (now % auction_duration).is_zero() {
                Self::rotate_auction_slots(now);
                T::BlockWeights::get().max_block
            } else {
                0
            }
        }
    }

    /// Get current active session
    #[pallet::storage]
    #[pallet::getter(fn current_session)]
    pub type CurrentIndex<T: Config> = StorageValue<_, ActiveSessionId, ValueQuery>;

    /// Active Auction Slots of current session index that accepting participants
    #[pallet::storage]
    #[pallet::getter(fn get_active_auction_slots)]
    pub type ActiveAuctionSlots<T: Config> = StorageMap<_, Twox64Concat, SessionIndex, Vec<AuctionSlot<T::BlockNumber, T::AccountId>>, OptionQuery>;

    /// Active Auction Slots that is currently conducting GN Protocol
    #[pallet::storage]
    #[pallet::getter(fn get_active_gnp_slots)]
    pub type GNPSlots<T: Config> = StorageMap<_, Twox64Concat, SessionIndex, Vec<AuctionSlot<T::BlockNumber, T::AccountId>>, OptionQuery>;

    /// Active set of EOI on Continuum Spot
    #[pallet::storage]
    #[pallet::getter(fn get_eoi_set)]
    pub type EOISlots<T: Config> = StorageMap<_, Twox64Concat, SessionIndex, Vec<SpotEOI<T::AccountId>>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        //New express of interest
        NewExpressOfInterestAdded(T::AccountId, SpotId),
    }


    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_interest(origin: OriginFor<T>, spot_id: SpotId) -> DispatchResultWithPostInfo {
            /// Ensure AccountId own a country
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn vote_bidder_rejection(origin: OriginFor<T>, bidders: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_max_bounds(origin: OriginFor<T>, new_bound: (u64, u64)) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_new_auction_rate(origin: OriginFor<T>, new_rate: u32) -> DispatchResultWithPostInfo {
            Ok(().into())
        }
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn bid(origin: OriginFor<T>, id: SpotId, value: T::Balance) -> DispatchResultWithPostInfo {
        //     Ok(().into())
        // }
    }
}

impl<T: Config> Pallet<T> {
    fn rotate_auction_slots(now: T::BlockNumber) -> DispatchResult {
        ///Get current active session
        let current_active_session_id = CurrentIndex::<T>::get();

        ///Change status of all current active auction slots
        let active_auction_slots = <ActiveAuctionSlots<T>>::get(&current_active_session_id);

        ///Move current auctions slot to start GN Protocol
        let started_gnp_auction_slots: Vec<AuctionSlot<T::BlockNumber, T::AccountId>> =
            active_auction_slots.iter()
                .map(|x| x.status == ContinuumAuctionSlotStatus::GNPStarted)
                .collect();

        GNPSlots::<T>::insert(now, started_gnp_auction_slots);
        ActiveAuctionSlots::<T>::remove(&current_active_session_id);

        /// Get active EOI and add the top N to new Auction Slots
        let mut current_eoi_slots = EOISlots::<T>::get(current_active_session_id);

        let get_top_n_ranking = &current_eoi_slots.sort_by_key(|eoi_slot| eoi_slot.participants.len());

        Ok(().into())
    }

    fn do_register(who: &T::AccountId, spot_id: &SpotId) -> SpotId {
        return 5;
    }
}
