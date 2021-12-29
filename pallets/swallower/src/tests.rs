use crate::{Error, Event, mock::{self, *}, types::ProtectState};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use crate::frame_support::traits::Hooks;

const ACCOUNT_ID_1:u64 = 3;
const ACCOUNT_ID_2:u64 = 4;
const ASSET_ID:u32 = 1;
const ADMIN_ID:u64 = 2;
const NAME:&[u8;4] = b"hole";
const NAME1:&[u8;10] = b"dragon_two";
const NAME2:&[u8;12] = b"dragon_three";
const ACCOUNT_ASSET_OWNER_ID:u64 = 1;
const MANAGER_ID:u64 = 0;
// 初始发布两个swallower.
fn init(){
	let block_number = System::block_number();
	CollectiveFlip::on_initialize(block_number);
	assert_ok!(Swallower::set_admin(Origin::root(),ADMIN_ID));
	assert_ok!(Swallower::set_asset_id(Origin::signed(ADMIN_ID),ASSET_ID));
	Assets::transfer(Origin::signed(ACCOUNT_ASSET_OWNER_ID),ASSET_ID,ACCOUNT_ID_1,170000000000).unwrap();
	assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME.to_vec()));
	Assets::transfer(Origin::signed(ACCOUNT_ASSET_OWNER_ID),ASSET_ID,ACCOUNT_ID_1,160000000000).unwrap();
	assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME1.to_vec()));
	Assets::transfer(Origin::signed(ACCOUNT_ASSET_OWNER_ID),ASSET_ID,ACCOUNT_ID_2,170000000000).unwrap();
	go_block_number(100);
	assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_2),NAME2.to_vec()));
}

fn go_block_number(number:u64){
	let current_block_number = System::block_number();
	for i in current_block_number..current_block_number+number{
		CollectiveFlip::on_initialize(i);
		System::set_block_number(i);
		let h:[u8;32] = hash69(i as u8);
		System::set_parent_hash(h.into());
	}
}

// Create a Hash with 69 for each byte,
// only used to build genesis config.
#[cfg(feature = "std")]
fn hash69<T: AsMut<[u8]> + Default>(i:u8) -> T {
	let mut h = T::default();
	h
		.as_mut()
		.iter_mut()
		.for_each(|byte| *byte = i);
	h
}


#[test]
fn test_set_admin() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Swallower::set_admin(Origin::root(),1));
		System::assert_last_event(mock::Event::Swallower(crate::Event::SetAdmin(1)));
		// Read pallet storage and assert an expected result.
		assert_eq!(Swallower::admin(), Some(1));
		assert_noop!(Swallower::set_admin(Origin::signed(1),1),BadOrigin);
	});
}

#[test]
fn manager_set_asset_id() {
	new_test_ext().execute_with(|| {
		assert_noop!(Swallower::set_asset_id(Origin::signed(2),1),Error::<TestRuntime>::NotExistAdmin);
		assert_ok!(Swallower::set_admin(Origin::root(),1));
		assert_noop!(Swallower::set_asset_id(Origin::signed(2),1),Error::<TestRuntime>::NotAdmin);
		assert_ok!(Swallower::set_asset_id(Origin::signed(1),1));
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::SetAssetId(1)));
	});
}

#[test]
fn test_mint_swallower(){
	new_test_ext().execute_with(||{
		
		// 检查没有对应的资产设置。
		assert_noop!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),b"hole".to_vec()),Error::<TestRuntime>::NotExistAssetId);
		// 设置管理账号。
		assert_ok!(Swallower::set_admin(Origin::root(),ADMIN_ID));
		// 设置资产
		assert_ok!(Swallower::set_asset_id(Origin::signed(ADMIN_ID),ASSET_ID));
		assert_noop!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),b"hole".to_vec()),Error::<TestRuntime>::NotEnoughMoney);
		// 转账给购买的用户。
		Assets::transfer(Origin::signed(1),ASSET_ID,ACCOUNT_ID_1,170000000000).unwrap();
		assert_eq!(Swallower::swallower_no(),0,"user init swallower is not zero!");
		assert_eq!(Swallower::swallower_amount(),0,"System swallower amount is error!");
		// Swallower::AssetsTransfer::transfer(1,1,3,100000000000,true);
		assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME.to_vec()));
		//检查用户的自己是否减少
		let user_balance = Assets::balance(ASSET_ID,ACCOUNT_ID_1);
		assert_eq!(user_balance,10000000000,"user balance is error!");
		let manager_balance = Assets::balance(ASSET_ID,MANAGER_ID);
		assert_eq!(manager_balance,160000000000,"manager not receive the asset token!");
		println!("user_balance is:{}",user_balance);


		// 检查系统总的吞噬者数量是否增加.
		assert_eq!(Swallower::swallower_amount(),1,"System swallower amount is error!");

		// TODO 测试数据越界,此处可能需要使用mock.
		// TODO 检查SwallowerNo是否增加.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		assert_eq!(swallower_no,1,"swallower number is wrong!");
		//检查用户是否增发了一个swallower.
		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID_1);
		println!("the owner_swallower is:{:?}",owner_swallower);
		assert_eq!(owner_swallower.len(),1,"the user should have one swallower!");
		let swallower_hash = owner_swallower[0];
		println!("owner_swallower[0] is:{:?}",swallower_hash);


		// 检查该swallower有没有进入保护区。
		let protect_state= Swallower::safe_zone(swallower_hash).unwrap();
		let block_number = System::block_number();
		println!("protect_state.end_block is:{}",protect_state.end_block);
		assert_eq!(protect_state.end_block,block_number+1600,"the safe zone end block is error!");


		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,NAME,"the swallower name is not correct!");
		assert_eq!(swallower.gene.len(),16,"the gene length is not 16");
		//测试生成的swallower_id.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		// 测试增发事件发送成功.
		System::assert_has_event(mock::Event::Swallower(Event::<TestRuntime>::Mint(ACCOUNT_ID_1,NAME.to_vec(),ASSET_ID,160000000000,swallower_hash)));
		let gene_amount = Swallower::gene_amount();
		assert_eq!(gene_amount,16,"The system gene amount is not correct!");
		let asset_amount = Swallower::asset_amount();
		assert_eq!(asset_amount,160000000000,"The system token amount is not correct!");
		assert_eq!(Swallower::gene_amount(),16,"the system gene amount is error!");
		
		// 用户再次增发一个。
		//检查名字是否存在。
		Assets::transfer(Origin::signed(1),ASSET_ID,ACCOUNT_ID_1,160000000000).unwrap();
		assert_noop!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME.to_vec()),Error::<TestRuntime>::NameRepeated);
		assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME1.to_vec()));
		assert_eq!(Swallower::swallower_amount(),2,"System swallower amount is error!");
		//检查用户的自己是否减少
		let user_balance = Assets::balance(ASSET_ID,ACCOUNT_ID_1);
		assert_eq!(user_balance,10000000000,"user balance is error!");
		let manager_balance = Assets::balance(ASSET_ID,MANAGER_ID);
		assert_eq!(manager_balance,320000000000,"manager not receive the asset token!");
		println!("user_balance is:{}",user_balance);

		// TODO 测试数据越界,此处可能需要使用mock.
		// TODO 检查SwallowerNo是否增加.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		assert_eq!(swallower_no,2,"swallower number is wrong!");
		//检查用户是否增发了一个swallower.
		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID_1);
		println!("the owner_swallower is:{:?}",owner_swallower);
		assert_eq!(owner_swallower.len(),2,"the user should have one swallower!");
		let swallower_hash = owner_swallower[1];
		println!("owner_swallower[0] is:{:?}",swallower_hash);
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,NAME1,"the swallower name is not correct!");
		assert_eq!(swallower.gene.len(),16,"the gene length is not 16");
		//测试生成的swallower_id.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		// 测试增发事件发送成功.
		System::assert_has_event(mock::Event::Swallower(Event::<TestRuntime>::Mint(ACCOUNT_ID_1,NAME1.to_vec(),ASSET_ID,160000000000,swallower_hash)));
		let gene_amount = Swallower::gene_amount();
		assert_eq!(gene_amount,32,"The system gene amount is not correct!");
		let asset_amount = Swallower::asset_amount();
		assert_eq!(asset_amount,320000000000,"The system token amount is not correct!");
		assert_eq!(Swallower::gene_amount(),32,"the system gene amount is error!");
	});
}

#[test]
fn test_burn_swallower(){
	new_test_ext().execute_with(||{
		// 检查没有对应的资产设置。
		assert_noop!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),b"hole".to_vec()),Error::<TestRuntime>::NotExistAssetId);
		// 设置管理账号。
		assert_ok!(Swallower::set_admin(Origin::root(),ADMIN_ID));
		// 设置资产
		assert_ok!(Swallower::set_asset_id(Origin::signed(ADMIN_ID),ASSET_ID));
		assert_noop!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),b"hole".to_vec()),Error::<TestRuntime>::NotEnoughMoney);
		// 转账给购买的用户。
		Assets::transfer(Origin::signed(1),ASSET_ID,ACCOUNT_ID_1,170000000000).unwrap();
		assert_eq!(Swallower::swallower_no(),0,"user init swallower is not zero!");
		// Swallower::AssetsTransfer::transfer(1,1,3,100000000000,true);
		assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME.to_vec()));
		//检查用户的自己是否减少
		let user_balance = Assets::balance(ASSET_ID,ACCOUNT_ID_1);
		assert_eq!(user_balance,10000000000,"user balance is error!");
		let manager_balance = Assets::balance(ASSET_ID,MANAGER_ID);
		assert_eq!(manager_balance,160000000000,"manager not receive the asset token!");
		println!("user_balance is:{}",user_balance);

		// TODO 测试数据越界,此处可能需要使用mock.
		// 检查SwallowerNo是否增加.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		assert_eq!(swallower_no,1,"swallower number is wrong!");
		//检查用户是否增发了一个swallower.
		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID_1);
		println!("the owner_swallower is:{:?}",owner_swallower);
		assert_eq!(owner_swallower.len(),1,"the user should have one swallower!");
		let swallower_hash = owner_swallower[0];
		println!("owner_swallower[0] is:{:?}",swallower_hash);
		// 检查该swallower有没有进入保护区。
		let protect_state= Swallower::safe_zone(swallower_hash).unwrap();
		let block_number = System::block_number();
		println!("protect_state.end_block is:{}",protect_state.end_block);
		assert_eq!(protect_state.end_block,block_number+1600,"the safe zone end block is error!");
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,NAME,"the swallower name is not correct!");
		assert_eq!(swallower.gene.len(),16,"the gene length is not 16");
		//测试生成的swallower_id.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		// 测试增发事件发送成功.
		System::assert_has_event(mock::Event::Swallower(Event::<TestRuntime>::Mint(ACCOUNT_ID_1,NAME.to_vec(),ASSET_ID,160000000000,swallower_hash)));
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::EntreSafeZone(swallower_hash,1,1601)));
		let gene_amount = Swallower::gene_amount();
		assert_eq!(gene_amount,16,"The system gene amount is not correct!");
		let asset_amount = Swallower::asset_amount();
		assert_eq!(asset_amount,160000000000,"The system token amount is not correct!");
		assert_eq!(Swallower::gene_amount(),16,"the system gene amount is error!");
		
		// 用户再次增发一个。
		//检查名字是否存在。
		Assets::transfer(Origin::signed(ACCOUNT_ASSET_OWNER_ID),ASSET_ID,ACCOUNT_ID_1,160000000000).unwrap();
		assert_noop!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME.to_vec()),Error::<TestRuntime>::NameRepeated);
		Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME1.to_vec()).unwrap();
		//检查用户自己的资金是否减少
		let user_balance = Assets::balance(ASSET_ID,ACCOUNT_ID_1);
		assert_eq!(user_balance,10000000000,"user balance is error!");
		let manager_balance = Assets::balance(ASSET_ID,MANAGER_ID);
		assert_eq!(manager_balance,320000000000,"manager not receive the asset token!");
		println!("user_balance is:{}",user_balance);

		// TODO 测试数据越界,此处可能需要使用mock.
		// 检查SwallowerNo是否增加.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		assert_eq!(swallower_no,2,"swallower number is wrong!");
		//检查用户是否增发了一个swallower.
		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID_1);
		println!("the owner_swallower is:{:?}",owner_swallower);
		assert_eq!(owner_swallower.len(),2,"the user should have two swallower!");
		let swallower_hash = owner_swallower[1];
		println!("owner_swallower[0] is:{:?}",swallower_hash);
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,NAME1,"the swallower name is not correct!");
		assert_eq!(swallower.gene.len(),16,"the gene length is not 16");
		//测试生成的swallower_id.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		// 测试增发事件发送成功.
		System::assert_has_event(mock::Event::Swallower(Event::<TestRuntime>::Mint(ACCOUNT_ID_1,NAME1.to_vec(),ASSET_ID,160000000000,swallower_hash)));
		let gene_amount = Swallower::gene_amount();
		assert_eq!(gene_amount,32,"The system gene amount is not correct!");
		let asset_amount = Swallower::asset_amount();
		assert_eq!(asset_amount,320000000000,"The system token amount is not correct!");
		assert_eq!(Swallower::gene_amount(),32,"the system gene amount is error!");
		
		assert_noop!(Swallower::burn_swallower(Origin::signed(100), swallower_hash),Error::<TestRuntime>::NotOwner);

		
		let price_gene = Swallower::gene_price().unwrap();
		println!("gene price is :{}",price_gene);
		let swallower_config = Swallower::swallower_config();
		let swallower_gene_count = Swallower::swallowers(&swallower_hash).unwrap().gene.len();
		let return_balance = price_gene
				.checked_mul(swallower_gene_count.try_into().unwrap())
				.unwrap();
		let return_balance = (return_balance as u64).checked_mul(100u64-swallower_config.destroy_fee_percent as u64).unwrap().checked_div(100).unwrap();
		println!("return_balance is :{}",return_balance);
		
		// 把manager的资金转走。
		// Assets::transfer(Origin::signed(manager_id),asset_id,ACCOUNT_ID_1,320000000000).unwrap();
		//获取管理员当前的资金
		let manager_balance = Assets::balance(ASSET_ID,MANAGER_ID);
		let account_balance = Assets::balance(ASSET_ID, ACCOUNT_ID_1);

		assert_eq!(Swallower::swallower_amount(),2,"System swallower amount is error!");
		assert_ok!(Swallower::burn_swallower(Origin::signed(ACCOUNT_ID_1), swallower_hash));
		assert_eq!(Swallower::swallower_amount(),1,"System swallower amount is error!");
		// 检查该swallower有没有退出安全区。
		let protect_state = Swallower::safe_zone(swallower_hash);
		assert!(protect_state.is_none());

		let manager_balance_after_burn = Assets::balance(ASSET_ID,MANAGER_ID);
		let account_balance_after_burn = Assets::balance(ASSET_ID, ACCOUNT_ID_1);
		assert_eq!(manager_balance_after_burn,manager_balance.checked_sub(return_balance).unwrap(),"the asset balance of manager is not correct after burning swallower");
		assert_eq!(account_balance_after_burn,account_balance.checked_add(return_balance).unwrap(),"The balance of account is not correct!");
		
		let swallower_no:u64 = Swallower::swallower_no();
		assert_eq!(swallower_no,2);
		let owner_swallowers = Swallower::owner_swallower(ACCOUNT_ID_1);
		//检查用户只应该有一个吞噬者.
		assert_eq!(owner_swallowers.len(),1,"User should have one swallower.");
		// 检查用户是否还拥有的swallower。
		let user_has_swallower = owner_swallowers
			.iter()
			.any(|s|*s==swallower_hash);
		assert!(!user_has_swallower,"user has the hash of swallower which had been burned!");
		// 检查删除吞噬者中的实体。
		assert_eq!(Swallower::swallowers(&swallower_hash),None);
		// 检查系统中吞噬者的基因数量
		assert_eq!(Swallower::gene_amount(),16,"the system gene amount is not correct!");
		assert_eq!(Swallower::asset_amount(),(320000000000-160000000000*97/100),"the asset amount of system is not correct!");
		// 检查事件。
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::Burn(ACCOUNT_ID_1,ASSET_ID,160000000000*97/100,swallower_hash)));


	});
}


#[test]
fn test_change_name(){
	new_test_ext().execute_with(||{
		let new_name = b"worm hole";
		let asset_owner = 1;
		const MANAGER_ID:u64 = 0;
		// 设置管理账号。
		assert_ok!(Swallower::set_admin(Origin::root(),ADMIN_ID));
		// 设置资产
		assert_ok!(Swallower::set_asset_id(Origin::signed(ADMIN_ID),ASSET_ID));
		// 转账给购买的用户。
		assert_ok!(Assets::transfer(Origin::signed(1),ASSET_ID,ACCOUNT_ID_1,170000000000));
		assert_eq!(Swallower::swallower_no(),0,"user init swallower is not zero!");
		assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME.to_vec()));
		//检查用户的自己是否减少
		let user_balance = Assets::balance(ASSET_ID,ACCOUNT_ID_1);
		assert_eq!(user_balance,10000000000,"user balance is error!");
		let manager_balance = Assets::balance(ASSET_ID,MANAGER_ID);
		assert_eq!(manager_balance,160000000000,"manager not receive the asset token!");
		println!("user_balance is:{}",user_balance);

		// TODO 获取吞噬者名称.
		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID_1);
		println!("the owner_swallower is:{:?}",owner_swallower);
		let swallower_hash = owner_swallower[0];
		println!("owner_swallower[0] is:{:?}",swallower_hash);
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,NAME,"the swallower name is not correct!");
		assert_eq!(swallower.gene.len(),16,"the gene length is not 16");

		assert_ok!(Assets::transfer(Origin::signed(asset_owner),ASSET_ID,ACCOUNT_ID_1,100000000000));
		//修改吞噬者名称
		assert_ok!(Swallower::change_swallower_name(Origin::signed(ACCOUNT_ID_1),swallower_hash,new_name.to_vec()));
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,new_name,"the swallower change name is not success!");
		//检查用户的资金是否转到管理员账号
		let manager_balance = Assets::balance(ASSET_ID,MANAGER_ID);
		assert_eq!(manager_balance,(16+11)*10000000000,"manager not receive the asset token!");
		//检查系统资金池是否到账。
		let asset_amount = Swallower::asset_amount();
		assert_eq!(asset_amount,(16+11)*10000000000,"The system gene amount is not correct!");
		// 测试增发事件发送成功.
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::ChangeName(ACCOUNT_ID_1,new_name.to_vec(),ASSET_ID,110000000000,swallower_hash)));
	});
}


#[test]
fn test_make_battle(){
	new_test_ext().execute_with(||{
		init();
		//检查用户是否增发了一个swallower.
		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID_1);
		println!("the owner_swallower is:{:?}",owner_swallower);
		assert_eq!(owner_swallower.len(),2,"the user should have one swallower!");
		let challenger_hash = owner_swallower[0];
		// let swallower_dragon_one = Swallower::swallowers(swallower_hash_0).unwrap();
		let same_owner_hash_1 = owner_swallower[1];
		// let swallower_dragon_two = Swallower::swallowers(swallower_hash_1).unwrap();
		let owner_swallower_2 = Swallower::owner_swallower(ACCOUNT_ID_2);
		let facer_hash_2 = owner_swallower_2[0];
		// let swallower_dragon_three = Swallower::swallowers(swallower_hash_2).unwrap();
		// the dragon_one make a battle to dragon two
		assert_noop!(Swallower::make_battle(Origin::signed(ACCOUNT_ID_1), challenger_hash, same_owner_hash_1),Error::<TestRuntime>::WithSelf);
		assert_noop!(Swallower::make_battle(Origin::signed(ACCOUNT_ID_1), challenger_hash, facer_hash_2),Error::<TestRuntime>::SwallowerInSafeZone);
		System::set_block_number(1601);
		assert_noop!(Swallower::make_battle(Origin::signed(ACCOUNT_ID_1), challenger_hash, facer_hash_2),Error::<TestRuntime>::SwallowerInSafeZone);
		System::set_block_number(1702);
		CollectiveFlip::on_initialize(1702);
		
		assert_noop!(Swallower::make_battle(Origin::signed(ACCOUNT_ID_1), challenger_hash, [1;32].into()),Error::<TestRuntime>::SwallowerNotExist);
		assert_noop!(Swallower::make_battle(Origin::signed(ACCOUNT_ID_1), [1;32].into(), facer_hash_2),Error::<TestRuntime>::SwallowerNotExist);

		let result = Swallower::make_battle(Origin::signed(ACCOUNT_ID_1), challenger_hash, facer_hash_2);
		assert_eq!(result,Err(Error::<TestRuntime>::NotEnoughMoney.into()));
		// TODO check the test failed reason.
		// assert_noop!(Swallower::make_battle(Origin::signed(ACCOUNT_ID_1), swallower_hash_0, swallower_hash_2),Error::<TestRuntime>::NotEnoughMoney);
		Assets::transfer(Origin::signed(ACCOUNT_ASSET_OWNER_ID),ASSET_ID,ACCOUNT_ID_1,3000000000000).unwrap();
		let price_gene = Swallower::gene_price().unwrap();
		// let challenge_fee_ratio = Swallower::swallower_config().challenge_fee_ratio;
		let challenge_fee = price_gene.saturating_mul(3u64);
		//check the user balance
		let balance_of_challenger = Assets::balance(ASSET_ID, ACCOUNT_ID_1);
		let balance_of_manager = Assets::balance(ASSET_ID, MANAGER_ID);
		assert_ok!(Swallower::make_battle(Origin::signed(ACCOUNT_ID_1), challenger_hash, facer_hash_2));
		//检查用户的资金是否被扣除.
		let balance_of_challenger_after_battle = Assets::balance(ASSET_ID, ACCOUNT_ID_1);
		assert_eq!(challenge_fee,balance_of_challenger.saturating_sub(balance_of_challenger_after_battle));
		let balance_of_manager_after_battle = Assets::balance(ASSET_ID, MANAGER_ID);
		assert_eq!(balance_of_manager_after_battle,balance_of_manager + challenge_fee);
		// 检查数据库中的数据是否已经修改!
		
		let events = System::events();
		println!("events is:{:?}",events.last());
		// calculate the 
		// check the db data
		let swallower1 = Swallower::swallowers(challenger_hash).unwrap();
		println!("swallower1.gene is:{:?}",swallower1.gene);
		assert_eq!(swallower1.gene,vec!(106, 89, 231, 255, 98, 136, 136, 40, 56, 192, 225, 35, 90, 75));
		let facer2 = Swallower::swallowers(facer_hash_2).unwrap();
		println!("swallower2.gene is:{:?}",facer2.gene);
		
		assert_eq!(facer2.gene,vec!(29, 54, 231, 217, 94, 47, 125, 79, 68, 2, 32, 49, 209, 66, 29, 100, 208, 48));
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::BattleResult(false, vec!(225, 35, 90, 75), vec!(209, 66, 29, 100, 208, 48), vec!())));
		
		// 检查失败者是否进入安全区.
		let safe_zone:ProtectState<u64> = Swallower::safe_zone(challenger_hash).unwrap();
		assert_eq!(safe_zone.start_block, 1702,"start block is fail!");
		assert_eq!(safe_zone.end_block, 1802,"start block is fail!");
		assert_eq!(Swallower::safe_zone(facer_hash_2),None,"facer should not entre the safe zone!");
		// System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::ChangeName(ACCOUNT_ID_1,new_name.to_vec(),ASSET_ID,110000000000,swallower_hash)));
		// Swallower::handle_battle_result(vec!())swallower2;
	});
}

#[test]
fn test_entre_safe_zone(){
	new_test_ext().execute_with(||{
		// 设置管理账号。
		assert_ok!(Swallower::set_admin(Origin::root(),ADMIN_ID));
		// 设置资产
		assert_ok!(Swallower::set_asset_id(Origin::signed(ADMIN_ID),ASSET_ID));
		// 转账给购买的用户。
		assert_ok!(Assets::transfer(Origin::signed(1),ASSET_ID,ACCOUNT_ID_1,170000000000));
		assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME.to_vec()));

		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID_1);
		let swallower_hash = owner_swallower[0];
		assert_noop!(Swallower::user_entre_safe_zone(Origin::signed(ACCOUNT_ID_2), swallower_hash, 1000),Error::<TestRuntime>::NotOwner);
		assert_noop!(Swallower::user_entre_safe_zone(Origin::signed(ACCOUNT_ID_1), swallower_hash, 1000),Error::<TestRuntime>::SwallowerInSafeZone);
		assert_ok!(Swallower::exit_safe_zone(swallower_hash));
		assert_noop!(Swallower::user_entre_safe_zone(Origin::signed(ACCOUNT_ID_1), swallower_hash, 3000),Error::<TestRuntime>::OverMaxHeight);
		assert_noop!(Swallower::user_entre_safe_zone(Origin::signed(ACCOUNT_ID_1), swallower_hash, 1500),Error::<TestRuntime>::NotEnoughMoney);
		assert_ok!(Assets::transfer(Origin::signed(1),ASSET_ID,ACCOUNT_ID_1,25000000000000));
		let balance_of_user_before = Assets::balance(ASSET_ID,ACCOUNT_ID_1);
		let balance_of_manager_before = Assets::balance(ASSET_ID,MANAGER_ID);
		let height:u32 = 1500;
		let start_block = System::block_number();
		let end_block = start_block.saturating_add(height.into());
		
		assert_ok!(Swallower::user_entre_safe_zone(Origin::signed(ACCOUNT_ID_1), swallower_hash, 1500));
		let balance_of_user_after = Assets::balance(ASSET_ID,ACCOUNT_ID_1);
		let balance_of_manager_after = Assets::balance(ASSET_ID,MANAGER_ID);
		assert_eq!(balance_of_user_after,balance_of_user_before - 24000000000000,"the user amount is not correct!");
		assert_eq!(balance_of_manager_after,balance_of_manager_before + 24000000000000,"the manager amount is not correct!");
		assert!(Swallower::check_in_safe_zone(swallower_hash),"swallower not entre the safe zone!");
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::EntreSafeZone(swallower_hash,start_block,end_block)));
	});
}

#[test]
fn test_exit_safe_zone(){
	new_test_ext().execute_with(||{
		// 设置管理账号。
		assert_ok!(Swallower::set_admin(Origin::root(),ADMIN_ID));
		// 设置资产
		assert_ok!(Swallower::set_asset_id(Origin::signed(ADMIN_ID),ASSET_ID));
		// 转账给购买的用户。
		assert_ok!(Assets::transfer(Origin::signed(1),ASSET_ID,ACCOUNT_ID_1,170000000000));
		assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),NAME.to_vec()));

		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID_1);
		let swallower_hash = owner_swallower[0];
		assert_noop!(Swallower::user_exit_safe_zone(Origin::signed(ACCOUNT_ID_2), swallower_hash),Error::<TestRuntime>::NotOwner);
		assert_ok!(Swallower::user_exit_safe_zone(Origin::signed(ACCOUNT_ID_1),swallower_hash));
		assert_noop!(Swallower::user_exit_safe_zone(Origin::signed(ACCOUNT_ID_1), swallower_hash),Error::<TestRuntime>::SwallowerNotInSafeZone);
	});
}

#[test]
fn test_user_claim_reward_in_battle_zone(){
	new_test_ext().execute_with(||{
		// 设置管理账号。
		init();
		//检查用户是否增发了一个swallower.
		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID_1);
		println!("the owner_swallower is:{:?}",owner_swallower);
		assert_eq!(owner_swallower.len(),2,"the user should have one swallower!");
		let challenger_hash = owner_swallower[0];
		// assert_noop!(Swallower::user_claim_reward_in_battle_zone(Origin::signed(ACCOUNT_ID_1),[1u8;32].into()),Error::<TestRuntime>::SwallowerNotExist);
		// let same_owner_hash_1 = owner_swallower[1];
		let owner_swallower_2 = Swallower::owner_swallower(ACCOUNT_ID_2);
		let facer_hash_2 = owner_swallower_2[0];
		assert_noop!(Swallower::user_claim_reward_in_battle_zone(Origin::signed(ACCOUNT_ID_1),facer_hash_2),Error::<TestRuntime>::NotOwner);
		assert_noop!(Swallower::user_claim_reward_in_battle_zone(Origin::signed(ACCOUNT_ID_1),challenger_hash),Error::<TestRuntime>::SwallowerInSafeZone);
		assert_ok!(Swallower::exit_safe_zone(challenger_hash));
		for i in 0..20{
			Assets::transfer(Origin::signed(ACCOUNT_ASSET_OWNER_ID),ASSET_ID,ACCOUNT_ID_1,160000000000).unwrap();
			assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID_1),vec!(i)));
		}
		assert_noop!(Swallower::user_claim_reward_in_battle_zone(Origin::signed(ACCOUNT_ID_1),challenger_hash),Error::<TestRuntime>::RewardRatioLessThanAmount);
		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID_1);
		for i in 2..owner_swallower.len(){
			assert_ok!(Swallower::exit_safe_zone(owner_swallower[i]));
		}
		assert_ok!(Swallower::user_claim_reward_in_battle_zone(Origin::signed(ACCOUNT_ID_1),challenger_hash));
		let battle_zone_reward = Swallower::battle_zone_reward_map(challenger_hash).unwrap();
		let block_number = System::block_number();
		assert_eq!(battle_zone_reward.block_number,block_number);
		assert_eq!(battle_zone_reward.fee,16000000000);
		assert_noop!(Swallower::user_claim_reward_in_battle_zone(Origin::signed(ACCOUNT_ID_1),challenger_hash),Error::<TestRuntime>::RewardTooClose);
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::BattleZoneReward(challenger_hash,block_number,16000000000)));
	});
}