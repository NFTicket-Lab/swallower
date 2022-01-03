//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Swallower;
use codec::{FullCodec};
// use crate::mock::{TestRuntime, Origin};
use frame_benchmarking::{benchmarks,benchmarks_instance_pallet, whitelisted_caller, account};
use frame_system::RawOrigin as SystemOrigin;
use sp_runtime::traits::StaticLookup;
use frame_support::{traits::tokens::AssetId};
use sp_std::prelude::*;


const SEED: u32 = 0;
const ASSET_ID:u32 = 2;
const ACCOUNT_ID_1:u64 = 3;
const ACCOUNT_ID_2:u64 = 4;
const ADMIN_ID:u32 = 2;
const NAME:&[u8;4] = b"hole";
const NAME1:&[u8;10] = b"dragon_two";
const NAME2:&[u8;12] = b"dragon_three";
const ACCOUNT_ASSET_OWNER_ID:u64 = 1;
const MANAGER_ID:u64 = 0;



fn convert_asset_id<T:FullCodec,F:AssetId>(id:T)->F{
	<F>::decode(&mut (AsRef::<[u8]>::as_ref(&id.encode()))).unwrap()
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn assert_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn pre_set_admin<T:Config>()->T::AccountId{
	let admin: T::AccountId = account("admin", ADMIN_ID, SEED);
	let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(admin.clone());
	Swallower::<T>::set_admin(SystemOrigin::Root.into(),user_lookup).unwrap();
	admin
}

fn pre_set_asset_id<T:Config>(){
	let admin = pre_set_admin::<T>();
	create_default_asset::<T>(true);
	Swallower::<T>::set_asset_id(SystemOrigin::Signed(admin).into(), convert_asset_id(ASSET_ID)).unwrap();
}

fn transfer_to_sender<T:Config>(){
	let account_asset: T::AccountId = account("asset", ACCOUNT_ASSET_OWNER_ID as u32, SEED);
	let challenger: T::AccountId = account("challenger", ACCOUNT_ID_1 as u32, SEED);
	// let Bob = account("Bob", ACCOUNT_ID_1 as u32, SEED);
	let challenger_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(challenger.clone());
	let transfer_amount = 170000000000u64.try_into().unwrap_or(Default::default());
	pallet_assets::Pallet::<T>::mint(SystemOrigin::Signed(account_asset).into(),convert_asset_id(ASSET_ID),challenger_lookup,transfer_amount).unwrap();
	// pallet_assets::Pallet::<T>::transfer(SystemOrigin::Signed(account_asset).into(),convert_asset_id(ASSET_ID),challenger_lookup,transfer_amount).unwrap();
}

fn create_default_asset<T: Config>(
	is_sufficient: bool,
) -> (T::AccountId, <T::Lookup as StaticLookup>::Source) {
	let account_asset: T::AccountId = account("asset", ACCOUNT_ASSET_OWNER_ID as u32, SEED);
	let account_asset_lookup = T::Lookup::unlookup(account_asset.clone());
	let root = SystemOrigin::Root.into();
	assert!(pallet_assets::Pallet::<T>::force_create(
		root,
		convert_asset_id(ASSET_ID),
		account_asset_lookup.clone(),
		is_sufficient,
		170000000000u64.try_into().unwrap_or(Default::default()),
	)
	.is_ok());
	(account_asset, account_asset_lookup)
}


benchmarks!{
	set_admin {
		let caller: T::AccountId = whitelisted_caller();
		let admin: T::AccountId = account("admin", ADMIN_ID, SEED);
		let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(admin.clone());
	}: _(SystemOrigin::Root, user_lookup)
	verify {
		let admin_id = Swallower::<T>::admin().unwrap();
		assert_eq!(admin_id, admin);
		// assert_eq!(admin_id, caller);
	}

	set_asset_id {
		// let s in 0 .. 100;
		// let s = 1;
		let caller: T::AccountId = whitelisted_caller();
		let admin = pre_set_admin::<T>();
		let asset_id:AssetIdOf<T> = convert_asset_id(ASSET_ID);
	}: _(SystemOrigin::Signed(admin), asset_id)
	verify {
		let stor_asset_id = Swallower::<T>::asset_id().unwrap();
		// let s = convert_asset_id(s);
		assert_eq!(stor_asset_id, asset_id);
	}


	mint_swallower {
		let i in 0 .. 100;
		pre_set_asset_id::<T>();
		let name = vec![0u8;i as usize];
		transfer_to_sender::<T>();
		let challenger: T::AccountId = account("BOB", ACCOUNT_ID_1 as u32, SEED);
	}: _(SystemOrigin::Signed(challenger), name)
	verify {
		let hash = Default::default();
		assert_last_event::<T>(Event::EntreSafeZone (hash,1u32.into(),1601u32.into()).into());
	}


	impl_benchmark_test_suite!(Swallower, crate::mock::new_test_ext(), crate::mock::TestRuntime);
}
