//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Swallower;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	set_asset_id {
		let s in 0 .. 100;
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), s)
	verify {
		// assert_eq!(Swallower::asset_id().unwrap() as u32, s);
	}

	impl_benchmark_test_suite!(Swallower, crate::mock::new_test_ext(), crate::mock::TestRuntime);
}
