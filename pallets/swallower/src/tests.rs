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
