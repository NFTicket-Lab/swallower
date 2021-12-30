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

fn get_asset_id<T:FullCodec,F:AssetId>(id:T)->F{
	<F>::decode(&mut (AsRef::<[u8]>::as_ref(&id.encode()))).unwrap()
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks!{
	set_asset_id {
		// let s in 0 .. 100;
		let s = 1;
		let caller: T::AccountId = whitelisted_caller();
		let user: T::AccountId = account("user", 2, SEED);
		let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user.clone());
	}: set_asset_id(SystemOrigin::Signed(user), get_asset_id(s))
	verify {
		let asset_id = Swallower::<T>::asset_id().unwrap();
		let s = get_asset_id(s);
		assert_eq!(asset_id, s);
	}

	set_admin {
		let caller: T::AccountId = whitelisted_caller();
		let admin: T::AccountId = account("admin", 2, SEED);
		let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(admin.clone());
	}: _(SystemOrigin::Root, user_lookup)
	verify {
		let admin_id = Swallower::<T>::admin().unwrap();
		assert_eq!(admin_id, admin);
		// assert_eq!(admin_id, caller);
	}

	mint_swallower {
		let swallower_name = b"my_swallower";
		let caller: T::AccountId = whitelisted_caller();
	}: _(SystemOrigin::Signed(caller), swallower_name.to_vec())
	verify {
		// assert_last_event::<T>(Event::EntreSafeZone (_,2,3).into());
	}


	impl_benchmark_test_suite!(Swallower, crate::mock::new_test_ext(), crate::mock::TestRuntime);
}
