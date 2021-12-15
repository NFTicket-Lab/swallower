use crate::{mock::{self, *}, Manager, Error,Event};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};

#[test]
fn test_set_manager() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Swallower::set_manager(Origin::root(),1));
		System::assert_last_event(mock::Event::Swallower(crate::Event::SetManager(1)));
		// Read pallet storage and assert an expected result.
		assert_eq!(Manager::<TestRuntime>::get(), Some(1));
		assert_noop!(Swallower::set_manager(Origin::signed(1),1),BadOrigin);
	});
}

#[test]
fn manager_set_asset_id() {
	new_test_ext().execute_with(|| {
		assert_noop!(Swallower::set_asset_id(Origin::signed(2),1),Error::<TestRuntime>::NotExistManager);
		assert_ok!(Swallower::set_manager(Origin::root(),1));
		assert_noop!(Swallower::set_asset_id(Origin::signed(2),1),Error::<TestRuntime>::NotManager);
		assert_ok!(Swallower::set_asset_id(Origin::signed(1),1));
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::SetAssetId(1)));
	});
}

#[test]
fn test_mint_swallower(){
	new_test_ext().execute_with(||{
		let account_id = 3;
		let asset_id = 1;
		let manager_id = 2;
		let name = b"hole";
		// 检查没有对应的资产设置。
		assert_noop!(Swallower::mint_swallower(Origin::signed(account_id),b"hole".to_vec()),Error::<TestRuntime>::NotExistAssetId);
		// 设置管理账号。
		assert_ok!(Swallower::set_manager(Origin::root(),manager_id));
		// 设置资产
		assert_ok!(Swallower::set_asset_id(Origin::signed(manager_id),asset_id));
		assert_noop!(Swallower::mint_swallower(Origin::signed(account_id),b"hole".to_vec()),Error::<TestRuntime>::NotEnoughMoney);
		// 转账给购买的用户。
		Assets::transfer(Origin::signed(1),asset_id,account_id,170000000000).unwrap();
		assert_eq!(Swallower::swallower_no(),0,"user init swallower is not zero!");
		// Swallower::AssetsTransfer::transfer(1,1,3,100000000000,true);
		assert_ok!(Swallower::mint_swallower(Origin::signed(account_id),name.to_vec()));
		//检查用户的自己是否减少
		let user_balance = Assets::balance(asset_id,account_id);
		assert_eq!(user_balance,10000000000,"user balance is error!");
		let manager_balance = Assets::balance(asset_id,manager_id);
		assert_eq!(manager_balance,160000000000,"manager not receive the asset token!");
		println!("user_balance is:{}",user_balance);

		// TODO 测试数据越界,此处可能需要使用mock.
		// TODO 检查SwallowerNo是否增加.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		assert_eq!(swallower_no,1,"swallower number is wrong!");
		//检查用户是否增发了一个swallower.
		let owner_swallower = Swallower::owner_swallower(account_id);
		println!("the owner_swallower is:{:?}",owner_swallower);
		assert_eq!(owner_swallower.len(),1,"the user should have one swallower!");
		let swallower_hash = owner_swallower[0];
		println!("owner_swallower[0] is:{:?}",swallower_hash);
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,name,"the swallower name is not correct!");
		assert_eq!(swallower.gene.len(),16,"the gene length is not 16");
		//测试生成的swallower_id.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		// 测试增发事件发送成功.
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::Mint(account_id,name.to_vec(),asset_id,160000000000,swallower_hash)));
		let gene_amount = Swallower::gene_amount();
		assert_eq!(gene_amount,16,"The system gene amount is not correct!");
		let asset_amount = Swallower::asset_amount();
		assert_eq!(asset_amount,160000000000,"The system token amount is not correct!");
		assert_eq!(Swallower::gene_amount(),16,"the system gene amount is error!");
		
		// 用户再次增发一个。
		//检查名字是否存在。
		Assets::transfer(Origin::signed(1),asset_id,account_id,160000000000).unwrap();
		assert_noop!(Swallower::mint_swallower(Origin::signed(account_id),name.to_vec()),Error::<TestRuntime>::NameRepeated);
		Swallower::mint_swallower(Origin::signed(account_id),b"bitilong".to_vec()).unwrap();
		//检查用户的自己是否减少
		let user_balance = Assets::balance(asset_id,account_id);
		assert_eq!(user_balance,10000000000,"user balance is error!");
		let manager_balance = Assets::balance(asset_id,manager_id);
		assert_eq!(manager_balance,320000000000,"manager not receive the asset token!");
		println!("user_balance is:{}",user_balance);

		// TODO 测试数据越界,此处可能需要使用mock.
		// TODO 检查SwallowerNo是否增加.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		assert_eq!(swallower_no,2,"swallower number is wrong!");
		//检查用户是否增发了一个swallower.
		let owner_swallower = Swallower::owner_swallower(account_id);
		println!("the owner_swallower is:{:?}",owner_swallower);
		assert_eq!(owner_swallower.len(),2,"the user should have one swallower!");
		let swallower_hash = owner_swallower[1];
		println!("owner_swallower[0] is:{:?}",swallower_hash);
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,b"bitilong","the swallower name is not correct!");
		assert_eq!(swallower.gene.len(),16,"the gene length is not 16");
		//测试生成的swallower_id.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		// 测试增发事件发送成功.
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::Mint(account_id,b"bitilong".to_vec(),asset_id,160000000000,swallower_hash)));
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
		const ACCOUNT_ID:u64 = 3;
		let asset_id = 1;
		let manager_id = 2;
		let name = b"hole";
		const ASSET_OWNER:u64 = 1;
		// 检查没有对应的资产设置。
		assert_noop!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID),b"hole".to_vec()),Error::<TestRuntime>::NotExistAssetId);
		// 设置管理账号。
		assert_ok!(Swallower::set_manager(Origin::root(),manager_id));
		// 设置资产
		assert_ok!(Swallower::set_asset_id(Origin::signed(manager_id),asset_id));
		assert_noop!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID),b"hole".to_vec()),Error::<TestRuntime>::NotEnoughMoney);
		// 转账给购买的用户。
		Assets::transfer(Origin::signed(1),asset_id,ACCOUNT_ID,170000000000).unwrap();
		assert_eq!(Swallower::swallower_no(),0,"user init swallower is not zero!");
		// Swallower::AssetsTransfer::transfer(1,1,3,100000000000,true);
		assert_ok!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID),name.to_vec()));
		//检查用户的自己是否减少
		let user_balance = Assets::balance(asset_id,ACCOUNT_ID);
		assert_eq!(user_balance,10000000000,"user balance is error!");
		let manager_balance = Assets::balance(asset_id,manager_id);
		assert_eq!(manager_balance,160000000000,"manager not receive the asset token!");
		println!("user_balance is:{}",user_balance);

		// TODO 测试数据越界,此处可能需要使用mock.
		// 检查SwallowerNo是否增加.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		assert_eq!(swallower_no,1,"swallower number is wrong!");
		//检查用户是否增发了一个swallower.
		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID);
		println!("the owner_swallower is:{:?}",owner_swallower);
		assert_eq!(owner_swallower.len(),1,"the user should have one swallower!");
		let swallower_hash = owner_swallower[0];
		println!("owner_swallower[0] is:{:?}",swallower_hash);
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,name,"the swallower name is not correct!");
		assert_eq!(swallower.gene.len(),16,"the gene length is not 16");
		//测试生成的swallower_id.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		// 测试增发事件发送成功.
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::Mint(ACCOUNT_ID,name.to_vec(),asset_id,160000000000,swallower_hash)));
		let gene_amount = Swallower::gene_amount();
		assert_eq!(gene_amount,16,"The system gene amount is not correct!");
		let asset_amount = Swallower::asset_amount();
		assert_eq!(asset_amount,160000000000,"The system token amount is not correct!");
		assert_eq!(Swallower::gene_amount(),16,"the system gene amount is error!");
		
		// 用户再次增发一个。
		//检查名字是否存在。
		Assets::transfer(Origin::signed(ASSET_OWNER),asset_id,ACCOUNT_ID,160000000000).unwrap();
		assert_noop!(Swallower::mint_swallower(Origin::signed(ACCOUNT_ID),name.to_vec()),Error::<TestRuntime>::NameRepeated);
		Swallower::mint_swallower(Origin::signed(ACCOUNT_ID),b"bitilong".to_vec()).unwrap();
		//检查用户的自己是否减少
		let user_balance = Assets::balance(asset_id,ACCOUNT_ID);
		assert_eq!(user_balance,10000000000,"user balance is error!");
		let manager_balance = Assets::balance(asset_id,manager_id);
		assert_eq!(manager_balance,320000000000,"manager not receive the asset token!");
		println!("user_balance is:{}",user_balance);

		// TODO 测试数据越界,此处可能需要使用mock.
		// 检查SwallowerNo是否增加.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		assert_eq!(swallower_no,2,"swallower number is wrong!");
		//检查用户是否增发了一个swallower.
		let owner_swallower = Swallower::owner_swallower(ACCOUNT_ID);
		println!("the owner_swallower is:{:?}",owner_swallower);
		assert_eq!(owner_swallower.len(),2,"the user should have one swallower!");
		let swallower_hash = owner_swallower[1];
		println!("owner_swallower[0] is:{:?}",swallower_hash);
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,b"bitilong","the swallower name is not correct!");
		assert_eq!(swallower.gene.len(),16,"the gene length is not 16");
		//测试生成的swallower_id.
		let swallower_no = Swallower::swallower_no();
		println!("swallower_no is:{}",swallower_no);
		// 测试增发事件发送成功.
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::Mint(ACCOUNT_ID,b"bitilong".to_vec(),asset_id,160000000000,swallower_hash)));
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
		// Assets::transfer(Origin::signed(manager_id),asset_id,ACCOUNT_ID,320000000000).unwrap();
		//获取管理员当前的资金
		let manager_balance = Assets::balance(asset_id,manager_id);
		let account_balance = Assets::balance(asset_id, ACCOUNT_ID);


		assert_ok!(Swallower::burn_swallower(Origin::signed(ACCOUNT_ID), swallower_hash));
		

		let manager_balance_after_burn = Assets::balance(asset_id,manager_id);
		let account_balance_after_burn = Assets::balance(asset_id, ACCOUNT_ID);
		assert_eq!(manager_balance_after_burn,manager_balance.checked_sub(return_balance).unwrap(),"the asset balance of manager is not correct after burning swallower");
		assert_eq!(account_balance_after_burn,account_balance.checked_add(return_balance).unwrap(),"The balance of account is not correct!");
		
		let swallower_no:u64 = Swallower::swallower_no();
		assert_eq!(swallower_no,2);
		// 检查用户是否还拥有的swallower。
		let user_has_swallower = Swallower::owner_swallower(ACCOUNT_ID)
			.iter()
			.any(|s|*s==swallower_hash);
		assert!(!user_has_swallower,"user has the hash of swallower which had been burned!");
	});
}


#[test]
fn test_change_name(){
	new_test_ext().execute_with(||{
		let account_id = 3;
		let asset_id = 1;
		let manager_id = 2;
		let name = b"hole";
		let new_name = b"worm hole";
		let asset_owner = 1;
		// 设置管理账号。
		assert_ok!(Swallower::set_manager(Origin::root(),manager_id));
		// 设置资产
		assert_ok!(Swallower::set_asset_id(Origin::signed(manager_id),asset_id));
		// 转账给购买的用户。
		assert_ok!(Assets::transfer(Origin::signed(1),asset_id,account_id,170000000000));
		assert_eq!(Swallower::swallower_no(),0,"user init swallower is not zero!");
		assert_ok!(Swallower::mint_swallower(Origin::signed(account_id),name.to_vec()));
		//检查用户的自己是否减少
		let user_balance = Assets::balance(asset_id,account_id);
		assert_eq!(user_balance,10000000000,"user balance is error!");
		let manager_balance = Assets::balance(asset_id,manager_id);
		assert_eq!(manager_balance,160000000000,"manager not receive the asset token!");
		println!("user_balance is:{}",user_balance);

		// TODO 获取吞噬者名称.
		let owner_swallower = Swallower::owner_swallower(account_id);
		println!("the owner_swallower is:{:?}",owner_swallower);
		let swallower_hash = owner_swallower[0];
		println!("owner_swallower[0] is:{:?}",swallower_hash);
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,name,"the swallower name is not correct!");
		assert_eq!(swallower.gene.len(),16,"the gene length is not 16");

		assert_ok!(Assets::transfer(Origin::signed(asset_owner),asset_id,account_id,100000000000));
		//修改吞噬者名称
		assert_ok!(Swallower::change_swallower_name(Origin::signed(account_id),swallower_hash,new_name.to_vec()));
		let swallower = Swallower::swallowers(swallower_hash).unwrap();
		println!("swallower is:{:?}",swallower);
		assert_eq!(swallower.name,new_name,"the swallower change name is not success!");
		//检查用户的资金是否转到管理员账号
		let manager_balance = Assets::balance(asset_id,manager_id);
		assert_eq!(manager_balance,(16+11)*10000000000,"manager not receive the asset token!");
		//检查系统资金池是否到账。
		let asset_amount = Swallower::asset_amount();
		assert_eq!(asset_amount,(16+11)*10000000000,"The system gene amount is not correct!");
		// 测试增发事件发送成功.
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::ChangeName(account_id,new_name.to_vec(),asset_id,110000000000,swallower_hash)));
	});
}