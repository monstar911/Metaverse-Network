#![cfg(test)]

use super::*;
use crate as estate;
// use crate::{Config, Module};
use bc_primitives::*;
// // use sp_std::vec::Vec;
use frame_support::pallet_prelude::{GenesisBuild, Hooks, MaybeSerializeDeserialize};
use frame_support::sp_runtime::traits::AtLeast32Bit;
use frame_support::{
    construct_runtime, impl_outer_dispatch, impl_outer_event, impl_outer_origin,
    ord_parameter_types, parameter_types, traits::EnsureOrigin, weights::Weight,
};
use frame_support::ensure;
use frame_system::{EnsureRoot, EnsureSignedBy};
use frame_system::{ensure_root, ensure_signed};
use primitives::{Amount, CurrencyId, FungibleTokenId};
use sp_core::{
    H256,
    u32_trait::{_1, _2, _3, _4, _5}
};
use sp_runtime::{testing::Header, traits::IdentityLookup, ModuleId, DispatchError, Perbill};

pub type AccountId = u128;
pub type AuctionId = u64;
pub type Balance = u128;
pub type BitCountryId = u64;
pub type BlockNumber = u64;
pub type LandId = u64;
pub type EstateId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 5;
pub const BITCOUNTRY_ID: BitCountryId = 1;
pub const COUNTRY_ID_NOT_EXIST: BitCountryId = 1;
pub const NUUM: CurrencyId = 0;
pub const DOLLARS: Balance = 1_000_000_000_000_000_000;
pub const ALICE_COUNTRY_ID: BitCountryId = 1;
pub const BOB_COUNTRY_ID: BitCountryId = 2;

ord_parameter_types! {
    pub const One: AccountId = ALICE;
}

// Configure a mock runtime to test the pallet.

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: u32 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type MaxLocks = ();
    type WeightInfo = ();
}

// pub type AdaptedBasicCurrency =
// currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

parameter_types! {
    pub const GetNativeCurrencyId: FungibleTokenId = FungibleTokenId::NativeToken(0);
    pub const MiningCurrencyId: FungibleTokenId = FungibleTokenId::MiningResource(0);
    pub const LandTreasuryModuleId: ModuleId = ModuleId(*b"bit/land");
    pub const MinimumLandPrice: Balance = 10 * DOLLARS;
    pub const CountryFundModuleId: ModuleId = ModuleId(*b"bit/fund");
}

pub struct BitCountryInfoSource {}

impl BitCountryTrait<AccountId> for BitCountryInfoSource {
    fn check_ownership(who: &AccountId, country_id: &BitCountryId) -> bool {
        match *who {
            ALICE => *country_id == ALICE_COUNTRY_ID,
            BOB => *country_id == BOB_COUNTRY_ID,
            _ => false,
        }
    }

    fn get_bitcountry(bitcountry_id: u64) -> Option<BitCountryStruct<u128>> {
        None
    }

    fn get_bitcountry_token(bitcountry_id: u64) -> Option<FungibleTokenId> {
        None
    }

    fn update_bitcountry_token(
        bitcountry_id: u64,
        currency_id: FungibleTokenId,
    ) -> Result<(), DispatchError> {
        Ok(())
    }
}

type CouncilCollective = pallet_collective::Instance1;

impl Config for Runtime {
    type Event = Event;
    type LandTreasury = LandTreasuryModuleId;
    type BitCountryInfoSource = BitCountryInfoSource;
    type Currency = Balances;
    type MinimumLandPrice = MinimumLandPrice;
    type CouncilOrigin = EnsureSignedBy<One, AccountId>;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        // Currencies: currencies::{ Module, Storage, Call, Event<T>},
        // BitCountryModule: bitcountry::{Module, Call, Storage, Event<T>},
        Estate: estate:: {Module, Call, Storage, Event<T>}
    }
);

pub type EstateModule = Module<Runtime>;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
        ExtBuilder
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: vec![(ALICE, 100000)],
        }
            .assimilate_storage(&mut t)
            .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

pub fn last_event() -> Event {
    frame_system::Module::<Runtime>::events()
        .pop()
        .expect("Event expected")
        .event
}