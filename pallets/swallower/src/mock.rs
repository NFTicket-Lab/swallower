use super::*;
use crate as pallet_swallower;
use frame_support::{parameter_types, traits::GenesisBuild, construct_runtime};
use frame_system as system;
use pallet_assets::FrozenBalance;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Swallower: pallet_swallower::{Pallet, Call, Storage, Config<T>, Event<T>},
		CollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl system::Config for TestRuntime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for TestRuntime {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const AssetDeposit: u64 = 1;
	pub const ApprovalDeposit: u64 = 1;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 1;
	pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_assets::Config for TestRuntime {
	type Event = Event;
	type Balance = u64;
	type AssetId = u32;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = TestFreezer;
	type WeightInfo = ();
	type Extra = ();
}

impl pallet_randomness_collective_flip::Config for TestRuntime {}

parameter_types! {
	pub const InitGeneLimit: u32 = 16;
	pub const MaxSwallowerOwen: u32 = 128;
}
impl Config for TestRuntime {
	type Event = Event;
	type InitGeneLimit = InitGeneLimit;

	type AssetsTransfer= Assets;
	type GeneRandomness = CollectiveFlip;
	type MaxSwallowerOwen = MaxSwallowerOwen;
	// type MyAssetId = u32;
}

use std::{cell::RefCell, collections::HashMap, vec};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Hook {
	Died(u32, u64),
}
thread_local! {
	static FROZEN: RefCell<HashMap<(u32, u64), u64>> = RefCell::new(Default::default());
	static HOOKS: RefCell<Vec<Hook>> = RefCell::new(Default::default());
}

pub struct TestFreezer;
impl FrozenBalance<u32, u64, u64> for TestFreezer {
	fn frozen_balance(asset: u32, who: &u64) -> Option<u64> {
		FROZEN.with(|f| f.borrow().get(&(asset, who.clone())).cloned())
	}

	fn died(asset: u32, who: &u64) {
		HOOKS.with(|h| h.borrow_mut().push(Hook::Died(asset, who.clone())));
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage =
		frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();
	let asset_config: pallet_assets::GenesisConfig<TestRuntime> = pallet_assets::GenesisConfig {
		assets: vec![
			// id, owner, is_sufficient, min_balance
			(1, 0, true, 1),
		],
		metadata: vec![
			// id, name, symbol, decimals
			(1, "Token Name".into(), "TOKEN".into(), 10),
		],
		accounts: vec![
			// id, account_id, balance
			(1, 1, 10000000000000),
		],
	};
	asset_config.assimilate_storage(&mut storage).unwrap();
	let balance_config = pallet_balances::GenesisConfig::<TestRuntime>{
		balances: vec![(1,100000000000000000)],
	};
	balance_config.assimilate_storage(&mut storage).unwrap();
	//默认添加一个管理员。
	let swallower_config = pallet_swallower::GenesisConfig::<TestRuntime>{
		// manager: Some(1u64),
		admin:None,
		asset_id:None,
		// asset_id: None,
	};
	swallower_config.assimilate_storage(&mut storage).unwrap();
	let mut ext:sp_io::TestExternalities =storage.into();
	ext.execute_with(||System::set_block_number(1));
	ext
}