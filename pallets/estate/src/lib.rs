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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_support::{dispatch::DispatchResult, ensure, traits::Get, PalletId};
use frame_system::pallet_prelude::*;
use frame_system::{ensure_root, ensure_signed};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AccountIdConversion, One, Saturating},
	ArithmeticError, DispatchError,
};
use sp_std::vec::Vec;

use auction_manager::{Auction, CheckAuctionItemHandler};
use core_primitives::*;
pub use pallet::*;
use primitives::estate::EstateInfo;
use primitives::{
	estate::Estate, estate::LandUnitStatus, estate::OwnerId, Attributes, ClassId, EstateId, ItemId, MetaverseId,
	NftMetadata, TokenId, UndeployedLandBlock, UndeployedLandBlockId, UndeployedLandBlockType,
};
pub use rate::{MintingRateInfo, Range};
pub use weights::WeightInfo;

//#[cfg(feature = "runtime-benchmarks")]
//pub mod benchmarking;

#[cfg(test)]
mod mock;
mod rate;

#[cfg(test)]
mod tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::{Currency, Imbalance, ReservableCurrency};
	use sp_runtime::traits::{CheckedAdd, CheckedSub, Zero};

	use primitives::estate::EstateInfo;
	use primitives::staking::{Bond, RoundInfo, StakeSnapshot};
	use primitives::{RoundIndex, UndeployedLandBlockId};

	use crate::rate::{round_issuance_range, MintingRateInfo};

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Land treasury source
		#[pallet::constant]
		type LandTreasury: Get<PalletId>;

		/// Source of metaverse info
		type MetaverseInfoSource: MetaverseTrait<Self::AccountId>;

		/// Currency type
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Minimum land price
		type MinimumLandPrice: Get<BalanceOf<Self>>;

		/// Council origin which allows to update max bound
		type CouncilOrigin: EnsureOrigin<Self::Origin>;

		/// Auction handler
		type AuctionHandler: Auction<Self::AccountId, Self::BlockNumber> + CheckAuctionItemHandler<BalanceOf<Self>>;

		/// Minimum number of blocks per round
		#[pallet::constant]
		type MinBlocksPerRound: Get<u32>;

		/// Weight implementation for estate extrinsics
		type WeightInfo: WeightInfo;

		/// Minimum staking balance
		#[pallet::constant]
		type MinimumStake: Get<BalanceOf<Self>>;

		/// Delay of staking reward payment (in number of rounds)
		#[pallet::constant]
		type RewardPaymentDelay: Get<u32>;

		/// NFT trait required for land and estate tokenization
		type NFTTokenizationSource: NFTTrait<Self::AccountId, BalanceOf<Self>, ClassId = ClassId, TokenId = TokenId>;

		/// Default max bound for each metaverse mapping system, this could change through proposal
		type DefaultMaxBound: Get<(i32, i32)>;
	}

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::storage]
	#[pallet::getter(fn all_land_units_count)]
	/// Track the total number of land units
	pub(super) type AllLandUnitsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_undeployed_land_unit)]
	/// Track the total of undeployed land units
	pub(super) type TotalUndeployedLandUnit<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_land_units)]
	/// Index land owners by metaverse ID and coordinate
	pub type LandUnits<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		MetaverseId,
		Twox64Concat,
		(i32, i32),
		OwnerId<T::AccountId, TokenId>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_estate_id)]
	/// Track the next estate ID
	pub type NextEstateId<T: Config> = StorageValue<_, EstateId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_estates_count)]
	/// Track the total of estates
	pub(super) type AllEstatesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_estates)]
	/// Store estate information  
	pub(super) type Estates<T: Config> = StorageMap<_, Twox64Concat, EstateId, EstateInfo, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_estate_owner)]
	/// Track estate owners
	pub type EstateOwner<T: Config> =
		StorageMap<_, Twox64Concat, EstateId, OwnerId<T::AccountId, TokenId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_undeployed_land_block_id)]
	/// Track the next undeployed land ID
	pub(super) type NextUndeployedLandBlockId<T: Config> = StorageValue<_, UndeployedLandBlockId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_undeployed_land_block)]
	/// Store undeployed land blocks
	pub(super) type UndeployedLandBlocks<T: Config> =
		StorageMap<_, Blake2_128Concat, UndeployedLandBlockId, UndeployedLandBlock<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_undeployed_land_block_owner)]
	/// Index undeployed land blocks by account ID
	pub type UndeployedLandBlocksOwner<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, UndeployedLandBlockId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	/// Current round index and next round scheduled transition
	pub type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn minting_rate_config)]
	/// Minting rate configuration
	pub type MintingRateConfig<T: Config> = StorageValue<_, MintingRateInfo, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_stake)]
	/// Total NEER locked by estate
	type TotalStake<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn staked)]
	/// Total backing stake for selected candidates in the round
	pub type Staked<T: Config> = StorageMap<_, Twox64Concat, RoundIndex, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn exit_queue)]
	/// A queue of account awaiting exit
	type ExitQueue<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, EstateId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	/// Snapshot of estate staking per session
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		EstateId,
		StakeSnapshot<T::AccountId, BalanceOf<T>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn estate_stake)]
	/// Estate staking
	pub type EstateStake<T: Config> =
		StorageDoubleMap<_, Twox64Concat, EstateId, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub minting_rate_config: MintingRateInfo,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig {
				minting_rate_config: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<MintingRateConfig<T>>::put(self.minting_rate_config.clone());

			// Start Round 1 at Block 0
			let round: RoundInfo<T::BlockNumber> = RoundInfo::new(1u32, 0u32.into(), T::MinBlocksPerRound::get());

			let round_issuance_per_round = round_issuance_range::<T>(self.minting_rate_config.clone());

			<Round<T>>::put(round);
			<Pallet<T>>::deposit_event(Event::NewRound(
				T::BlockNumber::zero(),
				1u32,
				round_issuance_per_round.max,
			));
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New lands are minted [Beneficial Account Id, Metaverse Id, Coordinates]
		NewLandsMinted(T::AccountId, MetaverseId, Vec<(i32, i32)>),
		/// Land unit is transferred [Metaverse Id, Coordinates, From Account Id, To Account Id]
		TransferredLandUnit(MetaverseId, (i32, i32), T::AccountId, T::AccountId),
		/// Estate unit is transferred [Estate Id, From Account Id, To Account Id]
		TransferredEstate(EstateId, T::AccountId, T::AccountId),
		/// New land is minted [Beneficial Account Id, Metaverse Id, Coordinates]
		NewLandUnitMinted(OwnerId<T::AccountId, TokenId>, MetaverseId, (i32, i32)),
		/// New estate is minted [Estate Id, OwnerId, Metaverse Id, Coordinates]
		NewEstateMinted(EstateId, OwnerId<T::AccountId, TokenId>, MetaverseId, Vec<(i32, i32)>),
		/// Max bound is set for a metaverse [Metaverse Id, Min and Max Coordinate]
		MaxBoundSet(MetaverseId, (i32, i32)),
		/// Land block is deployed [From Account Id, Metaverse Id, Undeployed Land Block Id,
		/// Coordinates]
		LandBlockDeployed(T::AccountId, MetaverseId, UndeployedLandBlockId, Vec<(i32, i32)>),
		/// Undeployed land block is issued [Beneficial Account Id, Undeployed Land
		/// Block Id]
		UndeployedLandBlockIssued(T::AccountId, UndeployedLandBlockId),
		/// Undeployed land block is transferred [From Account Id, To Account Id, Undeployed
		/// Land Block Id]
		UndeployedLandBlockTransferred(T::AccountId, T::AccountId, UndeployedLandBlockId),
		/// Undeployed land block is approved [Owner Account Id, Approved Account Id, Undeployed
		/// Land Block Id]
		UndeployedLandBlockApproved(T::AccountId, T::AccountId, UndeployedLandBlockId),
		/// Estate is destroyed [Estate Id, Owner Id]
		EstateDestroyed(EstateId, OwnerId<T::AccountId, TokenId>),
		/// Estate is updated [Estate Id, Owner Id, Coordinates]
		EstateUpdated(EstateId, OwnerId<T::AccountId, TokenId>, Vec<(i32, i32)>),
		/// Land unit is added to an estate [Estate Id, Owner Id, Coordinates]
		LandUnitAdded(EstateId, OwnerId<T::AccountId, TokenId>, Vec<(i32, i32)>),
		/// Land unit is removed from an estate [Estate Id, Owner Id, Coordinates]
		LandUnitsRemoved(EstateId, OwnerId<T::AccountId, TokenId>, Vec<(i32, i32)>),
		/// Undeployed land block is unapproved [Undeployed Land Block Id]
		UndeployedLandBlockUnapproved(UndeployedLandBlockId),
		/// Undeployed land block is freezed [Undeployed Land Block Id]
		UndeployedLandBlockFreezed(UndeployedLandBlockId),
		/// Undeployed land block is unfreezed [Undeployed Land Block Id]
		UndeployedLandBlockUnfreezed(UndeployedLandBlockId),
		/// Undeployed land block is burnt [Undeployed Land Block Id]
		UndeployedLandBlockBurnt(UndeployedLandBlockId),
		/// New staking round started [Starting Block, Round, Total Land Unit]
		NewRound(T::BlockNumber, RoundIndex, u64),
		/// Updated staking snapshot [Round Index, Balance]
		StakeSnapshotUpdated(RoundIndex, BalanceOf<T>),
		/// Stakers are paid for the round [Round Index]
		StakersPaid(RoundIndex),
		/// Exit queue is cleared [Round Index]
		ExitQueueCleared(RoundIndex),
		/// Increased stake on an estate [OwnerId, Estate Id, Balance]
		EstateStakeIncreased(OwnerId<T::AccountId, TokenId>, EstateId, BalanceOf<T>),
		/// Decreased stake on an estate [OwnerId, Estate Id, Balance]
		EstateStakeDecreased(OwnerId<T::AccountId, TokenId>, EstateId, BalanceOf<T>),
		/// Left staking on an estate [OwnerId, Estate Id]
		EstateStakeLeft(OwnerId<T::AccountId, TokenId>, EstateId),
		/// Account rewarded for staking [Account Id, Balance]
		StakingRewarded(T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No permission
		NoPermission,
		/// No available estate ID
		NoAvailableEstateId,
		/// Insufficient fund
		InsufficientFund,
		/// Estate ID already exist
		EstateIdAlreadyExist,
		/// Land unit is not available
		LandUnitIsNotAvailable,
		/// Land unit is out of bound
		LandUnitIsOutOfBound,
		/// Undeployed land block is not found
		UndeployedLandBlockNotFound,
		/// Undeployed land block is not transferable
		UndeployedLandBlockIsNotTransferable,
		/// Undeployed land block does not hae enough land units
		UndeployedLandBlockDoesNotHaveEnoughLandUnits,
		/// Number of land block credit and land unit does not match
		UndeployedLandBlockUnitAndInputDoesNotMatch,
		/// Already own the undeployed land block
		AlreadyOwnTheUndeployedLandBlock,
		/// Undeployed land block is freezed
		UndeployedLandBlockFreezed,
		/// Undeployed land block is already freezed
		UndeployedLandBlockAlreadyFreezed,
		/// Undeployed land block is not frozen
		UndeployedLandBlockNotFrozen,
		/// Already owning the estate
		AlreadyOwnTheEstate,
		/// Already owning the land unit
		AlreadyOwnTheLandUnit,
		/// Estate is not in auction
		EstateNotInAuction,
		/// Land unit is not in auction
		LandUnitNotInAuction,
		/// Estate is already in auction
		EstateAlreadyInAuction,
		/// Land unit is already in auction
		LandUnitAlreadyInAuction,
		/// Estate is does not exist
		EstateDoesNotExist,
		/// Land unit does not exist
		LandUnitDoesNotExist,
		/// Only frozen undeployed land block can be destroyed
		OnlyFrozenUndeployedLandBlockCanBeDestroyed,
		/// Below minimum staking amount
		BelowMinimumStake,
		/// Value overflow
		Overflow,
		/// Estate stake is already left
		EstateStakeAlreadyLeft,
		/// Account has not staked anything
		AccountHasNoStake,
		// Invalid owner value
		InvalidOwnerValue,
		// Coordinate for estate is not valid
		CoordinatesForEstateIsNotValid,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::mint_land())]
		pub fn mint_land(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			metaverse_id: MetaverseId,
			coordinate: (i32, i32),
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Mint land unit
			let owner = Self::mint_land_unit(metaverse_id, beneficiary, coordinate, LandUnitStatus::NonExisting)?;

			// Update total land count
			Self::set_total_land_unit(One::one(), false)?;

			Self::deposit_event(Event::<T>::NewLandUnitMinted(owner, metaverse_id, coordinate));

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint_lands())]
		pub fn mint_lands(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			metaverse_id: MetaverseId,
			coordinates: Vec<(i32, i32)>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Mint land units
			for coordinate in coordinates.clone() {
				Self::mint_land_unit(
					metaverse_id,
					beneficiary.clone(),
					coordinate,
					LandUnitStatus::NonExisting,
				)?;
			}

			// Update total land count
			Self::set_total_land_unit(coordinates.len() as u64, false)?;

			Self::deposit_event(Event::<T>::NewLandsMinted(
				beneficiary.clone(),
				metaverse_id.clone(),
				coordinates.clone(),
			));

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::transfer_land())]
		pub fn transfer_land(
			origin: OriginFor<T>,
			to: T::AccountId,
			metaverse_id: MetaverseId,
			coordinate: (i32, i32),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				!T::AuctionHandler::check_item_in_auction(ItemId::LandUnit(coordinate, metaverse_id)),
				Error::<T>::LandUnitAlreadyInAuction
			);

			Self::do_transfer_landunit(coordinate, &who, &to, metaverse_id)?;
			Ok(().into())
		}

		/// Mint new estate with no existing land unit
		#[pallet::weight(T::WeightInfo::mint_estate())]
		pub fn mint_estate(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			metaverse_id: MetaverseId,
			coordinates: Vec<(i32, i32)>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Generate new estate id
			let new_estate_id = Self::get_new_estate_id()?;

			// Generate sub account from estate
			let estate_account_id: T::AccountId = T::LandTreasury::get().into_sub_account(new_estate_id);

			// Mint land units
			for coordinate in coordinates.clone() {
				Self::mint_land_unit(
					metaverse_id,
					beneficiary.clone(),
					coordinate,
					LandUnitStatus::NonExisting,
				)?;
			}
			// Update total land count
			Self::set_total_land_unit(coordinates.len() as u64, false)?;

			// Update estate information
			Self::update_estate_information(new_estate_id, metaverse_id, &beneficiary, coordinates)?;
			Ok(().into())
		}

		/// Create new estate from existing land units
		#[pallet::weight(T::WeightInfo::create_estate())]
		pub fn create_estate(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			metaverse_id: MetaverseId,
			coordinates: Vec<(i32, i32)>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(
				Self::verify_land_unit_for_estate(coordinates.clone()),
				Error::<T>::CoordinatesForEstateIsNotValid
			);

			// Generate new estate id
			let new_estate_id = Self::get_new_estate_id()?;

			// Generate sub account from estate
			let estate_account_id: T::AccountId = T::LandTreasury::get().into_sub_account(new_estate_id);

			// Mint land units
			for coordinate in coordinates.clone() {
				Self::mint_land_unit(
					metaverse_id,
					estate_account_id.clone(),
					coordinate,
					LandUnitStatus::Existing(beneficiary.clone()),
				)?;
			}

			// Update estate information
			Self::update_estate_information(new_estate_id, metaverse_id, &beneficiary, coordinates.clone())?;

			Ok(().into())
		}

		/// Transfer estate ownership
		#[pallet::weight(T::WeightInfo::transfer_estate())]
		pub fn transfer_estate(
			origin: OriginFor<T>,
			to: T::AccountId,
			estate_id: EstateId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				!T::AuctionHandler::check_item_in_auction(ItemId::Estate(estate_id)),
				Error::<T>::EstateAlreadyInAuction
			);

			Self::do_transfer_estate(estate_id, &who, &to)?;

			Ok(().into())
		}

		/// Deploy raw land block to metaverse and turn raw land block to land unit with given
		/// coordinates
		#[pallet::weight(T::WeightInfo::deploy_land_block())]
		pub fn deploy_land_block(
			origin: OriginFor<T>,
			undeployed_land_block_id: UndeployedLandBlockId,
			metaverse_id: MetaverseId,
			land_block_coordinate: (i32, i32),
			coordinates: Vec<(i32, i32)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Ensure the max bound is set for the metaverse
			let max_bound = T::DefaultMaxBound::get();

			// Check whether the coordinate is within the bound
			ensure!(
				(land_block_coordinate.0 >= max_bound.0 && max_bound.1 >= land_block_coordinate.0)
					&& (land_block_coordinate.1 >= max_bound.0 && max_bound.1 >= land_block_coordinate.1),
				Error::<T>::LandUnitIsOutOfBound
			);

			ensure!(
				Self::verify_land_unit_in_bound(&land_block_coordinate, &coordinates),
				Error::<T>::LandUnitIsOutOfBound
			);

			UndeployedLandBlocks::<T>::try_mutate_exists(
				&undeployed_land_block_id,
				|undeployed_land_block| -> DispatchResultWithPostInfo {
					let mut undeployed_land_block_record = undeployed_land_block
						.as_mut()
						.ok_or(Error::<T>::UndeployedLandBlockNotFound)?;

					ensure!(
						undeployed_land_block_record.owner == who.clone(),
						Error::<T>::NoPermission
					);

					ensure!(
						undeployed_land_block_record.is_locked == false,
						Error::<T>::UndeployedLandBlockFreezed
					);

					let land_units_to_mint = coordinates.len() as u32;

					// Ensure undeployed land block only deployed once
					ensure!(
						undeployed_land_block_record.number_land_units == land_units_to_mint,
						Error::<T>::UndeployedLandBlockUnitAndInputDoesNotMatch
					);

					// Mint land units
					for coordinate in coordinates.clone() {
						Self::mint_land_unit(metaverse_id, who.clone(), coordinate, LandUnitStatus::NonExisting)?;
					}

					// Update total land count
					Self::set_total_land_unit(coordinates.len() as u64, false)?;

					// Burn undeployed land block
					Self::do_burn_undeployed_land_block(undeployed_land_block_id)?;

					Self::deposit_event(Event::<T>::LandBlockDeployed(
						who.clone(),
						metaverse_id,
						undeployed_land_block_id,
						coordinates,
					));

					Ok(().into())
				},
			)
		}

		/// Sudo issues new raw land block
		#[pallet::weight(T::WeightInfo::issue_undeployed_land_blocks())]
		pub fn issue_undeployed_land_blocks(
			who: OriginFor<T>,
			beneficiary: T::AccountId,
			number_of_land_block: u32,
			number_land_units_per_land_block: u32,
			undeployed_land_block_type: UndeployedLandBlockType,
		) -> DispatchResultWithPostInfo {
			ensure_root(who)?;

			Self::do_issue_undeployed_land_blocks(
				&beneficiary,
				number_of_land_block,
				number_land_units_per_land_block,
				undeployed_land_block_type,
			)?;

			Ok(().into())
		}

		/// Sudo Freeze raw land block
		#[pallet::weight(T::WeightInfo::freeze_undeployed_land_blocks())]
		pub fn freeze_undeployed_land_blocks(
			origin: OriginFor<T>,
			undeployed_land_block_id: UndeployedLandBlockId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			Self::do_freeze_undeployed_land_block(undeployed_land_block_id)?;

			Ok(().into())
		}

		/// Sudo Unfreeze raw land block
		#[pallet::weight(T::WeightInfo::unfreeze_undeployed_land_blocks())]
		pub fn unfreeze_undeployed_land_blocks(
			origin: OriginFor<T>,
			undeployed_land_block_id: UndeployedLandBlockId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			UndeployedLandBlocks::<T>::try_mutate_exists(
				&undeployed_land_block_id,
				|undeployed_land_block| -> DispatchResultWithPostInfo {
					let mut undeployed_land_block_record = undeployed_land_block
						.as_mut()
						.ok_or(Error::<T>::UndeployedLandBlockNotFound)?;

					ensure!(
						undeployed_land_block_record.is_locked == true,
						Error::<T>::UndeployedLandBlockNotFrozen
					);

					undeployed_land_block_record.is_locked = false;

					Self::deposit_event(Event::<T>::UndeployedLandBlockUnfreezed(undeployed_land_block_id));

					Ok(().into())
				},
			)
		}

		/// Transfer raw land block
		#[pallet::weight(T::WeightInfo::transfer_undeployed_land_blocks())]
		pub fn transfer_undeployed_land_blocks(
			origin: OriginFor<T>,
			to: T::AccountId,
			undeployed_land_block_id: UndeployedLandBlockId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::do_transfer_undeployed_land_block(&who, &to, undeployed_land_block_id)?;

			Ok(().into())
		}

		/// Burn raw land block that will reduce total supply
		#[pallet::weight(T::WeightInfo::burn_undeployed_land_blocks())]
		pub fn burn_undeployed_land_blocks(
			origin: OriginFor<T>,
			undeployed_land_block_id: UndeployedLandBlockId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			Self::do_burn_undeployed_land_block(undeployed_land_block_id)?;

			Ok(().into())
		}

		/// Burn raw land block that will reduce total supply
		#[pallet::weight(T::WeightInfo::approve_undeployed_land_blocks())]
		pub fn approve_undeployed_land_blocks(
			origin: OriginFor<T>,
			to: T::AccountId,
			undeployed_land_block_id: UndeployedLandBlockId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			UndeployedLandBlocks::<T>::try_mutate_exists(
				&undeployed_land_block_id,
				|undeployed_land_block| -> DispatchResultWithPostInfo {
					let mut undeployed_land_block_record = undeployed_land_block
						.as_mut()
						.ok_or(Error::<T>::UndeployedLandBlockNotFound)?;

					ensure!(
						undeployed_land_block_record.owner == who.clone(),
						Error::<T>::NoPermission
					);

					ensure!(
						undeployed_land_block_record.is_locked == false,
						Error::<T>::UndeployedLandBlockAlreadyFreezed
					);

					undeployed_land_block_record.approved = Some(to.clone());

					Self::deposit_event(Event::<T>::UndeployedLandBlockApproved(
						who.clone(),
						to.clone(),
						undeployed_land_block_id.clone(),
					));

					Ok(().into())
				},
			)
		}

		/// Unapprove external wallet to access raw land block.
		#[pallet::weight(T::WeightInfo::unapprove_undeployed_land_blocks())]
		pub fn unapprove_undeployed_land_blocks(
			origin: OriginFor<T>,
			undeployed_land_block_id: UndeployedLandBlockId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			UndeployedLandBlocks::<T>::try_mutate_exists(
				&undeployed_land_block_id,
				|undeployed_land_block| -> DispatchResultWithPostInfo {
					let mut undeployed_land_block_record = undeployed_land_block
						.as_mut()
						.ok_or(Error::<T>::UndeployedLandBlockNotFound)?;

					ensure!(
						undeployed_land_block_record.owner == who.clone(),
						Error::<T>::NoPermission
					);

					ensure!(
						undeployed_land_block_record.is_locked == false,
						Error::<T>::UndeployedLandBlockAlreadyFreezed
					);

					undeployed_land_block_record.approved = None;

					Self::deposit_event(Event::<T>::UndeployedLandBlockUnapproved(
						undeployed_land_block_id.clone(),
					));

					Ok(().into())
				},
			)
		}

		/// Dissolve estate to land units
		#[pallet::weight(T::WeightInfo::dissolve_estate())]
		pub fn dissolve_estate(origin: OriginFor<T>, estate_id: EstateId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				!T::AuctionHandler::check_item_in_auction(ItemId::Estate(estate_id)),
				Error::<T>::EstateAlreadyInAuction
			);
			let estate_info = Estates::<T>::get(estate_id).ok_or(Error::<T>::EstateDoesNotExist)?;

			EstateOwner::<T>::try_mutate_exists(&estate_id, |estate_owner| {
				//ensure there is record of the estate owner with estate id and account id

				let estate_owner_value = Self::get_estate_owner(&estate_id).ok_or(Error::<T>::NoPermission)?;
				ensure!(
					Self::check_if_land_or_estate_owner(&who, &estate_owner_value, true, estate_info.metaverse_id),
					Error::<T>::NoPermission
				);
				// Reset estate ownership
				match estate_owner_value {
					OwnerId::Token(t) => {
						let estate_class_id: ClassId =
							T::MetaverseInfoSource::get_metaverse_estate_class(estate_info.metaverse_id)?;
						T::NFTTokenizationSource::burn_nft(&who, &(estate_class_id, t));
						*estate_owner = None;
					}
					OwnerId::Account(ref a) => {
						*estate_owner = None;
					}
				}

				// Remove estate
				Estates::<T>::remove(&estate_id);

				// Update total estates
				let total_estates_count = Self::all_estates_count();
				let new_total_estates_count = total_estates_count
					.checked_sub(One::one())
					.ok_or("Overflow adding new count to total estates")?;
				AllEstatesCount::<T>::put(new_total_estates_count);

				Self::deposit_event(Event::<T>::EstateDestroyed(
					estate_id.clone(),
					estate_owner_value.clone(),
				));

				Ok(().into())
			})
		}

		/// Add more land units to existing estate
		#[pallet::weight(T::WeightInfo::add_land_unit_to_estate())]
		pub fn add_land_unit_to_estate(
			origin: OriginFor<T>,
			estate_id: EstateId,
			land_units: Vec<(i32, i32)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				!T::AuctionHandler::check_item_in_auction(ItemId::Estate(estate_id)),
				Error::<T>::EstateAlreadyInAuction
			);

			let estate_info: EstateInfo = Estates::<T>::get(estate_id).ok_or(Error::<T>::EstateDoesNotExist)?;

			// Check estate ownership
			let estate_owner_value = Self::get_estate_owner(&estate_id).ok_or(Error::<T>::NoPermission)?;
			ensure!(
				Self::check_if_land_or_estate_owner(&who, &estate_owner_value, true, estate_info.metaverse_id),
				Error::<T>::NoPermission
			);

			// Check land unit ownership
			for land_unit in land_units.clone() {
				ensure!(
					Self::check_if_land_or_estate_owner(
						&who,
						&Self::get_land_units(estate_info.metaverse_id, land_unit).unwrap(),
						false,
						estate_info.metaverse_id
					),
					Error::<T>::LandUnitDoesNotExist
				);
			}

			// Mutate estates
			Estates::<T>::try_mutate_exists(&estate_id, |maybe_estate_info| {
				// Append new coordinates to estate
				let mut mut_estate_info = maybe_estate_info.as_mut().ok_or(Error::<T>::EstateDoesNotExist)?;
				mut_estate_info.land_units.append(&mut land_units.clone());

				Self::deposit_event(Event::<T>::LandUnitAdded(
					estate_id.clone(),
					estate_owner_value.clone(),
					land_units.clone(),
				));

				Ok(().into())
			})
		}

		/// Remove land units from existing estate
		#[pallet::weight(T::WeightInfo::remove_land_unit_from_estate())]
		pub fn remove_land_unit_from_estate(
			origin: OriginFor<T>,
			estate_id: EstateId,
			land_units: Vec<(i32, i32)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				!T::AuctionHandler::check_item_in_auction(ItemId::Estate(estate_id)),
				Error::<T>::EstateAlreadyInAuction
			);

			let estate_info: EstateInfo = Estates::<T>::get(estate_id).ok_or(Error::<T>::EstateDoesNotExist)?;

			// Check estate ownership
			let estate_owner_value = Self::get_estate_owner(&estate_id).ok_or(Error::<T>::NoPermission)?;
			ensure!(
				Self::check_if_land_or_estate_owner(&who, &estate_owner_value, true, estate_info.metaverse_id),
				Error::<T>::NoPermission
			);

			// Mutate estates
			Estates::<T>::try_mutate_exists(&estate_id, |maybe_estate_info| {
				let mut mut_estate_info = maybe_estate_info.as_mut().ok_or(Error::<T>::EstateDoesNotExist)?;

				// Mutate land unit ownership
				for land_unit in land_units.clone() {
					// Remove coordinates from estate
					let index = mut_estate_info.land_units.iter().position(|x| *x == land_unit).unwrap();
					mut_estate_info.land_units.remove(index);
				}

				Self::deposit_event(Event::<T>::LandUnitsRemoved(
					estate_id.clone(),
					estate_owner_value.clone(),
					land_units.clone(),
				));

				Ok(().into())
			})
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Internal getter for new estate ID
	fn get_new_estate_id() -> Result<EstateId, DispatchError> {
		let estate_id = NextEstateId::<T>::try_mutate(|id| -> Result<EstateId, DispatchError> {
			let current_id = *id;
			*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableEstateId)?;
			Ok(current_id)
		})?;
		Ok(estate_id)
	}

	/// Internal minting of land unit
	fn mint_land_unit(
		metaverse_id: MetaverseId,
		beneficiary: T::AccountId,
		coordinate: (i32, i32),
		land_unit_status: LandUnitStatus<T::AccountId>,
	) -> Result<OwnerId<T::AccountId, TokenId>, DispatchError> {
		let mut owner = OwnerId::Account(beneficiary.clone());

		match land_unit_status {
			LandUnitStatus::Existing(a) => {
				ensure!(
					LandUnits::<T>::contains_key(metaverse_id, coordinate),
					Error::<T>::LandUnitIsNotAvailable
				);

				let existing_owner_value = Self::get_land_units(metaverse_id, coordinate);
				match existing_owner_value {
					Some(owner_value) => match owner_value {
						OwnerId::Token(t) => owner = owner_value,
						OwnerId::Account(a) => {
							let token_properties = Self::get_land_token_properties(metaverse_id, coordinate);
							let class_id = T::MetaverseInfoSource::get_metaverse_land_class(metaverse_id)?;
							let asset_id = T::NFTTokenizationSource::mint_token(
								&beneficiary,
								class_id,
								token_properties.0,
								token_properties.1,
							)?;
							owner = OwnerId::Token(asset_id);
						}
					},
					None => {
						let token_properties = Self::get_land_token_properties(metaverse_id, coordinate);
						let class_id = T::MetaverseInfoSource::get_metaverse_land_class(metaverse_id)?;
						let asset_id = T::NFTTokenizationSource::mint_token(
							&beneficiary,
							class_id,
							token_properties.0,
							token_properties.1,
						)?;
						owner = OwnerId::Token(asset_id);
					}
				}
			}
			LandUnitStatus::NonExisting => {
				ensure!(
					!LandUnits::<T>::contains_key(metaverse_id, coordinate),
					Error::<T>::LandUnitIsNotAvailable
				);

				let token_properties = Self::get_land_token_properties(metaverse_id, coordinate);
				let class_id = T::MetaverseInfoSource::get_metaverse_land_class(metaverse_id)?;
				let asset_id = T::NFTTokenizationSource::mint_token(
					&beneficiary,
					class_id,
					token_properties.0,
					token_properties.1,
				)?;
				owner = OwnerId::Token(asset_id);
			}
		}
		LandUnits::<T>::insert(metaverse_id, coordinate, owner.clone());
		Ok(owner)
	}

	/// Internal updating information about an estate
	fn update_estate_information(
		new_estate_id: EstateId,
		metaverse_id: MetaverseId,
		beneficiary: &T::AccountId,
		coordinates: Vec<(i32, i32)>,
	) -> DispatchResult {
		// Update total estates
		let total_estates_count = Self::all_estates_count();
		let new_total_estates_count = total_estates_count
			.checked_add(One::one())
			.ok_or("Overflow adding new count to total estates")?;
		AllEstatesCount::<T>::put(new_total_estates_count);

		// Update estates
		let estate_info = EstateInfo {
			metaverse_id,
			land_units: coordinates.clone(),
		};

		let token_properties = Self::get_estate_token_properties(metaverse_id, new_estate_id);
		let class_id = T::MetaverseInfoSource::get_metaverse_estate_class(metaverse_id)?;
		let asset_id: TokenId =
			T::NFTTokenizationSource::mint_token(beneficiary, class_id, token_properties.0, token_properties.1)?;
		let owner = OwnerId::Token(asset_id);

		Estates::<T>::insert(new_estate_id, estate_info);

		EstateOwner::<T>::insert(new_estate_id, owner.clone());

		Self::deposit_event(Event::<T>::NewEstateMinted(
			new_estate_id.clone(),
			owner.clone(),
			metaverse_id,
			coordinates.clone(),
		));

		Ok(())
	}

	/// Internal getter of new undeployed land block ID
	fn get_new_undeployed_land_block_id() -> Result<UndeployedLandBlockId, DispatchError> {
		let undeployed_land_block_id =
			NextUndeployedLandBlockId::<T>::try_mutate(|id| -> Result<UndeployedLandBlockId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(One::one()).ok_or(Error::<T>::NoAvailableEstateId)?;
				Ok(current_id)
			})?;
		Ok(undeployed_land_block_id)
	}

	/// Internal transfer of undeployed land block
	fn do_transfer_undeployed_land_block(
		who: &T::AccountId,
		to: &T::AccountId,
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError> {
		UndeployedLandBlocks::<T>::try_mutate_exists(
			&undeployed_land_block_id,
			|undeployed_land_block| -> Result<UndeployedLandBlockId, DispatchError> {
				let mut undeployed_land_block_record = undeployed_land_block
					.as_mut()
					.ok_or(Error::<T>::UndeployedLandBlockNotFound)?;

				ensure!(
					undeployed_land_block_record.owner == who.clone(),
					Error::<T>::NoPermission
				);

				ensure!(
					undeployed_land_block_record.is_locked == false,
					Error::<T>::UndeployedLandBlockAlreadyFreezed
				);

				ensure!(
					undeployed_land_block_record.undeployed_land_block_type == UndeployedLandBlockType::Transferable,
					Error::<T>::UndeployedLandBlockIsNotTransferable
				);

				undeployed_land_block_record.owner = to.clone();

				UndeployedLandBlocksOwner::<T>::remove(who.clone(), &undeployed_land_block_id);
				UndeployedLandBlocksOwner::<T>::insert(to.clone(), &undeployed_land_block_id, ());

				Self::deposit_event(Event::<T>::UndeployedLandBlockTransferred(
					who.clone(),
					to.clone(),
					undeployed_land_block_id.clone(),
				));

				Ok(undeployed_land_block_id)
			},
		)
	}

	/// Internal burn of undeployed land block
	fn do_burn_undeployed_land_block(
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError> {
		let undeployed_land_block_info =
			UndeployedLandBlocks::<T>::get(undeployed_land_block_id).ok_or(Error::<T>::UndeployedLandBlockNotFound)?;

		ensure!(
			!undeployed_land_block_info.is_locked,
			Error::<T>::OnlyFrozenUndeployedLandBlockCanBeDestroyed
		);
		Self::set_total_undeployed_land_unit(undeployed_land_block_info.number_land_units as u64, true)?;
		UndeployedLandBlocksOwner::<T>::remove(undeployed_land_block_info.owner, &undeployed_land_block_id);
		UndeployedLandBlocks::<T>::remove(&undeployed_land_block_id);

		Self::deposit_event(Event::<T>::UndeployedLandBlockBurnt(undeployed_land_block_id.clone()));

		Ok(undeployed_land_block_id)
	}

	/// Internal freeze of undeployed land block
	fn do_freeze_undeployed_land_block(
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError> {
		UndeployedLandBlocks::<T>::try_mutate_exists(
			&undeployed_land_block_id,
			|undeployed_land_block| -> Result<UndeployedLandBlockId, DispatchError> {
				let mut undeployed_land_block_record = undeployed_land_block
					.as_mut()
					.ok_or(Error::<T>::UndeployedLandBlockNotFound)?;

				ensure!(
					undeployed_land_block_record.is_locked == false,
					Error::<T>::UndeployedLandBlockAlreadyFreezed
				);

				undeployed_land_block_record.is_locked = true;

				Self::deposit_event(Event::<T>::UndeployedLandBlockFreezed(undeployed_land_block_id));

				Ok(undeployed_land_block_id)
			},
		)
	}

	/// Internal issue of undeployed land block
	fn do_issue_undeployed_land_blocks(
		beneficiary: &T::AccountId,
		number_of_land_block: u32,
		number_land_units_per_land_block: u32,
		undeployed_land_block_type: UndeployedLandBlockType,
	) -> Result<Vec<UndeployedLandBlockId>, DispatchError> {
		let mut undeployed_land_block_ids: Vec<UndeployedLandBlockId> = Vec::new();

		for _ in 0..number_of_land_block {
			let new_undeployed_land_block_id = Self::get_new_undeployed_land_block_id()?;

			let undeployed_land_block = UndeployedLandBlock {
				id: new_undeployed_land_block_id,
				number_land_units: number_land_units_per_land_block,
				undeployed_land_block_type,
				approved: None,
				is_locked: false,
				owner: beneficiary.clone(),
			};

			UndeployedLandBlocks::<T>::insert(new_undeployed_land_block_id, undeployed_land_block);
			UndeployedLandBlocksOwner::<T>::insert(beneficiary.clone(), new_undeployed_land_block_id, ());

			// Update total undeployed land  count
			Self::set_total_undeployed_land_unit(number_land_units_per_land_block as u64, false)?;

			Self::deposit_event(Event::<T>::UndeployedLandBlockIssued(
				beneficiary.clone(),
				new_undeployed_land_block_id.clone(),
			));

			undeployed_land_block_ids.push(new_undeployed_land_block_id);
		}

		Ok(undeployed_land_block_ids)
	}

	/// Internal transfer of estate
	fn do_transfer_estate(
		estate_id: EstateId,
		from: &T::AccountId,
		to: &T::AccountId,
	) -> Result<EstateId, DispatchError> {
		EstateOwner::<T>::try_mutate_exists(&estate_id, |estate_owner| -> Result<EstateId, DispatchError> {
			//ensure there is record of the estate owner with estate id and account id
			ensure!(from != to, Error::<T>::AlreadyOwnTheEstate);
			let estate_owner_value = Self::get_estate_owner(&estate_id).ok_or(Error::<T>::NoPermission)?;
			let estate_info = Estates::<T>::get(estate_id).ok_or(Error::<T>::EstateDoesNotExist)?;
			ensure!(
				Self::check_if_land_or_estate_owner(from, &estate_owner_value, true, estate_info.metaverse_id),
				Error::<T>::NoPermission
			);

			match estate_owner_value {
				OwnerId::Token(t) => {
					let estate_info = Estates::<T>::get(estate_id).ok_or(Error::<T>::EstateDoesNotExist)?;
					let estate_class_id: ClassId =
						T::MetaverseInfoSource::get_metaverse_estate_class(estate_info.metaverse_id)?;
					T::NFTTokenizationSource::transfer_nft(from, to, &(estate_class_id, t));

					Self::deposit_event(Event::<T>::TransferredEstate(
						estate_id.clone(),
						from.clone(),
						to.clone(),
					));

					Ok(estate_id)
				}
				_ => Err(Error::<T>::InvalidOwnerValue.into()),
			}
		})
	}

	/// Internal transfer of land unit
	fn do_transfer_landunit(
		coordinate: (i32, i32),
		from: &T::AccountId,
		to: &T::AccountId,
		metaverse_id: MetaverseId,
	) -> Result<(i32, i32), DispatchError> {
		LandUnits::<T>::try_mutate_exists(
			&metaverse_id,
			&coordinate,
			|land_unit_owner| -> Result<(i32, i32), DispatchError> {
				// ensure there is record of the land unit with bit country id and coordinate
				ensure!(land_unit_owner.is_some(), Error::<T>::NoPermission);
				ensure!(from != to, Error::<T>::AlreadyOwnTheLandUnit);
				match land_unit_owner {
					Some(owner) => {
						ensure!(
							Self::check_if_land_or_estate_owner(from, owner, false, metaverse_id),
							Error::<T>::NoPermission
						);
						match owner {
							OwnerId::Token(t) => {
								let land_class_id: ClassId =
									T::MetaverseInfoSource::get_metaverse_land_class(metaverse_id)?;
								T::NFTTokenizationSource::transfer_nft(from, to, &(land_class_id, *t));
								// Update
								Self::deposit_event(Event::<T>::TransferredLandUnit(
									metaverse_id.clone(),
									coordinate.clone(),
									from.clone(),
									to.clone(),
								));

								Ok(coordinate)
							}
							_ => Err(Error::<T>::InvalidOwnerValue.into()),
						}
					}
					None => Err(DispatchError::Other("No Permissson")),
				}
			},
		)
	}

	/// Internal setting of total undeployed land units
	fn set_total_undeployed_land_unit(total: u64, deduct: bool) -> Result<(), DispatchError> {
		let total_undeployed_land_units = Self::all_undeployed_land_unit();

		if deduct {
			let new_total_undeployed_land_unit_count = total_undeployed_land_units
				.checked_sub(total)
				.ok_or("Overflow deducting new count to total undeployed lands")?;
			TotalUndeployedLandUnit::<T>::put(new_total_undeployed_land_unit_count);
		} else {
			let new_total_undeployed_land_unit_count = total_undeployed_land_units
				.checked_add(total)
				.ok_or("Overflow adding new count to total undeployed lands")?;
			TotalUndeployedLandUnit::<T>::put(new_total_undeployed_land_unit_count);
		}

		Ok(())
	}

	/// Internal setting of total land units
	fn set_total_land_unit(total: u64, deduct: bool) -> Result<(), DispatchError> {
		let total_land_units_count = Self::all_land_units_count();

		if deduct {
			let new_total_land_units_count = total_land_units_count
				.checked_sub(total)
				.ok_or("Overflow deducting new count to total lands")?;
			AllLandUnitsCount::<T>::put(new_total_land_units_count);
		} else {
			let new_total_land_units_count = total_land_units_count
				.checked_add(total)
				.ok_or("Overflow adding new count to total lands")?;
			AllLandUnitsCount::<T>::put(new_total_land_units_count);
		}
		Ok(())
	}

	/// Internal getter of land token properties
	fn get_land_token_properties(metaverse_id: MetaverseId, coordinate: (i32, i32)) -> (NftMetadata, Attributes) {
		let mut land_coordinate_attribute = Vec::<u8>::new();
		land_coordinate_attribute.append(&mut coordinate.0.to_be_bytes().to_vec());
		land_coordinate_attribute.append(&mut coordinate.1.to_be_bytes().to_vec());

		let mut nft_metadata: NftMetadata = NftMetadata::new();
		nft_metadata.append(&mut land_coordinate_attribute.clone());

		let mut nft_attributes: Attributes = Attributes::new();
		nft_attributes.insert("MetaverseId:".as_bytes().to_vec(), metaverse_id.to_be_bytes().to_vec());
		nft_attributes.insert("Coordinate:".as_bytes().to_vec(), land_coordinate_attribute);

		return (nft_metadata, nft_attributes);
	}

	/// Internal getter of estate token properties
	fn get_estate_token_properties(metaverse_id: MetaverseId, estate_id: EstateId) -> (NftMetadata, Attributes) {
		let mut nft_metadata: NftMetadata = NftMetadata::new();
		nft_metadata.append(&mut metaverse_id.to_be_bytes().to_vec());
		nft_metadata.append(&mut estate_id.to_be_bytes().to_vec());

		let mut nft_attributes: Attributes = Attributes::new();
		nft_attributes.insert("MetaverseId:".as_bytes().to_vec(), metaverse_id.to_be_bytes().to_vec());
		nft_attributes.insert("Estate Id:".as_bytes().to_vec(), estate_id.to_be_bytes().to_vec());

		return (nft_metadata, nft_attributes);
	}

	fn check_if_land_or_estate_owner(
		who: &T::AccountId,
		owner_id: &OwnerId<T::AccountId, TokenId>,
		is_estate_owner: bool,
		metaverse_id: MetaverseId,
	) -> bool {
		match owner_id {
			OwnerId::Token(asset_id) => {
				if is_estate_owner {
					let estate_class_id: ClassId =
						T::MetaverseInfoSource::get_metaverse_estate_class(metaverse_id).unwrap_or(0);
					return T::NFTTokenizationSource::check_ownership(who, &(estate_class_id, *asset_id))
						.unwrap_or(false);
				} else {
					let land_class_id: ClassId =
						T::MetaverseInfoSource::get_metaverse_land_class(metaverse_id).unwrap_or(0);
					return T::NFTTokenizationSource::check_ownership(who, &(land_class_id, *asset_id))
						.unwrap_or(false);
				}
			}
			_ => return false,
		}
	}

	fn verify_land_unit_for_estate(land_units: Vec<(i32, i32)>) -> bool {
		let mut vec_axis = land_units.iter().map(|lu| lu.0).collect::<Vec<_>>();
		let mut vec_yaxis = land_units.iter().map(|lu| lu.1).collect::<Vec<_>>();

		// Sort by ascending and dedup
		vec_axis.sort();
		vec_axis.dedup();
		vec_yaxis.sort();
		vec_yaxis.dedup();

		let mut is_axis_valid = true;
		let mut is_yaxis_valid = true;

		// Ensure axis is next to each other
		for (i, axis) in vec_axis.iter().enumerate() {
			if axis != &vec_axis[i] {
				let valid = axis.saturating_sub(vec_axis[i + 1]);
				if valid != 1 {
					is_axis_valid = false;
					break;
				}
			}
		}

		// Ensure yaxis is next to each other
		for (i, yaxis) in vec_yaxis.iter().enumerate() {
			if yaxis != &vec_yaxis[i] {
				let valid = yaxis.saturating_sub(vec_yaxis[i + 1]);
				if valid != 1 {
					is_yaxis_valid = false;
					break;
				}
			}
		}

		is_axis_valid && is_yaxis_valid
	}

	fn verify_land_unit_in_bound(block_coordinate: &(i32, i32), land_unit_coordinates: &Vec<(i32, i32)>) -> bool {
		let mut vec_axis = land_unit_coordinates.iter().map(|lu| lu.0).collect::<Vec<_>>();
		let mut vec_yaxis = land_unit_coordinates.iter().map(|lu| lu.1).collect::<Vec<_>>();

		let max_axis = *vec_axis.iter().max().unwrap();
		let max_yaxis = *vec_yaxis.iter().max().unwrap();

		block_coordinate.0.saturating_sub(49i32) <= *vec_axis.iter().min().unwrap()
			&& block_coordinate.0 <= max_axis.saturating_add(50i32)
			&& block_coordinate.1.saturating_sub(49i32) <= *vec_yaxis.iter().min().unwrap()
			&& block_coordinate.1 <= max_yaxis.saturating_add(50i32)
	}
}

impl<T: Config> MetaverseLandTrait<T::AccountId> for Pallet<T> {
	fn get_user_land_units(who: &T::AccountId, metaverse_id: &MetaverseId) -> Vec<(i32, i32)> {
		// Check land units owner.
		let mut total_land_units: Vec<(i32, i32)> = Vec::default();

		let land_in_metaverse = LandUnits::<T>::iter_prefix(metaverse_id)
			.filter(|(_, owner)| Self::check_if_land_or_estate_owner(who, owner, false, *metaverse_id))
			.collect::<Vec<_>>();

		for land_unit in land_in_metaverse {
			let land = land_unit.0;
			total_land_units.push(land);
		}

		total_land_units
	}

	fn is_user_own_metaverse_land(who: &T::AccountId, metaverse_id: &MetaverseId) -> bool {
		Self::get_user_land_units(&who, metaverse_id).len() > 0
	}
}

impl<T: Config> UndeployedLandBlocksTrait<T::AccountId> for Pallet<T> {
	fn issue_undeployed_land_blocks(
		beneficiary: &T::AccountId,
		number_of_land_block: u32,
		number_land_units_per_land_block: u32,
		undeployed_land_block_type: UndeployedLandBlockType,
	) -> Result<Vec<UndeployedLandBlockId>, DispatchError> {
		let new_undeployed_land_block_id = Self::do_issue_undeployed_land_blocks(
			&beneficiary,
			number_of_land_block,
			number_land_units_per_land_block,
			undeployed_land_block_type,
		)?;

		Ok(new_undeployed_land_block_id)
	}

	fn transfer_undeployed_land_block(
		who: &T::AccountId,
		to: &T::AccountId,
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError> {
		let undeployed_land_block_id = Self::do_transfer_undeployed_land_block(who, to, undeployed_land_block_id)?;

		Ok(undeployed_land_block_id)
	}

	fn burn_undeployed_land_block(
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError> {
		let undeployed_land_block_id = Self::do_burn_undeployed_land_block(undeployed_land_block_id)?;

		Ok(undeployed_land_block_id)
	}

	fn freeze_undeployed_land_block(
		undeployed_land_block_id: UndeployedLandBlockId,
	) -> Result<UndeployedLandBlockId, DispatchError> {
		let undeployed_land_block_id = Self::do_freeze_undeployed_land_block(undeployed_land_block_id)?;

		Ok(undeployed_land_block_id)
	}
}

impl<T: Config> Estate<T::AccountId> for Pallet<T> {
	fn transfer_estate(estate_id: EstateId, from: &T::AccountId, to: &T::AccountId) -> Result<EstateId, DispatchError> {
		ensure!(
			T::AuctionHandler::check_item_in_auction(ItemId::Estate(estate_id)),
			Error::<T>::EstateNotInAuction
		);

		let estate_id = Self::do_transfer_estate(estate_id, from, to)?;
		Ok(estate_id)
	}

	fn transfer_landunit(
		coordinate: (i32, i32),
		from: &T::AccountId,
		to: &(T::AccountId, MetaverseId),
	) -> Result<(i32, i32), DispatchError> {
		ensure!(
			T::AuctionHandler::check_item_in_auction(ItemId::LandUnit(coordinate, to.1)),
			Error::<T>::LandUnitNotInAuction
		);

		let coordinate = Self::do_transfer_landunit(coordinate, from, &(to).0, to.1)?;
		Ok(coordinate)
	}

	fn check_estate(estate_id: EstateId) -> Result<bool, DispatchError> {
		Ok(Estates::<T>::contains_key(estate_id))
	}

	fn check_landunit(metaverse_id: MetaverseId, coordinate: (i32, i32)) -> Result<bool, DispatchError> {
		Ok(LandUnits::<T>::contains_key(metaverse_id, coordinate))
	}

	fn get_total_land_units() -> u64 {
		AllLandUnitsCount::<T>::get()
	}

	fn get_total_undeploy_land_units() -> u64 {
		TotalUndeployedLandUnit::<T>::get()
	}
}
