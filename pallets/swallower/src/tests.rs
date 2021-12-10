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
		// 检查没有对应的资产设置。
		assert_noop!(Swallower::mint_swallower(Origin::signed(account_id),b"hole".to_vec()),Error::<TestRuntime>::NotExistAssetId);
		// 设置管理账号。
		Swallower::set_manager(Origin::root(),1).unwrap();
		// 设置资产
		Swallower::set_asset_id(Origin::signed(1),asset_id).unwrap();
		assert_noop!(Swallower::mint_swallower(Origin::signed(account_id),b"hole".to_vec()),Error::<TestRuntime>::NotEnoughMoney);
		// 转账给购买的用户。
		// assert_ok!(Swallower::mint_swallower(Origin::signed(account_id),b"hole".to_vec()));
	});
}
