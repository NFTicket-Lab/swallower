//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Swallower;
use frame_system::Pallet as System;
use pallet_randomness_collective_flip::Pallet as  CollectiveFlip;
use codec::{FullCodec};
// use crate::mock::{TestRuntime, Origin};
use frame_benchmarking::{benchmarks,benchmarks_instance_pallet, whitelisted_caller, account};
use frame_system::RawOrigin as SystemOrigin;
use sp_runtime::traits::StaticLookup;
use frame_support::{traits::tokens::AssetId, BoundedVec};
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

fn mint_to_facer<T:Config>(){
	let account_asset: T::AccountId = account("asset", ACCOUNT_ASSET_OWNER_ID as u32, SEED);
	let facer: T::AccountId = account("facer", ACCOUNT_ID_2 as u32, SEED);
	let facer_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(facer.clone());
	let transfer_amount = 16000000000000u64.try_into().unwrap_or(Default::default());
	let asset_id:<T as pallet_assets::Config>::AssetId = convert_asset_id(ASSET_ID);
	assert!(pallet_assets::Pallet::<T>::mint(
		SystemOrigin::Signed(account_asset.clone()).into(),
		asset_id,
		facer_lookup.clone(),
		transfer_amount,
	)
	.is_ok());
}


fn create_default_asset<T: Config>(
	is_sufficient: bool,
) -> (T::AccountId, <T::Lookup as StaticLookup>::Source) {
	let account_asset: T::AccountId = account("asset", ACCOUNT_ASSET_OWNER_ID as u32, SEED);
	let account_asset_lookup = T::Lookup::unlookup(account_asset.clone());
	let challenger: T::AccountId = account("challenger", ACCOUNT_ID_1 as u32, SEED);
	// let Bob = account("Bob", ACCOUNT_ID_1 as u32, SEED);
	let challenger_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(challenger.clone());
	let root = SystemOrigin::Root.into();
	let balance:<T as pallet_assets::Config>::Balance = 10u64.try_into().unwrap_or_default();
	let asset_id:<T as pallet_assets::Config>::AssetId = convert_asset_id(ASSET_ID);
	assert!(pallet_assets::Pallet::<T>::force_create(
		root,
		asset_id,
		account_asset_lookup.clone(),
		is_sufficient,
		balance,
	)
	.is_ok());
	assert!(pallet_assets::Pallet::<T>::mint(
		SystemOrigin::Signed(account_asset.clone()).into(),
		asset_id,
		challenger_lookup.clone(),
		17000000000000000000u64.try_into().unwrap_or(Default::default()),
	)
	.is_ok());
	(account_asset, account_asset_lookup)
}


// fn go_block_number<T:Config>(number:u64){
// 	let current_block_number:u32 = System::block_number().into();
// 	for i in current_block_number..current_block_number+number{
// 		// CollectiveFlip::on_initialize(i);
// 		System::set_block_number(i);
// 		let h:[u8;32] = hash69(i as u8);
// 		System::set_parent_hash(h.into());
// 	}
// }

// // Create a Hash with 69 for each byte,
// // only used to build genesis config.
// #[cfg(feature = "std")]
// fn hash69<T: AsMut<[u8]> + Default>(i:u8) -> T {
// 	let mut h = T::default();
// 	h
// 		.as_mut()
// 		.iter_mut()
// 		.for_each(|byte| *byte = i);
// 	h
// }


benchmarks!{
	// set_admin {
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	let admin: T::AccountId = account("admin", ADMIN_ID, SEED);
	// 	let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(admin.clone());
	// }: _(SystemOrigin::Root, user_lookup)
	// verify {
	// 	let admin_id = Swallower::<T>::admin().unwrap();
	// 	assert_eq!(admin_id, admin);
	// 	// assert_eq!(admin_id, caller);
	// }

	// set_asset_id {
	// 	// let s in 0 .. 100;
	// 	// let s = 1;
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	let admin = pre_set_admin::<T>();
	// 	let asset_id:AssetIdOf<T> = convert_asset_id(ASSET_ID);
	// }: _(SystemOrigin::Signed(admin), asset_id)
	// verify {
	// 	let stor_asset_id = Swallower::<T>::asset_id().unwrap();
	// 	// let s = convert_asset_id(s);
	// 	assert_eq!(stor_asset_id, asset_id);
	// }


	// mint_swallower {
	// 	let i in 0 .. 100;
	// 	pre_set_asset_id::<T>();
	// 	let name = vec![0u8;i as usize];
	// 	let challenger: T::AccountId = account("challenger", ACCOUNT_ID_1 as u32, SEED);
	// 	let asset_id:<T as pallet_assets::Config>::AssetId = convert_asset_id(ASSET_ID);
	// 	// let asset_account_balance = pallet_assets::Pallet::<T>::balance(asset_id,&challenger);
	// }: _(SystemOrigin::Signed(challenger.clone()), name)
	// verify {
	// 	let bounded_vec = Swallower::<T>::owner_swallower(&challenger);
	// 	let hash:&<T as frame_system::Config>::Hash = bounded_vec.get(0).unwrap();
	// 	assert_last_event::<T>(Event::EntreSafeZone (*hash,1u32.into(),1601u32.into()).into());
	// }

	// change_swallower_name {
	// 	pre_set_asset_id::<T>();
	// 	let name = b"swallower".to_vec();
	// 	let new_name = b"newName".to_vec();
	// 	let challenger: T::AccountId = account("challenger", ACCOUNT_ID_1 as u32, SEED);
	// 	let asset_id:<T as pallet_assets::Config>::AssetId = convert_asset_id(ASSET_ID);
	// 	Swallower::<T>::mint_swallower(SystemOrigin::Signed(challenger.clone()).into(),name).unwrap();
	// 	let bounded_vec = Swallower::<T>::owner_swallower(&challenger);
	// 	let hash:<T as frame_system::Config>::Hash = bounded_vec.get(0).unwrap().clone();
	// 	// let asset_account_balance = pallet_assets::Pallet::<T>::balance(asset_id,&challenger);
	// }: _(SystemOrigin::Signed(challenger.clone()),hash, new_name.clone())
	// verify {
	// 	assert_last_event::<T>(Event::ChangeName (challenger,new_name,convert_asset_id(ASSET_ID),11u32.into(),hash).into());
	// }

	// burn_swallower {
	// 	pre_set_asset_id::<T>();
	// 	let name = b"swallower".to_vec();
	// 	let challenger: T::AccountId = account("challenger", ACCOUNT_ID_1 as u32, SEED);
	// 	let asset_id:<T as pallet_assets::Config>::AssetId = convert_asset_id(ASSET_ID);
	// 	Swallower::<T>::mint_swallower(SystemOrigin::Signed(challenger.clone()).into(),name).unwrap();
	// 	let bounded_vec = Swallower::<T>::owner_swallower(&challenger);
	// 	let hash:<T as frame_system::Config>::Hash = bounded_vec.get(0).unwrap().clone();


	// 	let account_asset: T::AccountId = account("asset", ACCOUNT_ASSET_OWNER_ID as u32, SEED);
	// 	let manager:T::AccountId = Swallower::<T>::manager();
	// 	let manager_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(manager.clone());
	// 	pallet_assets::Pallet::<T>::mint(
	// 		SystemOrigin::Signed(account_asset.clone()).into(),
	// 		asset_id,
	// 		manager_lookup.clone(),
	// 		16u32.into(),
	// 	).unwrap();
	// 	// let asset_account_balance = pallet_assets::Pallet::<T>::balance(asset_id,&challenger);
	// }: _(SystemOrigin::Signed(challenger.clone()),hash)
	// verify {
	// 	assert_last_event::<T>(Event::Burn(challenger,convert_asset_id(ASSET_ID),15u32.into(),hash).into());
	// }

	// make_battle {
	// 	pre_set_asset_id::<T>();
	// 	mint_to_facer::<T>();
	// 	let challenge_name = b"challenge_name".to_vec();
	// 	let facer_name = b"facer_name".to_vec();
	// 	let challenger: T::AccountId = account("challenger", ACCOUNT_ID_1 as u32, SEED);
	// 	let facer: T::AccountId = account("facer", ACCOUNT_ID_2 as u32, SEED);
	// 	let facer_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(facer.clone());
	// 	let asset_id:<T as pallet_assets::Config>::AssetId = convert_asset_id(ASSET_ID);
	// 	Swallower::<T>::mint_swallower(SystemOrigin::Signed(challenger.clone()).into(),challenge_name).unwrap();
	// 	Swallower::<T>::mint_swallower(SystemOrigin::Signed(facer.clone()).into(),facer_name).unwrap();
	// 	let bounded_vec = Swallower::<T>::owner_swallower(&challenger);
	// 	let challenger_hash:<T as frame_system::Config>::Hash = bounded_vec.get(0).unwrap().clone();
	// 	let bounded_vec = Swallower::<T>::owner_swallower(&facer);
	// 	let facer_hash:<T as frame_system::Config>::Hash = bounded_vec.get(0).unwrap().clone();

	// 	Swallower::<T>::user_exit_safe_zone(SystemOrigin::Signed(challenger.clone()).into(),challenger_hash).unwrap();
	// 	Swallower::<T>::user_exit_safe_zone(SystemOrigin::Signed(facer.clone()).into(),facer_hash).unwrap();

	// }: _(SystemOrigin::Signed(challenger.clone()),challenger_hash,facer_hash)
	// verify {
	// 	assert_last_event::<T>(Event::BattleResult(false,vec!(151, 219, 9),vec!(48, 208, 193, 215, 231, 235, 106),vec!()).into());
	// }

	user_entre_safe_zone {
		pre_set_asset_id::<T>();
		mint_to_facer::<T>();
		let challenge_name = b"challenge_name".to_vec();
		let challenger: T::AccountId = account("challenger", ACCOUNT_ID_1 as u32, SEED);
		let asset_id:<T as pallet_assets::Config>::AssetId = convert_asset_id(ASSET_ID);
		Swallower::<T>::mint_swallower(SystemOrigin::Signed(challenger.clone()).into(),challenge_name).unwrap();
		let bounded_vec = Swallower::<T>::owner_swallower(&challenger);
		let challenger_hash:<T as frame_system::Config>::Hash = bounded_vec.get(0).unwrap().clone();
		Swallower::<T>::user_exit_safe_zone(SystemOrigin::Signed(challenger.clone()).into(),challenger_hash).unwrap();
	}: _(SystemOrigin::Signed(challenger.clone()),challenger_hash,1000u32.into())
	verify {
		assert_last_event::<T>(Event::EntreSafeZone(challenger_hash,1u32.into(),1001u32.into()).into());
	}

	user_exit_safe_zone {
		pre_set_asset_id::<T>();
		mint_to_facer::<T>();
		let challenge_name = b"challenge_name".to_vec();
		let challenger: T::AccountId = account("challenger", ACCOUNT_ID_1 as u32, SEED);
		let asset_id:<T as pallet_assets::Config>::AssetId = convert_asset_id(ASSET_ID);
		Swallower::<T>::mint_swallower(SystemOrigin::Signed(challenger.clone()).into(),challenge_name).unwrap();
		let bounded_vec = Swallower::<T>::owner_swallower(&challenger);
		let challenger_hash:<T as frame_system::Config>::Hash = bounded_vec.get(0).unwrap().clone();
	}: _(SystemOrigin::Signed(challenger.clone()),challenger_hash)
	verify {
		assert_last_event::<T>(Event::ExitZone(challenger_hash,1u32.into()).into());
	}

	user_claim_reward_in_battle_zone {
		pre_set_asset_id::<T>();
		mint_to_facer::<T>();
		let challenge_name = b"challenge_name".to_vec();
		let challenger: T::AccountId = account("challenger", ACCOUNT_ID_1 as u32, SEED);
		let asset_id:<T as pallet_assets::Config>::AssetId = convert_asset_id(ASSET_ID);
		Swallower::<T>::mint_swallower(SystemOrigin::Signed(challenger.clone()).into(),challenge_name).unwrap();
		let bounded_vec = Swallower::<T>::owner_swallower(&challenger);
		let challenger_hash:<T as frame_system::Config>::Hash = bounded_vec.get(0).unwrap().clone();
		Swallower::<T>::user_exit_safe_zone(SystemOrigin::Signed(challenger.clone()).into(),challenger_hash).unwrap();
	}: _(SystemOrigin::Signed(challenger.clone()),challenger_hash)
	verify {
		assert_last_event::<T>(Event::BattleZoneReward(challenger_hash, 1u32.into(), 1u32.into()).into());
	}

	
	impl_benchmark_test_suite!(Swallower, crate::mock::new_test_ext(), crate::mock::TestRuntime);
}
