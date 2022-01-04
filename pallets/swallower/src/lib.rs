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

	use frame_support::{
		ensure,
		pallet_prelude::ValueQuery,
		traits::{fungibles::InspectMetadata, Randomness},
		Twox64Concat,
	};

	use crate::{
		types::{
			BattleZoneReward, FeeConfig, ProtectConfig, ProtectState, Swallower, TransInfo, Winner,
		},
		weights::WeightInfo,
	};
	use frame_support::{
		dispatch::DispatchResult,
		inherent::Vec,
		pallet_prelude::*,
		sp_runtime::traits::Hash,
		traits::tokens::{
			fungibles,
			fungibles::{Inspect, Transfer},
		},
		transactional,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use pallet_assets::{self as assets};
	use sp_io::hashing::blake2_128;
	use sp_runtime::{
		traits::{CheckedAdd, CheckedDiv, CheckedMul, Saturating, StaticLookup},
		ArithmeticError, DispatchError,
	};
	pub(crate) type AssetBalanceOf<T> =
		<<T as Config>::AssetsTransfer as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type AssetIdOf<T> = <<T as Config>::AssetsTransfer as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	// pub(crate) type TransInfo<'a,T> = TransInfo<'a ,T>;
	pub(crate) type SwallowerStruct<T> = Swallower<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash>;
	// type EngeSwallower<T> = Swallower<BoundedVec<u8,<T as assets::Config>::StringLimit>>;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	const RATIO: u32 = 100;
	// static mut ASSET_ID_SET:u32 = 0; //记录系统设置的Asset_id.
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
	pub type GeneAmount<T> = StorageValue<_, u64, ValueQuery, GetDefault>;

	// 吞噬者总数
	#[pallet::storage]
	#[pallet::getter(fn swallower_amount)]
	pub type SwallowerAmount<T> = StorageValue<_, u64, ValueQuery, GetDefault>;

	// 吞噬者序号。
	#[pallet::storage]
	#[pallet::getter(fn swallower_no)]
	pub type SwallowerNo<T> = StorageValue<_, u64, ValueQuery>;

	// pallet拥有的代币数量,这里只是记个数量。实际的代币存放在管理员处。由管理员负责转出转入。
	#[pallet::storage]
	#[pallet::getter(fn asset_amount)]
	pub type AssetAmount<T> = StorageValue<_, AssetBalanceOf<T>, ValueQuery, GetDefault>;

	// 设置游戏配置
	#[pallet::storage]
	#[pallet::getter(fn swallower_config)]
	pub type SwallowerConfig<T> = StorageValue<_, FeeConfig, ValueQuery>;

	// 保护区配置
	#[pallet::storage]
	#[pallet::getter(fn protect_zone_config)]
	pub type ProtectZoneConfig<T> = StorageValue<_, ProtectConfig, ValueQuery>;

	// 设置支付币种。
	#[pallet::storage]
	#[pallet::getter(fn asset_id)]
	pub type AssetId<T> = StorageValue<_, AssetIdOf<T>>;

	// 设置管理员账户。
	#[pallet::storage]
	#[pallet::getter(fn admin)]
	pub type Admin<T> = StorageValue<_, <T as frame_system::Config>::AccountId>;

	//设置资金管理员,资金管理账号应为无私钥账户，不可提走资金。
	#[pallet::storage]
	#[pallet::getter(fn manager)]
	pub type Manager<T> = StorageValue<_, <T as frame_system::Config>::AccountId, ValueQuery>;

	//用户拥有的吞噬者hash队列
	#[pallet::storage]
	#[pallet::getter(fn owner_swallower)]
	pub type OwnerSwallower<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::MaxSwallowerOwen>,
		ValueQuery,
	>;

	// hash值对应的swallower对象
	#[pallet::storage]
	#[pallet::getter(fn swallowers)]
	pub type Swallowers<T: Config> = StorageMap<_, Twox64Concat, T::Hash, SwallowerStruct<T>>;

	//保护区,如果该map中存在该吞噬者，则吞噬者处于保护中。
	#[pallet::storage]
	#[pallet::getter(fn safe_zone)]
	pub type SafeZone<T: Config> = StorageMap<_, Twox64Concat, T::Hash, ProtectState<T::BlockNumber>>;

	#[pallet::storage]
	#[pallet::getter(fn battle_zone_reward_map)]
	pub type BattleZoneRewardMap<T: Config> = StorageMap<_, Twox64Concat, T::Hash, BattleZoneReward<T::BlockNumber, AssetBalanceOf<T>>>;

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
		Mint(T::AccountId, Vec<u8>, AssetIdOf<T>, AssetBalanceOf<T>, T::Hash),
		Burn(T::AccountId, AssetIdOf<T>, AssetBalanceOf<T>, T::Hash),
		ChangeName(T::AccountId, Vec<u8>, AssetIdOf<T>, AssetBalanceOf<T>, T::Hash),
		EntreSafeZone(T::Hash, T::BlockNumber, T::BlockNumber),
		ExitZone(T::Hash, T::BlockNumber),		//user exit the safe zone
		BattleResult(bool, Vec<u8>, Vec<u8>, Vec<(u8, u8)>),
		BattleZoneReward(T::Hash, T::BlockNumber, AssetBalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotAdmin,
		NotExistAdmin,
		NotExistAssetId,
		NotEnoughMoney, //用户金额不足
		ExceedMaxSwallowerOwned,
		NameRepeated,
		NotOwner,
		SwallowerNotExist,
		SwallowerInSafeZone,
		SwallowerNotInSafeZone,
		WithSelf,     //不能和自己交易。
		HashNotFound, // hash not keep in struct.
		OverMaxHeight,
		RewardRatioLessThanAmount, //未达到领奖阀值.
		RewardTooClose,            //最近已经领过奖了.
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		// config:Vec<(Option<T::AccountId>,Option<AssetIdOf<T>>)>,
		pub admin: Option<T::AccountId>,
		pub asset_id: Option<u32>,
		// pub asset_id:Option<Box<AssetIdOf<T>>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				admin: None,
				asset_id: None,
				// asset_id:None,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			if let Some(m) = &self.admin {
				Admin::<T>::set(Some(m.clone()));
			}
			if let Some(asset_id) = self.asset_id {
				let asset_id =
					AssetIdOf::<T>::decode(&mut (AsRef::<[u8]>::as_ref(&asset_id.encode())))
						.unwrap();
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
		type InitGeneLimit: Get<u32>;

		type AssetsTransfer: fungibles::Transfer<AccountIdOf<Self>>
			+ InspectMetadata<AccountIdOf<Self>>;

		type GeneRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		// type MyAssetId:frame_support::traits::tokens::misc::AssetId+MaybeSerializeDeserialize;

		#[pallet::constant]
		type MaxSwallowerOwen: Get<u32>;

		type SwallowerWeightInfo: WeightInfo;
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// 设置管理员
		#[pallet::weight(T::SwallowerWeightInfo::set_admin())]
		pub fn set_admin(
			origin: OriginFor<T>,
			admin: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			ensure_root(origin)?;
			let admin = T::Lookup::lookup(admin)?;
			Admin::<T>::set(Some(admin.clone()));
			Self::deposit_event(Event::<T>::SetAdmin(admin));
			Ok(())
		}

		/// 设置币种
		#[transactional]
		#[pallet::weight(T::SwallowerWeightInfo::set_asset_id(2000))]
		pub fn set_asset_id(origin: OriginFor<T>, asset_id: AssetIdOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let admin = Admin::<T>::get().ok_or(Error::<T>::NotExistAdmin)?;
			if sender != admin {
				return Err(Error::<T>::NotAdmin)?
			}
			let asset_id_type =
				AssetIdOf::<T>::decode(&mut (AsRef::<[u8]>::as_ref(&asset_id.encode()))).unwrap();
			AssetId::<T>::set(Some(asset_id_type));
			// ASSET_ID_SET = asset_id;
			Self::deposit_event(Event::<T>::SetAssetId(asset_id_type));
			Ok(())
		}

		/// mint swallower
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn mint_swallower(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// TODO 检查名字是否过长。
			//检查名字是否重复。
			ensure!(!Self::check_exist_name(&name), Error::<T>::NameRepeated);
			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			// let gene_amount:u64 = GeneAmount::<T>::get();
			// //获取系统总的代币数量.
			// let asset_amount = AssetAmount::<T>::get();
			// let decimal = T::AssetsTransfer::decimals(&asset_id);
			let price_gene = Self::gene_price()?;
			let init_gene_len = T::InitGeneLimit::get();
			log::info!("init_gene_len is:{}", init_gene_len);
			let price_swallower = price_gene
				.checked_mul(&init_gene_len.try_into().map_err(|_| ArithmeticError::Overflow)?)
				.ok_or(ArithmeticError::Overflow)?;
			let price_swallower: AssetBalanceOf<T> =
				price_swallower.try_into().map_err(|_| ArithmeticError::Overflow)?;

			//检查用户账户是否有足够的金额。
			let balance_user = T::AssetsTransfer::balance(asset_id, &who);
			if balance_user < price_swallower {
				return Err(Error::<T>::NotEnoughMoney)?
			}
			let manager = Manager::<T>::get();
			let trans_info = TransInfo::new(asset_id, &who, &manager, price_swallower);
			Self::mint(&who, name, &trans_info)?;
			Ok(())
		}

		/// 修改swallower名称
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn change_swallower_name(
			origin: OriginFor<T>,
			hash: T::Hash,
			name: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// 判断用户是否拥有这个swallower。
			let swallowers: BoundedVec<T::Hash, _> = OwnerSwallower::<T>::get(&sender);
			ensure!(swallowers.contains(&hash), Error::<T>::NotOwner);
			ensure!(!Self::check_exist_name(&name), Error::<T>::NameRepeated);
			//得到费用配置。
			let change_name_fee_config = SwallowerConfig::<T>::get().change_name_fee;
			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			let decimal = T::AssetsTransfer::decimals(&asset_id);
			let change_name_fee = change_name_fee_config.saturating_mul(10u64.pow(decimal as u32));
			let change_name_fee =
				change_name_fee.try_into().map_err(|_| ArithmeticError::Overflow)?;
			// 检查用户资金是否充足
			let balance_user = T::AssetsTransfer::balance(asset_id, &sender);
			if balance_user < change_name_fee {
				return Err(Error::<T>::NotEnoughMoney)?
			}
			Self::change_name(sender, name, hash, asset_id, change_name_fee)?;
			Ok(())
		}


		// 销毁swallower
		// 1. 基因吞噬者的拥有者可以通过主动销毁基因吞噬者，
		// 按照当前当前吞噬者的基因数量和当前基因价格获得代币返还，返还时需要扣除 3% 的手续费；
		// 1. 返还代币数 = 吞噬者基因数 × 基因价格 × 97%；
		#[pallet::weight(10_000+T::DbWeight::get().reads_writes(1,1))]
		pub fn burn_swallower(origin: OriginFor<T>, hash: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			log::info!(target:"swallower","burn sender is:{:?}",&sender);
			// 判断swallower的所有权。
			let swallowers: BoundedVec<T::Hash, _> = OwnerSwallower::<T>::get(&sender);
			ensure!(swallowers.contains(&hash), Error::<T>::NotOwner);
			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			//得到当前基因的价格。
			let price_gene = Self::gene_price()?;
			//得到费用配置。
			let swallower_config = Self::swallower_config();

			// 得到吞噬者基因数。
			let swallower_gene_count =
				Self::swallowers(&hash).ok_or(Error::<T>::SwallowerNotExist)?.gene.len();
			let return_balance = price_gene
				.checked_mul(
					&swallower_gene_count.try_into().map_err(|_| ArithmeticError::Overflow)?,
				)
				.ok_or(ArithmeticError::Overflow)?;
			// 需要扣除3%的费用。
			let return_balance = return_balance
				.saturating_mul((RATIO - swallower_config.destroy_fee_percent).into())
				.checked_div(&RATIO.into())
				.ok_or(ArithmeticError::Overflow)?;
			// 检查用户资金是否充足
			let manager = Self::manager();
			let balance_manager = T::AssetsTransfer::balance(asset_id, &manager);
			if balance_manager < return_balance {
				return Err(Error::<T>::NotEnoughMoney)?
			}
			let trans_info = TransInfo::new(asset_id, &sender, &manager, return_balance);
			Self::burn(hash, &trans_info)?;
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
		pub fn make_battle(
			origin: OriginFor<T>,
			challenger: T::Hash,
			facer: T::Hash,
		) -> DispatchResult {
			//检查两个吞噬者不能是同一个owner。不能自己人打自己人。
			let sender = ensure_signed(origin)?;
			let facer_swallower =
				Swallowers::<T>::get(&facer).ok_or(Error::<T>::SwallowerNotExist)?;
			let challenger_swallower =
				Swallowers::<T>::get(&challenger).ok_or(Error::<T>::SwallowerNotExist)?;
			log::info!("facer_swallower owner is:{:?}", facer_swallower.owner);
			ensure!(sender != facer_swallower.owner.clone().unwrap(), Error::<T>::WithSelf);
			// 检查能否开战。如果挑战者和被挑战者其中一个在安全区都不能开战。
			// 判断挑战者是否在安全区,如果在安全区,但是已经超时了,需要将该吞噬者移除安全区.
			let is_in_safe = Self::check_in_safe_zone(challenger);
			let is_in_safe_facer = Self::check_in_safe_zone(facer);
			// let in_safe_zone =
			// SafeZone::<T>::iter_keys().any(|hash|hash==challenger||hash==facer);
			if is_in_safe {
				return Err(Error::<T>::SwallowerInSafeZone.into())
			}
			if is_in_safe_facer {
				return Err(Error::<T>::SwallowerInSafeZone.into())
			}

			// 计算发起挑战需要支付的费用。
			let price_gene = Self::gene_price()?;
			let fee_config = Self::swallower_config();
			let challenge_fee_ratio: AssetBalanceOf<T> = fee_config
				.challenge_fee_ratio
				.try_into()
				.map_err(|_| ArithmeticError::Overflow)?;
			let challenge_fee = price_gene.saturating_mul(challenge_fee_ratio) / 100u32.into();
			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			let sender_balance = T::AssetsTransfer::balance(asset_id, &sender);
			if sender_balance < challenge_fee {
				return Err(Error::<T>::NotEnoughMoney)?
			}

			// 可以开战，立即开始对打。
			let manager = Self::manager();
			let trans_info = TransInfo::new(asset_id, &sender, &manager, challenge_fee);
			Self::battle(challenger_swallower, facer_swallower, &trans_info)?;
			Ok(())
		}

		// 1. 如果吞噬者不想接受挑战，可以进入保护区；
		// 2. 主动进入保护区需要支付保护费，进入保护区按保护时长（按区块高度）基因数量计算保护费，
		// 保护费进入资金池； 1. 保护费 = 基因价格 × 保护费系数 × 基因数量 × 区块高度
		// 3. 提前从保护区退出，保护费不退；???
		// 4. 如果吞噬者刚战斗结束，会自动进入保护区一段时间（按区块高度），
		// 进入时长与本次战斗获得(或者失去)的基因数量有关系，每个表示 N 个区块；???
		// 5. 刚铸造出来的吞噬者，自动拥有一定时间的保护期；
		#[pallet::weight(10_000)]
		pub fn user_entre_safe_zone(
			origin: OriginFor<T>,
			hash: T::Hash,
			height: T::BlockNumber,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Self::owner_swallower(&sender).contains(&hash), Error::<T>::NotOwner);
			let in_safe_zone = Self::check_in_safe_zone(hash);
			if in_safe_zone {
				return Err(Error::<T>::SwallowerInSafeZone.into())
			}
			let swallower_config = Self::swallower_config();
			ensure!(height < swallower_config.protect_max_length.into(), Error::<T>::OverMaxHeight);
			// 保护费 = 基因价格 × 保护费系数 × 基因数量 × 区块高度
			let gene_price = Self::gene_price()?;
			let swallower = Self::swallowers(&hash).ok_or(Error::<T>::SwallowerNotExist)?;
			let gene_len = swallower.gene.len() as u32;
			let height: u32 = height.try_into().map_err(|_| ArithmeticError::Overflow)?;
			let protect_fee_ratio = swallower_config.protect_fee_ratio;
			let protect_fee = gene_price
				.saturating_mul(gene_len.into())
				.saturating_mul(height.into())
				.saturating_mul(protect_fee_ratio.into()) /
				RATIO.into();
			// 检查用户资金是否足够支付保护费.
			let asset_id = Self::asset_id().ok_or(Error::<T>::NotExistAssetId)?;
			let balance_user = T::AssetsTransfer::balance(asset_id, &sender);
			if balance_user < protect_fee {
				return Err(Error::<T>::NotEnoughMoney)?
			}
			let start_block = frame_system::Pallet::<T>::block_number();
			let end_block = start_block.saturating_add(height.into());
			let manager = Self::manager();
			let trans_info = TransInfo::<T>::new(asset_id, &sender, &manager, protect_fee);
			Self::entre_safe_zone(hash, start_block, end_block, Some(&trans_info))?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn user_exit_safe_zone(origin: OriginFor<T>, hash: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Self::owner_swallower(&sender).contains(&hash), Error::<T>::NotOwner);
			let in_safe_zone = Self::check_in_safe_zone(hash);
			if !in_safe_zone {
				return Err(Error::<T>::SwallowerNotInSafeZone.into())
			}
			Self::exit_safe_zone(hash)?;

			Ok(())
		}

		// 1. 当在非保护区并且存活的吞噬者低于一定数量时，在非保护区的吞噬者可以领取奖励，
		// 但是必须保证在非保护区待一定的时间（比如1000个区块）； 1. 奖励领取的数量 = 初始基因位数 ×
		// 基因价格 × 奖励系数； 2. 奖励由总代币池出；
		// 3. 触发奖励的吞噬者数量与总吞噬者数量有关，计算公式如下：
		// 	1. 触发奖励的非保护区吞噬者数量 =  吞噬者总数 × 触发奖励系数；
		// 	说明:系统中,吞噬者减少,只有burn方法会减少.在burn方法中检测是否触发开启非保护区奖励.
		// 补充修正:比如说现在只有低于阈值1000个，那么这时候再野区的，都可以申请领奖励，领了过后，
		// 你就必须在野区待多久，这段时间，不允许进入保护区。
		#[pallet::weight(10_000)]
		pub fn user_claim_reward_in_battle_zone(
			origin: OriginFor<T>,
			hash: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Self::owner_swallower(&sender).contains(&hash), Error::<T>::NotOwner);

			let in_safe_zone = Self::check_in_safe_zone(hash);
			if in_safe_zone {
				return Err(Error::<T>::SwallowerInSafeZone.into())
			}
			let swallower_amount = Self::swallower_amount();
			let swallower_config = Self::swallower_config();
			let reward_trigger_ratio = swallower_config.reward_trigger_ratio;
			let trigger_reward_ratio =
				swallower_amount * reward_trigger_ratio as u64 / RATIO as u64;
			let block_number = frame_system::Pallet::<T>::block_number();
			let swallower_amount_in_safe_zone =
				SafeZone::<T>::iter_values().filter(|s| s.end_block >= block_number).count() as u64;
			let swallower_amount_in_battle = swallower_amount - swallower_amount_in_safe_zone;

			if swallower_amount_in_battle <= trigger_reward_ratio {
				return Err(Error::<T>::RewardRatioLessThanAmount)?
			}
			//检查用户是否已经领取
			if let Some(battle_zone_reward) = Self::battle_zone_reward_map(&hash) {
				let battle_zone_reward_block = swallower_config.battle_zone_reward_block;
				if battle_zone_reward.block_number + battle_zone_reward_block.into() > block_number
				{
					return Err(Error::<T>::RewardTooClose)?
				}
			}

			// 奖励领取的数量 = 基因个数 × 基因价格 × 奖励系数；
			let swallower = Swallowers::<T>::get(hash).ok_or(Error::<T>::SwallowerNotExist)?;
			let gene_price = Self::gene_price()?;
			let gene_len = AssetBalanceOf::<T>::from(swallower.gene.len() as u32);
			let fee = gene_len * gene_price * swallower_config.battle_zone_reward_ratio.into() /
				100u32.into();
			// transfer from manager
			let asset_id = Self::asset_id().ok_or(Error::<T>::NotExistAssetId)?;
			let manager = Self::manager();
			let trans_info = TransInfo::<T>::new(asset_id, &sender, &manager, fee);
			Self::claim_reward_in_battle_zone(&hash, &trans_info)?;

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		//开战
		// 通过随机数，从两个对战吞噬者中挑选一段基因进行对战；
		// 挑选位置通过随机数确定；
		// 挑选的长度是基因最短的吞噬者的基因长度和指定长度（默认16）中的最小；
		// 比如基因最短的吞噬者的基因长度是 12 则按 12 位来取；
		// 如果基因最短的吞噬者的基因长度是 22，则按 16 位来取；
		// 挑战就是比较相应基因位的上的基因，比较方式如下：
		// 假设一个基因位的基因有 256 个，基因是256个数字，那么将
		// 256个数字按照顺时针，从小到大围成一个圈； 如果某个基因距离另外一个基因的更长，
		// 则该基因胜出； 基因数字较大者的距离 = 256 - 大数 +小数；
		// 基因数字较小者的距离 = 大数 - 下数；
		// 如果距离相等，或者两个基因完全相同，则平手；
		#[transactional]
		fn battle(
			challenger: SwallowerStruct<T>,
			facer: SwallowerStruct<T>,
			trans_info: &TransInfo<T>,
		) -> DispatchResult {
			//收取的战斗费用转账给基金池
			// T::AssetsTransfer::transfer(trans_info.asset_id,trans_info.sender,trans_info.manager,
			// trans_info.fee,true)?;
			trans_info.transfer_to_manager()?;
			// 生成随机战斗数组。
			let random = T::GeneRandomness::random(b"battle").0;
			let random_ref: &[u8] = random.as_ref();
			let random_len = random_ref.len();
			log::info!("random_len is:{}", random_len);
			let challenge_gene_len = challenger.gene.len();
			let facer_gene_len = facer.gene.len();
			// 1.选择开始位置进行
			let max_challenge_length = Self::swallower_config().max_challenge_length;
			let min_length =
				challenge_gene_len.min(facer_gene_len).min(max_challenge_length as usize);
			let start_position = random_ref[0] as usize % min_length;
			log::info!("start_position is :{}，min_length is:{}", start_position, min_length);
			// 获取吞噬者的基因战斗部分。
			#[cfg(test)]
			println!("challenger gene is:{:?}", challenger.gene);
			#[cfg(test)]
			println!("facer gene is:{:?}", facer.gene);
			let winners = challenger.battle(&facer, start_position, min_length);
			Self::handle_battle_result(winners, challenger, facer)?;
			Ok(())
		}

		//胜利结果处理
		// 	3. 战斗结果
		//     1. 如果平手，两个吞噬者该基因位的基因销毁；
		//     2. 平手的基因会变成碎片，挑战者可以吞噬这些碎片。从而获得部分基因。
		//     3. 吞噬碎片需要一定的区块高度。没有达到这个区块高度，别的吞噬者可以发起挑战，
		// 战斗胜利后可以得到这部分碎片和新的战斗生成的碎片。     3. 如果挑战者不吞噬这部分碎片。
		// 别的基因也可以抢这些碎片进行     2. 如果胜利，
		// 失败一方在该基因位上的基因将追加到胜利一方的整条基因链的之后；     3. 如果失败，
		// 失败方该基因位上的基因将被销毁； 4. 如果某个吞噬者的所有
		// 基因都被销毁，则这个吞噬者会死亡（销毁）；
		#[transactional]
		pub(crate) fn handle_battle_result(
			winners: Vec<Winner>,
			mut challenger: SwallowerStruct<T>,
			mut facer: SwallowerStruct<T>,
		) -> DispatchResult {
			let mut challenger_win_genes = Vec::new();
			let mut facer_win_genes = Vec::new();
			let mut none_win_genes = Vec::new();
			for &winner in &winners {
				match winner {
					Winner::Challenger(f) => {
						// 挑战者胜利一局。
						let f = f as u8;
						challenger.evolve_gene(f);
						facer.lost_gene(f);
						challenger_win_genes.push(f);
					},
					Winner::Facer(c) => {
						//迎战者胜利一局。
						let c = c as u8;
						facer.evolve_gene(c);
						challenger.lost_gene(c);
						facer_win_genes.push(c);
					},
					Winner::NoneWin(c, f) => {
						//平手，两边的基因都损失掉。
						let c = c as u8;
						let f = f as u8;
						challenger.lost_gene(c);
						facer.lost_gene(f);
						none_win_genes.push((c, f));
						// 减少系统中的基因总量.
						GeneAmount::<T>::mutate(|g| *g = (*g).saturating_sub(2u64));
					},
				}
			}

			#[cfg(test)]
			println!("changed challenger gene is:{:?}", challenger.gene);
			#[cfg(test)]
			println!("changed facer gene is:{:?}", facer.gene);

			let challenger_hash = challenger.hash.ok_or(Error::<T>::HashNotFound)?;
			//判断吞噬者是否消亡.
			if challenger.is_destroy() {
				let owner = challenger.owner.unwrap();
				let manager = Self::manager();
				let asset_id = Self::asset_id().ok_or(Error::<T>::NotExistAssetId)?;
				let trans_info = TransInfo::new(
					asset_id,
					&owner,
					&manager,
					0u32.try_into().map_err(|_| ArithmeticError::Overflow)?,
				);
				Self::burn(challenger_hash, &trans_info)?;
			} else {
				//write to db.
				Swallowers::<T>::insert(challenger_hash, challenger);
			}
			let facer_hash = facer.hash.ok_or(Error::<T>::HashNotFound)?;
			if facer.is_destroy() {
				// 清理迎战者
				let owner = facer.owner.unwrap();
				let manager = Self::manager();
				let asset_id = Self::asset_id().ok_or(Error::<T>::NotExistAssetId)?;
				let trans_info = TransInfo::new(
					asset_id,
					&owner,
					&manager,
					0u32.try_into().map_err(|_| ArithmeticError::Overflow)?,
				);
				Self::burn(facer_hash, &trans_info)?;
			} else {
				Swallowers::<T>::insert(facer_hash, facer);
			}
			// get battle result
			let challenger_count = challenger_win_genes.iter().count();
			let facer_count = facer_win_genes.iter().count();

			let is_challenge_success;
			if challenger_count > facer_count {
				is_challenge_success = true;
				// 自动进入保护区,无需收费
				let auto_enter_safe_zone_block_number =
					Self::protect_zone_config().auto_enter_safe_zone_block_number;
				let start_block = frame_system::Pallet::<T>::block_number();
				Self::entre_safe_zone(
					facer_hash,
					start_block,
					start_block.saturating_add(auto_enter_safe_zone_block_number.into()),
					None,
				)?;
			} else {
				is_challenge_success = false;
				// 自动进入保护区,无需收费
				let auto_enter_safe_zone_block_number =
					Self::protect_zone_config().auto_enter_safe_zone_block_number;
				let start_block = frame_system::Pallet::<T>::block_number();
				Self::entre_safe_zone(
					challenger_hash,
					start_block,
					start_block.saturating_add(auto_enter_safe_zone_block_number.into()),
					None,
				)?;
			}

			Self::deposit_event(Event::<T>::BattleResult(
				is_challenge_success,
				challenger_win_genes,
				facer_win_genes,
				none_win_genes,
			));
			//挑战结果是挑战者胜利还是迎战者胜利,挑战者赢取了哪些基因,迎战者赢取了哪些基因.
			// 有哪些基因打平手了. TODO gen the battle result event.
			Ok(())
		}

		//检查用户是否在安全区.
		pub(crate) fn check_in_safe_zone(hash: T::Hash) -> bool {
			// 检查map中是否有该hash存在.
			if let Some(protect_state) = SafeZone::<T>::get(&hash) {
				log::info!("protect_state is:{:?}", protect_state);
				if protect_state.end_block >= frame_system::Pallet::<T>::block_number() {
					return true
				} else {
					// 删除该hash
					SafeZone::<T>::remove(hash);
					return false
				}
			} else {
				return false
			}
		}
		/// 增发一个吞噬者
		/// minter 增发的用户
		/// name 吞噬者的名称，首次给名字免费
		/// asset_id 增发吞噬者需要使用的资产id
		/// price 制造一个吞噬者需要的金额。
		/// init_gene_len 吞噬者初始基因的长度。
		/// 1. 支付指定的费用（ = 初始基因数×单基因价格）可以铸造一个基因吞噬者；
		///		2. 吞噬者铸造的时候会有一个初始的基因片段，初始基因片段为 15
		/// 位，铸造者需要按照基因价格支付铸造费（铸造费是系统代币，需要通过主链代币兑换得到）；
		/// 		1. 基因价格 = 系统总收取代币数量 ÷ 系统总基因数量
		///		2. 基因价格初始为  1 ；
		///	3. 铸造者可以指定吞噬者的名称，只要该名称不和现有吞噬者重复即可；
		#[transactional]
		fn mint(
			minter: &T::AccountId,
			name: Vec<u8>,
			trans_info: &TransInfo<T>,
		) -> Result<(), DispatchError> {
			let asset_id = trans_info.asset_id;
			let price = trans_info.fee;
			//从增发者的账户转账给管理员.
			trans_info.transfer_to_manager()?;
			// T::AssetsTransfer::transfer(trans_info.asset_id,trans_info.sender,trans_info.manager,
			// trans_info.fee,true)?;

			// AssetAmount::<T>::try_mutate(|a|{
			// 	*a = match a.checked_add(&price){
			// 		Some(p)=>p,
			// 		None=>return Err(ArithmeticError::Overflow),
			// 	};
			// 	return Ok(())
			// })?;

			let dna = Self::gen_dna(&name);
			// 记录吞噬者序号
			let swallower_no: u64 = Self::swallower_no();
			let swallower_no = swallower_no.saturating_add(1);
			//增加系统中吞噬者的数量.
			SwallowerNo::<T>::set(swallower_no);
			//增发一个吞噬者给购买者.
			let mut swallower =
				SwallowerStruct::<T>::new(name.clone(), dna.to_vec(), swallower_no, minter.clone());

			//吞噬者生成hash值.
			let swallower_hash = T::Hashing::hash_of(&swallower);
			swallower.hash = Some(swallower_hash);
			//记录用户拥有这个吞噬者
			OwnerSwallower::<T>::try_mutate(minter, |swallower_vec| {
				swallower_vec.try_push(swallower_hash)
			})
			.map_err(|_| Error::<T>::ExceedMaxSwallowerOwned)?;
			//记录该hash值对应的吞噬者实体.
			Swallowers::<T>::insert(swallower_hash, swallower.clone());
			SwallowerAmount::<T>::mutate(|amount| *amount = amount.saturating_add(1));
			//发送一个吞噬者增发成功事件
			Self::deposit_event(Event::<T>::Mint(
				minter.clone(),
				name,
				asset_id,
				price,
				swallower_hash,
			));
			//增加系统中吞噬者的基因数量.
			GeneAmount::<T>::mutate(|g| *g = g.saturating_add(dna.len() as u64));
			//增加系统中币的总数量

			let start_block = frame_system::Pallet::<T>::block_number();
			let auto_protect_duration = Self::protect_zone_config().first_mint_protect_duration;
			let end_block = start_block.saturating_add(auto_protect_duration.into());

			// 自动进入保护区
			Self::entre_safe_zone(swallower_hash, start_block, end_block, None)?;

			Ok(())
		}

		// 进入安全区
		#[transactional]
		fn entre_safe_zone(
			swallower_hash: T::Hash,
			start_block: T::BlockNumber,
			end_block: T::BlockNumber,
			trans_info: Option<&TransInfo<T>>,
		) -> DispatchResult {
			if let Some(trans_info) = trans_info {
				trans_info.transfer_to_manager()?;
				// T::AssetsTransfer::transfer(trans_info.asset_id,trans_info.sender,trans_info.
				// manager,trans_info.fee,true)?;
			}
			let protect_state = ProtectState::new(start_block, end_block);
			// let end_safe_map = StorageValueRef::local(b"ocw-swallower::end-safe");
			// let get =end_safe_map.get();
			// end_safe_map.mutate::<BoundedBTreeMap<T::BlockNumber,T::Hash,T::MaxSwallowerOwen>,
			// Error::<T>,_>(|v|{ 	let btree_map = v.unwrap().unwrap();
			// 	// let mut bm=btree_map.;
			// 	(*btree_map).insert(end_block, swallower_hash);
			// 	Ok(btree_map)
			// });
			SafeZone::<T>::insert(swallower_hash, protect_state);
			Self::deposit_event(Event::<T>::EntreSafeZone(swallower_hash, start_block, end_block));
			Ok(())
		}

		// 获取在战斗区域的奖励
		#[transactional]
		fn claim_reward_in_battle_zone(
			swallower_hash: &T::Hash,
			trans_info: &TransInfo<T>,
		) -> DispatchResult {
			trans_info.transfer_to_sender()?;
			let block_number = frame_system::Pallet::<T>::block_number();
			let battle_zone_reward = BattleZoneReward::new(block_number, trans_info.fee);
			BattleZoneRewardMap::<T>::insert(swallower_hash, battle_zone_reward);
			Self::deposit_event(Event::<T>::BattleZoneReward(
				swallower_hash.clone(),
				block_number,
				trans_info.fee,
			));
			Ok(())
		}
		// 退出安全区,进入战斗区域.
		pub(crate) fn exit_safe_zone(swallower_hash: T::Hash) -> DispatchResult {
			SafeZone::<T>::remove(swallower_hash);
			let current_block = frame_system::Pallet::<T>::block_number();
			Self::deposit_event(Event::<T>::ExitZone(swallower_hash,current_block));
			Ok(())
		}

		#[transactional]
		fn burn(swallower_hash: T::Hash, trans_info: &TransInfo<T>) -> Result<(), DispatchError> {
			let sender = trans_info.sender;
			let asset_id = trans_info.asset_id;
			let return_balance = trans_info.fee;
			//从管理员转账给销毁的用户
			trans_info.transfer_to_sender()?;

			//删除用户拥有这个吞噬者
			OwnerSwallower::<T>::try_mutate(sender, |swallower_vec| -> Result<(), Error<T>> {
				let index = swallower_vec
					.iter()
					.position(|s| *s == swallower_hash)
					.ok_or(Error::<T>::HashNotFound)?;
				swallower_vec.remove(index);
				Ok(())
			})?;
			//删除该hash值对应的吞噬者实体.
			let swallower =
				Swallowers::<T>::take(swallower_hash).ok_or(Error::<T>::SwallowerNotExist)?;
			//减少系统中总的基因数量.
			if swallower.gene.len() > 0 {
				GeneAmount::<T>::mutate(|g| *g = (*g).saturating_sub(swallower.gene.len() as u64));
			}

			SwallowerAmount::<T>::mutate(|amount| *amount = amount.saturating_sub(1));
			//退出安全区
			SafeZone::<T>::remove(swallower_hash);
			// TODO 退出战斗区。不在安全区就在战斗区域.所以不用退出.

			//发送一个吞噬者销毁事件
			Self::deposit_event(Event::<T>::Burn(
				sender.clone(),
				asset_id,
				return_balance,
				swallower_hash,
			));

			Ok(())
		}

		/// 修改吞噬者名称,如果吞噬者不存在,则返回吞噬者不存在.
		/// 修改名称需要支付一定的费用.费用设置在runtime内.
		#[transactional]
		pub fn change_name(sender: T::AccountId,name: Vec<u8>,hash: T::Hash,asset_id: AssetIdOf<T>,fee: AssetBalanceOf<T>,) -> Result<(), DispatchError> {
			let manager = Manager::<T>::get();
			// 转账给系统管理员，并且增加系统中的总的币的数量。
			T::AssetsTransfer::transfer(asset_id, &sender, &manager, fee, false)?;
			AssetAmount::<T>::try_mutate::<_, DispatchError, _>(|a| {
				*a = a.checked_add(&fee).ok_or(ArithmeticError::Overflow)?;
				Ok(())
			})?;

			Swallowers::<T>::mutate(&hash, |swallower| {
				match swallower {
					Some(s) => s.name = name.clone(),
					None => return Err(Error::<T>::SwallowerNotExist),
				}
				return Ok(())
			})?;
			// 增加一个改名事件。
			Self::deposit_event(Event::<T>::ChangeName(sender, name, asset_id, fee, hash));
			Ok(())
		}

		// ACTION #6: function to randomly generate DNA
		fn gen_dna(name: &[u8]) -> [u8; 16] {
			let payload = (
				T::GeneRandomness::random(b"dna").0,
				<frame_system::Pallet<T>>::block_number(),
				name,
			);
			payload.using_encoded(blake2_128)
		}

		// 检查吞噬者的名字是否存在。
		pub(crate) fn check_exist_name(name: &Vec<u8>) -> bool {
			for swallower in Swallowers::<T>::iter_values() {
				if name == &swallower.name {
					return true
				}
			}
			return false
		}

		// 获取系统当前基因价格
		pub(crate) fn gene_price() -> Result<AssetBalanceOf<T>, DispatchError> {
			let asset_id = AssetId::<T>::get().ok_or(Error::<T>::NotExistAssetId)?;
			let asset_amount = AssetAmount::<T>::get();
			let gene_amount: u64 = GeneAmount::<T>::get();
			let decimal = T::AssetsTransfer::decimals(&asset_id);
			let price_gene;
			if gene_amount != 0 && asset_amount.ne(&0u32.into()) {
				let gene_amount: AssetBalanceOf<T> =
					GeneAmount::<T>::get().try_into().map_err(|_| ArithmeticError::Overflow)?;
				price_gene = asset_amount
					.checked_div(&gene_amount)
					.ok_or(ArithmeticError::DivisionByZero)?;
			} else {
				price_gene = (1 * 10u64.pow(decimal as u32))
					.try_into()
					.map_err(|_| ArithmeticError::Overflow)?;
			}
			return Ok(price_gene)
		}
	}
}
