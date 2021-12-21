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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

mod types;

#[frame_support::pallet]
pub mod pallet {


use frame_support::{storage::bounded_btree_map::BoundedBTreeMap, traits::fungibles::InspectMetadata};
	use frame_support::{Twox64Concat, ensure};
	use frame_support::pallet_prelude::{ValueQuery, OptionQuery};
	use frame_support::traits::{Randomness};
	use sp_runtime::offchain::storage::StorageValueRef;
use sp_runtime::traits::{CheckedDiv,CheckedMul,CheckedAdd, StaticLookup, Saturating, CheckedSub};
	use frame_support::traits::tokens::fungibles::Inspect;
	use pallet_assets::{self as assets};
	use frame_support::{pallet_prelude::*, dispatch::DispatchResult, transactional};
	use frame_system::{pallet_prelude::*, ensure_signed};
	use sp_io::hashing::blake2_128;
	use crate::types::{Swallower, FeeConfig, ProtectState, ProtectConfig, TransInfo};
	use crate::weights::WeightInfo;
	use frame_support::inherent::Vec;
	use sp_runtime::{ArithmeticError, DispatchError};
	use frame_support::traits::tokens::{fungibles};
	use frame_support::traits::tokens::fungibles::Transfer;
	use frame_support::sp_runtime::traits::Hash;
	// use sp_runtime::traits::Hash;
	pub(crate) type AssetBalanceOf<T> =	<<T as Config>::AssetsTransfer as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type AssetIdOf<T> = <<T as Config>::AssetsTransfer as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type TransInfoMessage<'a,T> = TransInfo<'a ,AssetIdOf<T>,AccountIdOf<T>,AssetBalanceOf<T>>;
	// type EngeSwallower<T> = Swallower<BoundedVec<u8,<T as assets::Config>::StringLimit>>;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	const RATIO:u32 = 100;
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	// #[pallet::storage]
	// #[pallet::getter(fn gene_price)]
	// pub type GenePrice<T> = StorageValue<_, u32,ValueQuery,GetDefault>;

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
	pub type AssetAmount<T> = StorageValue<_,AssetBalanceOf<T>,ValueQuery,GetDefault>;

	// 设置游戏配置
	#[pallet::storage]
	#[pallet::getter(fn swallower_config)]
	pub type SwallowerConfig<T> = StorageValue<_,FeeConfig,ValueQuery>;
	// 保护区配置
	#[pallet::storage]
	#[pallet::getter(fn protect_zone_config)]
	pub type ProtectZoneConfig<T> = StorageValue<_,ProtectConfig,ValueQuery>;

	// 设置支付币种。
	#[pallet::storage]
	#[pallet::getter(fn asset_id)]
	pub type AssetId<T> = StorageValue<_,AssetIdOf<T>>;

	// 设置管理员账户。
	#[pallet::storage]
	#[pallet::getter(fn admin)]
	pub type Admin<T> = StorageValue<_,<T as frame_system::Config>::AccountId>;

	//设置资金管理员,资金管理账号应为无私钥账户，不可提走资金。
	#[pallet::storage]
	#[pallet::getter(fn manager)]
	pub type Manager<T> = StorageValue<_,<T as frame_system::Config>::AccountId,ValueQuery>;

	//用户拥有的吞噬者hash队列
	#[pallet::storage]
	#[pallet::getter(fn owner_swallower)]
	pub type OwnerSwallower<T:Config> = StorageMap<_,Twox64Concat,T::AccountId,BoundedVec<T::Hash,T::MaxSwallowerOwen>,ValueQuery>;

	// hash值对应的swallower对象
	#[pallet::storage]
	#[pallet::getter(fn swallowers)]
	pub type Swallowers<T:Config> = StorageMap<_,Twox64Concat,T::Hash,Swallower<T::AccountId>>;

	//保护区,如果该map中存在该吞噬者，则吞噬者处于保护中。
	#[pallet::storage]
	#[pallet::getter(fn safe_zone)]
	pub type SafeZone<T:Config> = StorageMap<_,Twox64Concat,T::Hash,ProtectState<T::BlockNumber>>;


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		// SomethingStored(u32, T::AccountId),
		SetAdmin(T::AccountId),
		SetAssetId(AssetIdOf<T>),
		Mint(T::AccountId,Vec<u8>,AssetIdOf<T>,AssetBalanceOf<T>,T::Hash),
		Burn(T::AccountId,AssetIdOf<T>,AssetBalanceOf<T>,T::Hash),
		ChangeName(T::AccountId,Vec<u8>,AssetIdOf<T>,AssetBalanceOf<T>,T::Hash),
		EntreSafeZone(T::Hash,T::BlockNumber,T::BlockNumber),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		NotAdmin,
		NotExistAdmin,
		NotExistAssetId,
		NotEnoughMoney, //用户金额不足
		ExceedMaxSwallowerOwned,
		NameRepeated,
		NotOwner,
		SwallowerNotExist,
		SwallowerInSafeZone,
		WithSelf, //不能和自己交易。
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T:Config>{
		// config:Vec<(Option<T::AccountId>,Option<AssetIdOf<T>>)>,
		pub admin:Option<T::AccountId>,
		pub asset_id:Option<u32>,
		// pub asset_id:Option<Box<AssetIdOf<T>>>,
	}

	#[cfg(feature = "std")]
	impl<T:Config> Default for GenesisConfig<T>{
		fn default() -> Self {
			GenesisConfig{
				admin:None,
				asset_id:None,
				// asset_id:None,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T:Config> GenesisBuild<T> for GenesisConfig<T>{
		fn build(&self) {
			if let Some(m) = &self.admin{
				Admin::<T>::set(Some(m.clone()));
			}
			if let Some(asset_id) = self.asset_id{
				let asset_id = AssetIdOf::<T>::decode(&mut (AsRef::<[u8]>::as_ref(&asset_id.encode()))).unwrap();
				AssetId::<T>::set(Some(asset_id));
			}

		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + assets::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		//The max length of the gene name.
		#[pallet::constant]
		type InitGeneLimit:Get<u32>;

		type AssetsTransfer:fungibles::Transfer<AccountIdOf<Self>>+InspectMetadata<AccountIdOf<Self>>;

		type GeneRandomness:Randomness<Self::Hash,Self::BlockNumber>;

		// type MyAssetId:frame_support::traits::tokens::misc::AssetId+MaybeSerializeDeserialize;

		#[pallet::constant]
		type MaxSwallowerOwen:Get<u32>;

		type SwallowerWeightInfo: WeightInfo;
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// 修改swallower名称
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn change_swallower_name(origin:OriginFor<T>, hash:T::Hash, name:Vec<u8>) ->DispatchResult{
			let sender = ensure_signed(origin)?;
			// 判断用户是否拥有这个swallower。
			let swallowers:BoundedVec<T::Hash,_> = OwnerSwallower::<T>::get(&sender);
			ensure!(swallowers.contains(&hash),Error::<T>::NotOwner);
			ensure!(!Self::check_exist_name(&name),Error::<T>::NameRepeated);
			//得到费用配置。
			let change_name_fee_config = SwallowerConfig::<T>::get().change_name_fee;
			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			let decimal = T::AssetsTransfer::decimals(&asset_id);
			let change_name_fee = change_name_fee_config.saturating_mul(10u64.pow(decimal as u32));
			let change_name_fee = change_name_fee.try_into().map_err(|_|ArithmeticError::Overflow)?;
			// 检查用户资金是否充足
			let balance_user = T::AssetsTransfer::balance(asset_id,&sender);
			if balance_user<change_name_fee{
				return Err(Error::<T>::NotEnoughMoney)?;
			}
			Self::change_name(sender,name, hash ,asset_id,change_name_fee)?;
			Ok(())
		}
		
		/// mint swallower
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn mint_swallower(origin:OriginFor<T>,name:Vec<u8>)->DispatchResult{
			let who = ensure_signed(origin)?;
			// TODO 检查名字是否过长。
			//检查名字是否重复。
			ensure!(!Self::check_exist_name(&name),Error::<T>::NameRepeated);
			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			// let gene_amount:u64 = GeneAmount::<T>::get();
			// //获取系统总的代币数量.
			// let asset_amount = AssetAmount::<T>::get();
			// let decimal = T::AssetsTransfer::decimals(&asset_id);
			let price_gene = Self::gene_price()?;
			let init_gene_len = T::InitGeneLimit::get();
			log::info!("init_gene_len is:{}",init_gene_len);
			let price_swallower = price_gene.checked_mul(&init_gene_len.try_into().map_err(|_|ArithmeticError::Overflow)?).ok_or(ArithmeticError::Overflow)?;
			let price_swallower:AssetBalanceOf<T> = price_swallower.try_into().map_err(|_|ArithmeticError::Overflow)?;


			//检查用户账户是否有足够的金额。
			let balance_user = T::AssetsTransfer::balance(asset_id,&who);
			if balance_user<price_swallower{
				return Err(Error::<T>::NotEnoughMoney)?;
			}

			Self::mint(who,name,asset_id,price_swallower)?;
			Ok(())
		}

		// 销毁swallower
		// 1. 基因吞噬者的拥有者可以通过主动销毁基因吞噬者，按照当前当前吞噬者的基因数量和当前基因价格获得代币返还，返还时需要扣除 3% 的手续费；
        // 1. 返还代币数 = 吞噬者基因数 × 基因价格 × 97%；
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn burn_swallower(origin:OriginFor<T>, hash:T::Hash) ->DispatchResult{
			let sender = ensure_signed(origin)?;
			log::info!(target:"swallower","burn sender is:{:?}",&sender);
			// 判断swallower的所有权。
			let swallowers:BoundedVec<T::Hash,_> = OwnerSwallower::<T>::get(&sender);
			ensure!(swallowers.contains(&hash),Error::<T>::NotOwner);
			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			//得到当前基因的价格。
			let price_gene = Self::gene_price()?;
			//得到费用配置。
			let swallower_config = Self::swallower_config();
			
			// 得到吞噬者基因数。
			let swallower_gene_count = Self::swallowers(&hash).ok_or(Error::<T>::SwallowerNotExist)?.gene.len();
			let return_balance = price_gene
				.checked_mul(&swallower_gene_count.try_into()
				.map_err(|_|ArithmeticError::Overflow)?)
				.ok_or(ArithmeticError::Overflow)?;
			// 需要扣除3%的费用。
			let return_balance = return_balance
				.saturating_mul((RATIO-swallower_config.destroy_fee_percent).into())
				.checked_div(&RATIO.into())
				.ok_or(ArithmeticError::Overflow)?;
			// 检查用户资金是否充足
			let manager = Self::manager();
			let balance_manager = T::AssetsTransfer::balance(asset_id,&manager);
			if balance_manager<return_balance{
				return Err(Error::<T>::NotEnoughMoney)?;
			}
			Self::burn(sender,hash ,asset_id,return_balance)?;
			Ok(())
		}

		/// 设置管理员
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn set_admin(origin:OriginFor<T>,admin:<T::Lookup as StaticLookup>::Source)->DispatchResult{
			ensure_root(origin)?;
			let admin = T::Lookup::lookup(admin)?;
			Admin::<T>::set(Some(admin.clone()));
			Self::deposit_event(Event::<T>::SetAdmin(admin));
			Ok(())
		}

		/// 设置币种
		#[transactional]
		#[pallet::weight(T::SwallowerWeightInfo::set_asset_id(*asset_id))]
		pub fn set_asset_id(origin:OriginFor<T>,asset_id:u32)->DispatchResult{
			let sender = ensure_signed(origin)?;
			let admin = Admin::<T>::get().ok_or(Error::<T>::NotExistAdmin)?;
			if sender!=admin{
				return Err(Error::<T>::NotAdmin)?;
			}
			let asset_id = AssetIdOf::<T>::decode(&mut (AsRef::<[u8]>::as_ref(&asset_id.encode()))).unwrap();
			AssetId::<T>::set(Some(asset_id));
			Self::deposit_event(Event::<T>::SetAssetId(asset_id));
			Ok(())
		}

	// 	4. 吞噬挑战
    // 1. 吞噬者可以向其他吞噬者发起挑战，从而获得其基因；
    // 2. 发起挑战，需要支付代币，所有代币将投放进入总的代币池；
    //     1. 挑战费用 = 基因价格 × 挑战费系数
	// challenger 发起挑战的吞噬者者,
	// facer 应战的吞噬者
	// 吞噬挑战
	// 吞噬者可以向其他吞噬者发起挑战，从而获得其基因；
	// 发起挑战，需要支付代币，所有代币将投放进入总的代币池；
	// 挑战费用 = 基因价格 × 挑战费系数
		#[pallet::weight(10_000)]
		pub fn make_battle(origin:OriginFor<T>,challenger:T::Hash,facer:T::Hash)->DispatchResult{
			//检查两个吞噬者不能是同一个owner。不能自己人打自己人。
			let sender = ensure_signed(origin)?;
			let facer_swallower = Swallowers::<T>::get(&facer).ok_or(Error::<T>::SwallowerNotExist)?;
			let challenger_swallower = Swallowers::<T>::get(&facer).ok_or(Error::<T>::SwallowerNotExist)?;
			log::info!("facer_swallower owner is:{:?}",facer_swallower.owner);
			ensure!(sender!=facer_swallower.owner.clone().unwrap(),Error::<T>::WithSelf);
			// 检查能否开战。如果挑战者和被挑战者其中一个在安全区都不能开战。
			// 判断挑战者是否在安全区,如果在安全区,但是已经超时了,需要将该吞噬者移除安全区.
			let is_in_safe = Self::check_in_safe_zone(challenger);
			let is_in_safe_facer = Self::check_in_safe_zone(facer);
			// let in_safe_zone = SafeZone::<T>::iter_keys().any(|hash|hash==challenger||hash==facer);
			if is_in_safe{
				return Err(Error::<T>::SwallowerInSafeZone.into());
			}
			if is_in_safe_facer{
				return Err(Error::<T>::SwallowerInSafeZone.into());
			}

			// 计算发起挑战需要支付的费用。
			let price_gene = Self::gene_price()?;
			let fee_config = Self::swallower_config();
			let challenge_fee_ratio:AssetBalanceOf<T> = fee_config.challenge_fee_ratio
			.try_into()
			.map_err(|_|ArithmeticError::Overflow)?;
			let challenge_fee = price_gene.saturating_mul(challenge_fee_ratio);
			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			let sender_balance = T::AssetsTransfer::balance(asset_id,&sender);
			if sender_balance < challenge_fee {
				return Err(Error::<T>::NotEnoughMoney.into());
			}
			
			// 可以开战，立即开始对打。
			let manager = Self::manager();
			let trans_info = TransInfo::new(asset_id,&sender,&manager,challenge_fee);
			Self::fight(challenger_swallower,facer_swallower,&trans_info)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T>{
		//开战
		// 通过随机数，从两个对战吞噬者中挑选一段基因进行对战；
		// 挑选位置通过随机数确定；
		// 挑选的长度是基因最短的吞噬者的基因长度和指定长度（默认16）中的最小；
		// 比如基因最短的吞噬者的基因长度是 12 则按 12 位来取；
		// 如果基因最短的吞噬者的基因长度是 22，则按 16 位来取；
		// 挑战就是比较相应基因位的上的基因，比较方式如下：
		// 假设一个基因位的基因有 256 个，基因是256个数字，那么将 256个数字按照顺时针，从小到大围成一个圈；
		// 如果某个基因距离另外一个基因的更长，则该基因胜出；
		// 基因数字较大者的距离 = 256 - 大数 +小数；
		// 基因数字较小者的距离 = 大数 - 下数；
		// 如果距离相等，或者两个基因完全相同，则平手；
		#[transactional]
		fn fight(challenger:Swallower<T::AccountId>,facer:Swallower<T::AccountId>,trans_info:&TransInfoMessage<T>)->DispatchResult{
			//收取的战斗费用转账给基金池
			T::AssetsTransfer::transfer(trans_info.asset_id,trans_info.sender,trans_info.manager,trans_info.challenge_fee,true)?;
			// 生成随机战斗数组。
			let random = T::GeneRandomness::random(b"battle").0;
			let random_ref:&[u8] = random.as_ref();
			let random_len = random_ref.len();
			log::info!("random_len is:{}",random_len);
			let challenge_gene_len = challenger.gene.len();
			let facer_gene_len = facer.gene.len();
			// 1.选择开始位置进行
			let min_length = challenge_gene_len.min(facer_gene_len).min(16);
			let start_position = random_ref[0] as usize % min_length;
			Ok(())
		}

		//检查用户是否在安全区.
		fn check_in_safe_zone(hash:T::Hash)->bool{
			// 检查map中是否有该hash存在.
			if let Some(protect_state)=SafeZone::<T>::get(&hash){
				log::info!("protect_state is:{:?}",protect_state);
				if protect_state.end_block >= frame_system::Pallet::<T>::block_number(){
					return true;
				}else{
					// 删除该hash
					SafeZone::<T>::remove(hash);
					return false;
				}
			}else{
				return false;
			}
		}
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
		fn mint(minter:T::AccountId,name:Vec<u8>,asset_id:AssetIdOf<T>,price:AssetBalanceOf<T>)->Result<(), DispatchError>{
			let manager = Manager::<T>::get();
			//从增发者的账户转账给管理员.
			T::AssetsTransfer::transfer(asset_id,&minter,&manager,price,true)?;
			let dna = Self::gen_dna();
			// 记录吞噬者序号
			let swallower_no:u64 = Self::swallower_no();
			let swallower_no = swallower_no.saturating_add(1);
			//增加系统中吞噬者的数量.
			SwallowerNo::<T>::set(swallower_no);
			//增发一个吞噬者给购买者.
			let swallower = Swallower::<T::AccountId>::new(name.clone(),dna,swallower_no,minter.clone());

			//吞噬者生成hash值.
			let swallower_hash = T::Hashing::hash_of(&swallower);
			//记录用户拥有这个吞噬者
			OwnerSwallower::<T>::try_mutate(&minter, |swallower_vec|{
				swallower_vec.try_push(swallower_hash)
			}).map_err(|_|Error::<T>::ExceedMaxSwallowerOwned)?;
			//记录该hash值对应的吞噬者实体.
			Swallowers::<T>::insert(swallower_hash, swallower.clone());

			//发送一个吞噬者增发成功事件
			Self::deposit_event(Event::<T>::Mint(minter.clone(),name,asset_id,price,swallower_hash));
			//增加系统中吞噬者的基因数量.
			GeneAmount::<T>::mutate(|g|*g=(*g).saturating_add(dna.len() as u64));
			//增加系统中币的总数量
			AssetAmount::<T>::try_mutate(|a|{
				*a = match a.checked_add(&price){
					Some(p)=>p,
					None=>return Err(ArithmeticError::Overflow),
				};
				return Ok(())
			})?;

			let start_block = frame_system::Pallet::<T>::block_number();
			let auto_protect_duration = Self::protect_zone_config().first_mint_protect_duration;
			let end_block = start_block.saturating_add(auto_protect_duration.into());
			
			// 自动进入保护区
			Self::entre_safe_zone(swallower_hash,start_block,end_block)?;

			Ok(())
		}

		// 进入安全区
		#[transactional]
		fn entre_safe_zone(swallower_hash:T::Hash,start_block:T::BlockNumber,end_block:T::BlockNumber)->DispatchResult{
			let protect_state = ProtectState::new(start_block, end_block);
			// let end_safe_map = StorageValueRef::local(b"ocw-swallower::end-safe");
			// let get =end_safe_map.get();
			// end_safe_map.mutate::<BoundedBTreeMap<T::BlockNumber,T::Hash,T::MaxSwallowerOwen>,Error::<T>,_>(|v|{
			// 	let btree_map = v.unwrap().unwrap();
			// 	// let mut bm=btree_map.; 
			// 	(*btree_map).insert(end_block, swallower_hash);
			// 	Ok(btree_map)
			// });
			// TODO 添加进入安全区事件。
			SafeZone::<T>::insert(swallower_hash, protect_state);
			Self::deposit_event(Event::<T>::EntreSafeZone(swallower_hash,start_block,end_block));
			Ok(())
		}

		// 退出安全区
		fn exit_safe_zone(swallower_hash:T::Hash)->DispatchResult{
			SafeZone::<T>::remove(swallower_hash);
			Ok(())
		}

		#[transactional]
		fn burn(sender:T::AccountId,swallower_hash:T::Hash,asset_id:AssetIdOf<T>,return_balance:AssetBalanceOf<T>)->Result<(), DispatchError>{
			let manager = Manager::<T>::get();
			//从管理员转账给销毁的用户
			T::AssetsTransfer::transfer(asset_id,&manager,&sender,return_balance,true)?;
			// // 记录吞噬者序号
			// let swallower_no:u64 = Self::swallower_no();
			// let swallower_no = swallower_no.saturating_sub(1);
			// //增加系统中吞噬者的数量.
			// SwallowerNo::<T>::set(swallower_no);

			//删除用户拥有这个吞噬者
			OwnerSwallower::<T>::mutate(&sender, |swallower_vec|{
				if let Some((index,_)) = swallower_vec
					.iter()
					.enumerate()
					.find(|(_i,h)|**h==swallower_hash){
						swallower_vec.remove(index);
					}
				// Ok(())
			});
			//删除该hash值对应的吞噬者实体.
			let swallower = Swallowers::<T>::take(swallower_hash).ok_or(Error::<T>::SwallowerNotExist)?;
			//减少系统中吞噬者的基因数量.
			GeneAmount::<T>::mutate(|g|*g=(*g).saturating_sub(swallower.gene.len() as u64));
			//增加系统中币的总数量
			AssetAmount::<T>::try_mutate(|a|{
				*a = match a.checked_sub(&return_balance){
					Some(p)=>p,
					None=>return Err(ArithmeticError::Overflow),
				};
				return Ok(())
			})?;


			//退出安全区
			SafeZone::<T>::remove(swallower_hash);
			// TODO 退出战斗区。

			//发送一个吞噬者销毁事件
			Self::deposit_event(Event::<T>::Burn(sender.clone(),asset_id,return_balance,swallower_hash));
			

			Ok(())
		}

		/// 修改吞噬者名称,如果吞噬者不存在,则返回吞噬者不存在.
		/// 修改名称需要支付一定的费用.费用设置在runtime内.
		#[transactional]
		pub fn change_name(sender:T::AccountId,name:Vec<u8>,hash:T::Hash,asset_id:AssetIdOf<T>,fee:AssetBalanceOf<T>)->Result<(),DispatchError>{
			let manager = Manager::<T>::get();
			// 转账给系统管理员，并且增加系统中的总的币的数量。
			T::AssetsTransfer::transfer(asset_id,&sender,&manager,fee,false)?;
			AssetAmount::<T>::try_mutate::<_,DispatchError,_>(|a|{
				*a = a.checked_add(&fee).ok_or(ArithmeticError::Overflow)?;
				Ok(())
			})?;

			Swallowers::<T>::mutate(&hash, |swallower|{
				match swallower{
					Some(s)=>s.name = name.clone(),
					None=>return Err(Error::<T>::SwallowerNotExist),
				}
				return Ok(())
			})?;
			// 增加一个改名事件。
			Self::deposit_event(Event::<T>::ChangeName(sender,name,asset_id,fee,hash));
			Ok(())
		}

		// ACTION #6: function to randomly generate DNA
		fn gen_dna()->[u8;16]{
			let payload = (
				T::GeneRandomness::random(b"dna").0,
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		// 检查吞噬者的名字是否存在。
		pub(crate) fn check_exist_name(name:&Vec<u8>)->bool{
			for swallower in Swallowers::<T>::iter_values(){
				if name == &swallower.name{
					return true;
				}
			}
			return false;
		}

		// 获取系统当前基因价格
		pub(crate) fn gene_price()->Result<AssetBalanceOf<T>,DispatchError>{
			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			let asset_amount = AssetAmount::<T>::get();
			let gene_amount:u64 = GeneAmount::<T>::get();
			let decimal = T::AssetsTransfer::decimals(&asset_id);
			let price_gene ;
			if gene_amount!=0&&asset_amount.ne(&0u32.into()){
				let gene_amount:AssetBalanceOf<T> = GeneAmount::<T>::get()
					.try_into()
					.map_err(|_|ArithmeticError::Overflow)?;
				price_gene = asset_amount.checked_div(&gene_amount).ok_or(ArithmeticError::DivisionByZero)?;
			}else{
				price_gene = (1*10u64.pow(decimal as u32)).try_into().map_err(|_|ArithmeticError::Overflow)?;
			}
			return Ok(price_gene);
		}
	}
}
