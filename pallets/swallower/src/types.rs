use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{inherent::Vec, RuntimeDebug};
use scale_info::TypeInfo;


#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
// #[scale_info(skip_type_params(T))]
pub struct Swallower<AccountId> {
	pub(super) no: u64,
	pub(super) name: Vec<u8>,
	pub(super) init_gene: [u8; 16],
	pub(super) gene: Vec<u8>,
	pub(super) owner:Option<AccountId>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
// 吞噬者在保护区的记录
pub struct ProtectState<BlockNumber>{
	pub(super) start_block:BlockNumber,
	pub(super) end_block:BlockNumber,
}

impl<BlockNumber> ProtectState<BlockNumber>{
	pub fn new(start_block:BlockNumber,end_block:BlockNumber)->Self{
		ProtectState{
			start_block,
			end_block,
		}
	}
} 

// 1. 初始基因位数，默认16位；
// 2. 最长的挑战基因位数，默认 10 位（一般比初始基因位数小，这样新吞噬者之间挑战才有随机性）；
// 3. 销毁手续费比例，默认 3%；
// 4. 初始基因价格，默认 1；
// 5. 挑战费系数，默认 300%；
// 6. 保护费系数 ，默认 10%；
// 7. 奖励触发系数，默认 10%；
// 8. 领取非保护区奖励必须待在非保护区的区块数   1800 (大约1小时)
// 9. 非保护区奖励系数：10%；
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct FeeConfig {
	#[codec(compact)]
	pub(super) change_name_fee: u64,
	pub(super) max_challenge_length: u8,	//最长的挑战基因位数，默认 10 位（一般比初始基因位数小，这样新吞噬者之间挑战才有随机性）
	pub(super) destroy_fee_percent: u32,	//销毁手续费比例，默认 3%；
	pub(super) challenge_fee_ratio:u32,	//挑战费系数，默认 300%；
	pub(super) protect_fee_ratio:u32,	//保护费系数 ，默认 10%；
	pub(super) reward_trigger_ratio:u32,	//奖励触发系数，默认 10%；
	pub(super) battle_zone_reward_block:u32,	//领取非保护区奖励必须待在非保护区的区块数   1800 (大约1小时)
	pub(super) battle_zone_reward_ratio:u32,	//非保护区奖励系数：10%；
	
}

impl Default for FeeConfig {
	fn default() -> Self {
		FeeConfig { 
			change_name_fee: 11, 
			max_challenge_length: 10, 
			destroy_fee_percent: 3,
			challenge_fee_ratio:300,
			protect_fee_ratio:10,
			reward_trigger_ratio:10,
			battle_zone_reward_block:1800,
			battle_zone_reward_ratio:10,
		}
	}
}

// 保护区配置
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ProtectConfig{
	pub(super) first_mint_protect_duration:u32,	//首次生成吞噬者后的保护时间：1600；
}

impl Default for ProtectConfig {
	fn default() -> Self {
		ProtectConfig { 
			first_mint_protect_duration:1600,
		}
	}
}

impl<AccountId> Swallower<AccountId> {
	pub(crate) fn new(name: Vec<u8>, init_gene: [u8; 16], no: u64,owner:AccountId) -> Self {
		Swallower { 
			no, 
			name, 
			init_gene, 
			gene: init_gene.to_vec(), 
			owner:Some(owner),
		}
	}
}


// 转账信息,内部辅助对象。
pub(super) struct TransInfo<'a, AssetIdOf,AccountId,AssetBalanceOf>{
	pub(super) asset_id:AssetIdOf,
	pub(super) sender:&'a AccountId,
	pub(super) manager:&'a AccountId,
	pub(super) challenge_fee:AssetBalanceOf,
}

impl<'a, AssetIdOf,AccountId,AssetBalanceOf> TransInfo<'a, AssetIdOf,AccountId,AssetBalanceOf>{
	pub fn new(asset_id:AssetIdOf,sender:&'a AccountId,manager:&'a AccountId,challenge_fee:AssetBalanceOf)->Self{
		TransInfo{
			asset_id,
			sender,
			manager,
			challenge_fee,
		}
	}
}