
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{inherent::Vec, RuntimeDebug};
use scale_info::TypeInfo;


#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
// #[scale_info(skip_type_params(T))]
pub struct Swallower<AccountId> {
	pub(super) no: u64,
	pub(super) name: Vec<u8>,
	pub(super) init_gene: Vec<u8>,
	pub(super) gene: Vec<u8>,
	pub(super) owner:Option<AccountId>,
}

impl<AccountId> Swallower<AccountId> {
	pub(crate) fn new(name: Vec<u8>, init_gene: Vec<u8>, no: u64,owner:AccountId) -> Self {
		Swallower { 
			no, 
			name, 
			init_gene:init_gene.clone(), 
			gene: init_gene, 
			owner:Some(owner),
		}
	}

	pub(crate) fn get_battle_part(&self,start_position:usize,min_length:usize)->Vec<u8>{
		//得到战斗头部
		let gene = &self.gene;
		let gene = gene.as_slice();
		let (head,tail) = gene.split_at(start_position);
		// 把头部拼接到尾部。
		let reverse_head = [tail,head].concat();
		//截取最小长度部分。
		let (head,_) = reverse_head.split_at(min_length);
		head.to_vec()
	}

	pub(crate) fn battle(&self,facer:&Self,start_position:usize,min_length:usize)->Vec<Winner>{
		let challenger_battle_part = self.get_battle_part(start_position, min_length);
		let facer_battle_part = facer.get_battle_part(start_position, min_length);
		//比如：
		// A抽取的基因  4,230,37,56 
		// B抽取的基因，23, 54,162, 32
		// 第一轮 4 vs 23 ，因为 256 - 23 + 4 = 237 ，23-4 = 19 ，237 > 19 ，所以  B 胜出，B 获得 A 的基因 4，然后将基因4添加到自己的基因链的后边，最后基因就变成了（ 23, 54, 162, 32 , 4），A的基因就变成了 (230, 37,56)
		// 然后第二轮  230 vs 54 ，256 - 230 +54 =80，230-54=176 ，176 > 80 ，所以 B 胜出，B 获得 A 的基因，230，然后基因 230 添加到自己的基因链的后边，最后B基因就编程了（23, 54, 162, 32 , 4,230) ，相应的B的基因变成 ( 37,56)
		// 以此类推
		// 看他们差值的绝对值是不是比128大。
		// 如果比128小，则大的值的基因获胜。
		// 如果比128大，则小的基因值获胜。
		let winners = challenger_battle_part.iter()
			.zip(facer_battle_part.iter())
			.map(|(c,f)|{   			//c 挑战者基因数，f应战者基因数。
				let c:i32 = *c as i32;
				let f:i32 = *f as i32;
				let winner = if (c-f).abs() < 128{
					if c > f{
						Winner::Challenger(c,f)
					}else{
						Winner::Facer(c,f)
					}
				}else{
					if c > f{
						Winner::Facer(c,f)
					}else{
						Winner::Challenger(c,f)
					}
				};
				winner
			}).collect::<Vec<Winner>>();
		return winners;
	}
}

#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Winner {
	Challenger(i32,i32),
	Facer(i32,i32),
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

#[cfg(test)]
mod test{
	use super::*;
	#[test]
	fn test_battle(){
		let challenger = Swallower::new(b"challenger".to_vec(),vec!(4,230,37,56),1,1);
		let facer = Swallower::new(b"face".to_vec(),vec!(23, 54,162,32),2,2);
		let winner = challenger.battle(&facer, 2, 4);
		println!("winner is:{:?}",winner);
	}

}