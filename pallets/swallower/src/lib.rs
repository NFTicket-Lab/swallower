#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use pallet_assets::{self as assets};
	use frame_support::{pallet_prelude::*, dispatch::DispatchResult, transactional};
	use frame_system::{pallet_prelude::*, ensure_signed, EnsureRoot};
	use frame_support::BoundedVec;
	use crate::types::Swallower;
	use frame_support::inherent::Vec;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + assets::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		//The max length of the gene name.
		#[pallet::constant]
		type InitGeneLimit:Get<u32>;
	}


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	#[pallet::storage]
	#[pallet::getter(fn gene_price)]
	pub type GenePrice<T> = StorageValue<_, u32,ValueQuery,GetDefault>;

	// 基因总数,每次增发或者消除一个基因，需要修改系统基因总数。初始值为0
	#[pallet::storage]
	#[pallet::getter(fn gene_amount)]
	pub type GeneAmount<T> = StorageValue<_,u64,ValueQuery,GetDefault>;

	// 设置支付币种。
	#[pallet::storage]
	#[pallet::getter(fn asset_id)]
	pub type AssetId<T> = StorageValue<_,u64>;

	//设置管理员
	#[pallet::storage]
	#[pallet::getter(fn manager)]
	pub type Manager<T> = StorageValue<_,<T as frame_system::Config>::AccountId>;
	
	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		// SomethingStored(u32, T::AccountId),
		SetManager(T::AccountId),
		SetAssetId(u32),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		NotManager,
		NotExistManager,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// mint swallower
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn mint_swallower(origin:OriginFor<T>,name:Vec<u8>)->DispatchResult{
			let who = ensure_signed(origin)?;
			let swallower:Swallower<BoundedVec<u8,<T as assets::Config>::StringLimit>> = Self::mint(who,name);
			Ok(())
		}

		/// 设置管理员
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn set_manager(origin:OriginFor<T>,manager:T::AccountId)->DispatchResult{
			ensure_root(origin)?;
			Manager::<T>::set(Some(manager.clone()));
			Self::deposit_event(Event::<T>::SetManager(manager));
			Ok(())
			// let who = ensure_signed(origin)?;
			// if let Some(origin_manager) = Manager::<T>::get(){
			// 	if who == origin_manager{
			// 		Manager::<T>::set(Some(manager));
			// 	}else{
			// 		return Err(Error::<T>::NotManager)?;
			// 	}
			// }else{
			// 	return Err(Error::<T>::NotExistManager)?;
			// }
			// Ok(())
		}

		/// 设置币种
		#[transactional]
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn set_asset_id(origin:OriginFor<T>,asset_id:u32)->DispatchResult{
			let sender = ensure_signed(origin)?;
			let manager = Manager::<T>::get().ok_or(Error::<T>::NotExistManager)?;
			if sender!=manager{
				return Err(Error::<T>::NotExistManager)?;
			}
			AssetId::<T>::set(Some(asset_id as u64));
			Self::deposit_event(Event::<T>::SetAssetId(asset_id));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T>{
		/// 1. 支付指定的费用（ = 初始基因数×单基因价格）可以铸造一个基因吞噬者；
		///		2. 吞噬者铸造的时候会有一个初始的基因片段，初始基因片段为 15 位，铸造者需要按照基因价格支付铸造费（铸造费是系统代币，需要通过主链代币兑换得到）；
		///		1. 基因价格 = 系统总收取代币数量 ÷ 系统总基因数量
		///		2. 基因价格初始为  1 ；
		///	3. 铸造者可以指定吞噬者的名称，只要该名称不和现有吞噬者重复即可；
		fn mint(who:T::AccountId,name:Vec<u8>)->Swallower<BoundedVec<u8,<T as assets::Config>::StringLimit>>{
			let gene_amount = GeneAmount::<T>::get();
			//获取系统总的代币数量。
			Swallower{ name: todo!(), init_gene: todo!() }
		}
	}
}
