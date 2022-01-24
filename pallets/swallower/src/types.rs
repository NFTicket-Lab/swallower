
use codec::{Decode, Encode, MaxEncodedLen};
use crate::sp_runtime::traits::Zero;
use frame_support::{inherent::Vec, RuntimeDebug};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, ArithmeticError, traits::{CheckedAdd, CheckedSub}};
use frame_support::traits::tokens::fungibles::Transfer;

use crate::{Config, AssetIdOf, AssetBalanceOf, AssetAmount};

const GENE_MIDDLE_VALUE:i32 = 128;
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
// #[scale_info(skip_type_params(T))]
pub struct Swallower<AccountId,Hash> {
	pub(super) no: u64,
	pub(super) name: Vec<u8>,
	pub(super) init_gene: Vec<u8>,
	pub(super) gene: Vec<u8>,
	pub(super) owner:Option<AccountId>,
	//TODO 添加一个hash值.
	pub(super) hash:Option<Hash>,
}

impl<AccountId,Hash> Swallower<AccountId,Hash> {
	pub(crate) fn new(name: Vec<u8>, init_gene: Vec<u8>, no: u64,owner:AccountId) -> Self {
		Swallower {
			no,
			name,
			init_gene:init_gene.clone(),
			gene: init_gene,
			owner:Some(owner),
			hash:None,
		}
	}

	// 得到战斗头部.
	// start_position和min_length 不能超过本身的基因长度.如果超过会panic.
	fn get_battle_part(&self,start_position:usize,min_length:usize)->Vec<u8>{
		//得到战斗头部
		let gene = &self.gene;
		// let gene = gene.as_slice();
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

		//print the battle gene
		#[cfg(test)]
		println!("challenger_battle_part is:{:?}",&challenger_battle_part);
		#[cfg(test)]
		println!("facer_battle_part is:{:?}",&facer_battle_part);
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
			.map(|(c,f)|{   			//c 挑战者基因数值，f应战者基因数值。
				let c:i32 = *c as i32;
				let f:i32 = *f as i32;
				let winner = if (c-f).abs() < GENE_MIDDLE_VALUE{
					if c > f{
						Winner::Challenger(f)
					}else{
						Winner::Facer(c)
					}
				}else if (c-f).abs() > GENE_MIDDLE_VALUE{
					if c > f{
						Winner::Facer(c)
					}else{
						Winner::Challenger(f)
					}
				}else{
					Winner::NoneWin(c,f)
				};
				winner
			}).collect::<Vec<Winner>>();
		return winners;
	}

	//判断是否消亡
	pub fn is_destroy(&self)->bool{
		return self.gene.len() == 0;
	}


	pub fn evolve_gene(&mut self,gene:u8){
		self.gene.push(gene);
	}

	pub fn lost_gene(&mut self,gene:u8){
		self.gene.remove(self.gene.iter().position(|g|*g==gene).unwrap());
	}
}

#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Winner {
	Challenger(i32),
	Facer(i32),
	NoneWin(i32,i32),
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
	pub(super) protect_max_length:u32,	//保护区最多的区块高度.2000
	pub(super) reward_trigger_ratio:u32,	//奖励触发系数，默认 10%；
	pub(super) battle_zone_reward_block:u32,	//领取非保护区奖励必须待在非保护区的区块数   1800 (大约1小时)
	pub(super) battle_zone_reward_ratio:u32,	//非保护区奖励系数：10%；
	pub(super) ratio:u32,
}

impl Default for FeeConfig {
	fn default() -> Self {
		FeeConfig {
			change_name_fee: 11,
			max_challenge_length: 10,
			destroy_fee_percent: 3,
			challenge_fee_ratio:300,
			protect_fee_ratio:10,
			protect_max_length:2000,
			reward_trigger_ratio:10,
			battle_zone_reward_block:1800,
			battle_zone_reward_ratio:10,
			ratio:100,
		}
	}
}

impl FeeConfig{
	/// update the config .
	pub fn update_config(
		&mut self,
		change_name_fee:Option<u64>,
		max_challenge_length:Option<u8>,
		destroy_fee_percent:Option<u32>,
		challenge_fee_ratio:Option<u32>,
		protect_fee_ratio:Option<u32>,
		protect_max_length:Option<u32>,
		reward_trigger_ratio:Option<u32>,
		battle_zone_reward_block:Option<u32>,
		battle_zone_reward_ratio:Option<u32>,
		ratio:Option<u32>,
	)->(Vec<u64>,Vec<Vec<u8>>){
		let mut update_vec = Vec::new();
		let mut index_vec = Vec::new();
		if let Some(change_name_fee) = change_name_fee{
			self.change_name_fee = change_name_fee;
			update_vec.push(change_name_fee);
			index_vec.push(b"change_name_fee".to_vec());
		}
		if let Some(max_challenge_length) = max_challenge_length{
			self.max_challenge_length = max_challenge_length;
			update_vec.push(max_challenge_length as u64);
			index_vec.push(b"max_challenge_length".to_vec());
		}
		if let Some(destroy_fee_percent) = destroy_fee_percent{
			self.destroy_fee_percent = destroy_fee_percent;
			update_vec.push(destroy_fee_percent as u64);
			index_vec.push(b"destroy_fee_percent".to_vec());
		}
		if let Some(challenge_fee_ratio) = challenge_fee_ratio{
			self.challenge_fee_ratio = challenge_fee_ratio;
			update_vec.push(challenge_fee_ratio as u64);
			index_vec.push(b"challenge_fee_ratio".to_vec());
		}
		if let Some(protect_fee_ratio) = protect_fee_ratio{
			self.protect_fee_ratio = protect_fee_ratio;
			update_vec.push(protect_fee_ratio as u64);
			index_vec.push(b"protect_fee_ratio".to_vec());
		}
		if let Some(protect_max_length) = protect_max_length{
			self.protect_max_length = protect_max_length;
			update_vec.push(protect_max_length as u64);
			index_vec.push(b"protect_max_length".to_vec());
		}
		if let Some(reward_trigger_ratio) = reward_trigger_ratio{
			self.reward_trigger_ratio = reward_trigger_ratio;
			update_vec.push(reward_trigger_ratio as u64);
			index_vec.push(b"reward_trigger_ratio".to_vec());
		}
		if let Some(battle_zone_reward_block) = battle_zone_reward_block{
			self.battle_zone_reward_block = battle_zone_reward_block;
			update_vec.push(battle_zone_reward_block as u64);
			index_vec.push(b"battle_zone_reward_block".to_vec());
		}
		if let Some(battle_zone_reward_ratio) = battle_zone_reward_ratio{
			self.battle_zone_reward_ratio = battle_zone_reward_ratio;
			update_vec.push(battle_zone_reward_ratio as u64);
			index_vec.push(b"battle_zone_reward_ratio".to_vec());
		}
		if let Some(ratio) = ratio{
			self.ratio = ratio;
			update_vec.push(ratio as u64);
			index_vec.push(b"ratio".to_vec());
		}
		(update_vec,index_vec)
	}
}

// 保护区配置
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ProtectConfig{
	pub(super) first_mint_protect_duration:u32,	//首次生成吞噬者后的保护时间：1600；
	pub(super) auto_enter_safe_zone_block_number:u32,	//自动进入保护区区块高度：100；
}

impl Default for ProtectConfig {
	fn default() -> Self {
		ProtectConfig {
			first_mint_protect_duration:1600,
			auto_enter_safe_zone_block_number:100,
		}
	}
}

// 转账信息,内部辅助对象。
pub(super) struct TransInfo<'a, T:Config>{
	pub(super) asset_id:AssetIdOf<T>,
	pub(super) sender:&'a T::AccountId,
	pub(super) manager:&'a T::AccountId,
	pub(super) fee:AssetBalanceOf<T>,
}

impl<'a, T:Config> TransInfo<'a, T>{
	pub fn new(asset_id:AssetIdOf<T>,sender:&'a T::AccountId,manager:&'a T::AccountId,fee:AssetBalanceOf<T>)->Self{
		TransInfo{
			asset_id,
			sender,
			manager,
			fee,
		}
	}

	pub fn transfer_to_manager(&self)->Result<(), DispatchError>{
		if !self.fee.is_zero(){
			T::AssetsTransfer::transfer(self.asset_id,self.sender,self.manager,self.fee,true)?;
			AssetAmount::<T>::try_mutate(|a|{
				*a = match a.checked_add(&self.fee){
					Some(p)=>p,
					None=>return Err(ArithmeticError::Overflow),
				};
				return Ok(())
			})?;
		}
		return Ok(());
	}
	pub fn transfer_to_sender(&self)->Result<(), DispatchError>{
		if !self.fee.is_zero(){
			T::AssetsTransfer::transfer(self.asset_id,self.manager,self.sender,self.fee,true)?;
			AssetAmount::<T>::try_mutate(|a|{
				*a = match a.checked_sub(&self.fee){
					Some(p)=>p,
					None=>return Err(ArithmeticError::Overflow),
				};
				return Ok(())
			})?;
		}
		return Ok(());
	}
}

#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo,MaxEncodedLen)]
pub struct BattleZoneReward<BlockNumber,Balance>{
	pub(super) block_number:BlockNumber,
	pub(super) fee:Balance,
}

impl<BlockNumber,Balance> BattleZoneReward<BlockNumber,Balance>{
	pub fn new(block_number:BlockNumber,fee:Balance)->Self{
		BattleZoneReward{
			block_number,
			fee,
		}
	}
}

#[cfg(test)]
mod type_test{
	use super::*;
	#[test]
	fn test_battle(){
		//37,56,4,230
		//162,32,78,23
		let mut challenger:Swallower<u32,&[u8]> = Swallower::new(b"challenger".to_vec(),vec!(4,230,50,56),1,1);
		let mut facer:Swallower<u32,&[u8]> = Swallower::new(b"face".to_vec(),vec!(23, 54,162,32,132),2,2);
		println!("challenger gene is:{:?}",challenger.gene);
		println!("facer gene is:{:?}",facer.gene);
		let winners = challenger.battle(&facer, 2, 4);
		println!("winner is:{:?}",winners);
		//修改自身的基因。赢了就把别人的基因拼接到自己的尾部。输了就把自己的基因删除。
		for winner in winners{
			match winner{
				Winner::Challenger(f)=>{
					challenger.evolve_gene(f as u8);
					facer.lost_gene(f as u8);
				},
				Winner::Facer(c)=>{
					facer.evolve_gene(c as u8);
					challenger.lost_gene(c as u8);
				},
				Winner::NoneWin(c,f)=>{		//平手，两边的基因都损失掉。
					challenger.lost_gene(c as u8);
					facer.lost_gene(f as u8);
				}
			}
		}
		println!("challenger gene is:{:?}",challenger.gene);
		assert_eq!(challenger.gene,[56, 32],"challenger gene is error!");
		println!("facer gene is:{:?}",facer.gene);
		assert_eq!(facer.gene,[23, 54, 162,50,230],"facer gene is error!");
	}
}
