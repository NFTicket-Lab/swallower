#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_support;
extern crate sp_runtime;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use pallet_assets::{self as assets};
	use frame_support::{pallet_prelude::*, dispatch::DispatchResult, transactional};
	use frame_system::{pallet_prelude::*, ensure_signed};
	use frame_support::BoundedVec;
	use crate::types::Swallower;
	use frame_support::inherent::Vec;
	use sp_runtime::{ArithmeticError, DispatchError};
	use frame_support::traits::tokens::fungibles;
	use frame_support::traits::tokens::fungibles::Transfer;

	type _Swallower<T:Config> = Swallower<BoundedVec<u8,<T as assets::Config>::StringLimit>>;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + assets::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		//The max length of the gene name.
		#[pallet::constant]
		type InitGeneLimit:Get<u32>;

		type assets_fun:fungibles::Transfer<T::AccoundId>;
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

	// pallet拥有的代币数量,这里只是记个数量。实际的代币存放在管理员处。由管理员负责转出转入。
	#[pallet::storage]
	#[pallet::getter(fn asset_amount)]
	pub type AssetAmount<T> = StorageValue<_,u64,ValueQuery,GetDefault>;

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
			let swallower = Self::mint(who,name);
			Ok(())
		}

		/// 设置管理员
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn set_manager(origin:OriginFor<T>,manager:T::AccountId)->DispatchResult{
			ensure_root(origin)?;
			Manager::<T>::set(Some(manager.clone()));
			Self::deposit_event(Event::<T>::SetManager(manager));
			Ok(())
		}

		/// 设置币种
		#[transactional]
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn set_asset_id(origin:OriginFor<T>,asset_id:u32)->DispatchResult{
			let sender = ensure_signed(origin)?;
			let manager = Manager::<T>::get().ok_or(Error::<T>::NotExistManager)?;
			if sender!=manager{
				return Err(Error::<T>::NotManager)?;
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
		#[transactional]
		fn mint(who:T::AccountId,name:Vec<u8>)->Result<_Swallower<T>, DispatchError>{
			let gene_amount:u64 = GeneAmount::<T>::get();
			//获取系统总的代币数量.
			let asset_amount:u64 = AssetAmount::<T>::get();
			let mut price_gene = 1;
			if gene_amount!=0&&asset_amount!=0{
				price_gene = asset_amount.checked_div(gene_amount).ok_or(ArithmeticError::DivisionByZero)?;
			}
			let init_gene_len = T::InitGeneLimit::get();
			let price_swallower = (init_gene_len as u64).checked_mul(price_gene).ok_or(ArithmeticError::Overflow)? ;
			//从增发者的账户转账给管理员.
			let manager = Manager::<T>::get().ok_or(Error::<T>::NotExistManager)?;
			let asset_id = AssetId::<T>::get()?;
			<T>::assets_fun::transfer(asset_id,who,manager,price_swallower,true)?;
			//增发一个吞噬者给购买者.
			//发送一个吞噬者增发成功事件
			//增加系统中吞噬者的数量.
			//增加系统中币的总数量

			Ok(Swallower{ name: todo!(), init_gene: todo!() })
		}
	}
}
