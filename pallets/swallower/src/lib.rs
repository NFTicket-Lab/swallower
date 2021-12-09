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
	use frame_support::traits::Randomness;
	use frame_support::traits::tokens::fungibles::Inspect;
	// use mock::Swallower;
	use pallet_assets::{self as assets};
	use frame_support::{pallet_prelude::*, dispatch::DispatchResult, transactional};
	use frame_system::{pallet_prelude::*, ensure_signed};
	use sp_io::hashing::blake2_128;
	use crate::types::Swallower;
	use frame_support::inherent::Vec;
	use sp_runtime::{ArithmeticError, DispatchError};
	use frame_support::traits::tokens::fungibles;
	use frame_support::traits::tokens::fungibles::Transfer;
	use frame_support::sp_runtime::traits::Hash;

	pub(crate) type AssetBalanceOf<T> =	<<T as Config>::AssetsTransfer as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type AssetIdOf<T> = <<T as Config>::AssetsTransfer as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
	// type EngeSwallower<T> = Swallower<BoundedVec<u8,<T as assets::Config>::StringLimit>>;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + assets::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		//The max length of the gene name.
		#[pallet::constant]
		type InitGeneLimit:Get<u32>;

		type AssetsTransfer:fungibles::Transfer<<Self as frame_system::Config>::AccountId>;

		type GeneRandomness:Randomness<Self::Hash,Self::BlockNumber>;
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

	// 吞噬者序号。
	#[pallet::storage]
	#[pallet::getter(fn swallower_no)]
	pub type SwallowerNo<T> = StorageValue<_,u64,ValueQuery>;

	// pallet拥有的代币数量,这里只是记个数量。实际的代币存放在管理员处。由管理员负责转出转入。
	#[pallet::storage]
	#[pallet::getter(fn asset_amount)]
	pub type AssetAmount<T> = StorageValue<_,u64,ValueQuery,GetDefault>;

	// 设置支付币种。
	#[pallet::storage]
	#[pallet::getter(fn asset_id)]
	pub type AssetId<T> = StorageValue<_,AssetIdOf<T>>;

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
		SetAssetId(AssetIdOf<T>),
		Mint(T::AccountId,Vec<u8>,AssetIdOf<T>,AssetBalanceOf<T>,u64),
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
		NotExistAssetId,
		NotEnoughMoney, //用户金额不足
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
			//检查名字是否过长。

			let gene_amount:u64 = GeneAmount::<T>::get();
			//获取系统总的代币数量.
			let asset_amount:u64 = AssetAmount::<T>::get();
			let mut price_gene = 1;
			if gene_amount!=0&&asset_amount!=0{
				price_gene = asset_amount.checked_div(gene_amount).ok_or(ArithmeticError::DivisionByZero)?;
			}
			let init_gene_len = T::InitGeneLimit::get();
			let price_swallower = (init_gene_len as u64).checked_mul(price_gene).ok_or(ArithmeticError::Overflow)?;
			let price_swallower:AssetBalanceOf<T> = price_swallower.try_into().map_err(|_|ArithmeticError::Overflow)?;

			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			//检查用户账户是否有足够的金额。
			let balance_user = T::AssetsTransfer::balance(asset_id,&who);
			if balance_user<price_swallower{
				return Err(Error::<T>::NotEnoughMoney)?;
			}

			let swallower = Self::mint(who,name,asset_id,price_swallower);
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
		pub fn set_asset_id(origin:OriginFor<T>,asset_id:AssetIdOf<T>)->DispatchResult{
			let sender = ensure_signed(origin)?;
			let manager = Manager::<T>::get().ok_or(Error::<T>::NotExistManager)?;
			if sender!=manager{
				return Err(Error::<T>::NotManager)?;
			}
			AssetId::<T>::set(Some(asset_id));
			Self::deposit_event(Event::<T>::SetAssetId(asset_id));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T>{
		/// 增发一个吞噬者
		/// minter 增发的用户
		/// name 吞噬者的名称，首次给名字免费
		/// asset_id 增发吞噬者需要使用的资产id
		/// price 制造一个吞噬者需要的金额。
		/// init_gene_len 吞噬者初始基因的长度。
		/// 1. 支付指定的费用（ = 初始基因数×单基因价格）可以铸造一个基因吞噬者；
		///		2. 吞噬者铸造的时候会有一个初始的基因片段，初始基因片段为 15 位，铸造者需要按照基因价格支付铸造费（铸造费是系统代币，需要通过主链代币兑换得到）；
		///		1. 基因价格 = 系统总收取代币数量 ÷ 系统总基因数量
		///		2. 基因价格初始为  1 ；
		///	3. 铸造者可以指定吞噬者的名称，只要该名称不和现有吞噬者重复即可；
		#[transactional]
		fn mint(minter:T::AccountId,name:Vec<u8>,asset_id:AssetIdOf<T>,price:AssetBalanceOf<T>)->Result<Swallower, DispatchError>{
			let manager = Manager::<T>::get().ok_or(Error::<T>::NotExistManager)?;
			//从增发者的账户转账给管理员.
			T::AssetsTransfer::transfer(asset_id,&minter,&manager,price,true)?;
			let dna = Self::gen_dna();
			// 记录吞噬者序号
			let swallower_no:u64 = Self::swallower_no();
			let swallower_no = swallower_no.saturating_add(1);
			//增加系统中吞噬者的数量.
			SwallowerNo::<T>::set(swallower_no);
			//增发一个吞噬者给购买者.
			let swallower = Swallower::new(name.clone(),dna,swallower_no);
			//发送一个吞噬者增发成功事件
			Self::deposit_event(Event::<T>::Mint(minter,name,asset_id,price,swallower_no));
			//增加系统中吞噬者的基因数量.
			GeneAmount::<T>::mutate(|g|(*g).saturating_add(dna.len() as u64));
			//增加系统中币的总数量
			AssetAmount::<T>::mutate(|a|*a+1);
			//吞噬者生成hash值.
			let swallower_id = T::Hashing::hash_of(&swallower);
			// swallower.using_encoded(blake2_128);
			//记录用户拥有这个吞噬者
			//记录该hash值对应的吞噬者实体.
			Ok(swallower)
		}

		// ACTION #6: function to randomly generate DNA
		fn gen_dna()->[u8;16]{
			let payload = (
				T::GeneRandomness::random(b"dna").0,
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}
	}
}
