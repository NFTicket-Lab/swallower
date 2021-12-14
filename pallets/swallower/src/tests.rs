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
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::Mint(account_id,name.to_vec(),asset_id,160000000000,swallower_no)));
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
		System::assert_last_event(mock::Event::Swallower(Event::<TestRuntime>::Mint(account_id,b"bitilong".to_vec(),asset_id,160000000000,swallower_no)));
		let gene_amount = Swallower::gene_amount();
		assert_eq!(gene_amount,32,"The system gene amount is not correct!");
		let asset_amount = Swallower::asset_amount();
		assert_eq!(asset_amount,320000000000,"The system token amount is not correct!");
		assert_eq!(Swallower::gene_amount(),32,"the system gene amount is error!");
	});
}
