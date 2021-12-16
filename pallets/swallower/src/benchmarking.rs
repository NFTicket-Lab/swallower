//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Swallower;
// use crate::mock::{TestRuntime, Origin};
use frame_benchmarking::{benchmarks,benchmarks_instance_pallet, whitelisted_caller, account};
use frame_system::RawOrigin as SystemOrigin;
use sp_runtime::traits::StaticLookup;
use frame_support::{pallet_prelude::*};
use frame_system::{pallet_prelude::*};
const SEED: u32 = 0;
benchmarks!{
	set_asset_id {
		let s in 0 .. 100;
		let caller: T::AccountId = whitelisted_caller();
		let user: T::AccountId = account("user", 2, SEED);
		let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user.clone());
		Swallower::<T>::set_admin(SystemOrigin::Root.into(),user_lookup).unwrap();
	}: _(SystemOrigin::Signed(user), s)
	verify {
		let asset_id = Swallower::<T>::asset_id().unwrap();
		let s = AssetIdOf::<T>::decode(&mut (AsRef::<[u8]>::as_ref(&s.encode()))).unwrap();
		assert_eq!(asset_id, s);
	}

	impl_benchmark_test_suite!(Swallower, crate::mock::new_test_ext(), crate::mock::TestRuntime);
}
