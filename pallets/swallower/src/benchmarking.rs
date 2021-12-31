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

const SEED: u32 = 0;
const ASSET_ID:u32 = 1;
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

fn pre_set_admin<T:Config>()->T::AccountId{
	let admin: T::AccountId = account("admin", ADMIN_ID, SEED);
	let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(admin.clone());
	Swallower::<T>::set_admin(SystemOrigin::Root.into(),user_lookup).unwrap();
	admin
}

fn pre_set_asset_id<T:Config>(){
	let admin: T::AccountId = account("admin", ADMIN_ID, SEED);
	Swallower::<T>::set_asset_id(SystemOrigin::Signed(admin).into(), convert_asset_id(ASSET_ID)).unwrap();
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
		let s = 1;
		let caller: T::AccountId = whitelisted_caller();
		let admin = pre_set_admin::<T>();
	}: set_asset_id(SystemOrigin::Signed(admin), convert_asset_id(s))
	verify {
		let asset_id = Swallower::<T>::asset_id().unwrap();
		let s = convert_asset_id(s);
		assert_eq!(asset_id, s);
	}


	// mint_swallower {
	// 	let swallower_name = b"my_swallower";
	// 	let caller: T::AccountId = whitelisted_caller();
	// }: _(SystemOrigin::Signed(caller), swallower_name.to_vec())
	// verify {
	// 	// assert_last_event::<T>(Event::EntreSafeZone (_,2,3).into());
	// }


	impl_benchmark_test_suite!(Swallower, crate::mock::new_test_ext(), crate::mock::TestRuntime);
}
