use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_kities_works() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
		assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true); // 创建的kitty是存在的
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id)); // kitty的owner
		assert_eq!(KittiesModule::kitty_parents(kitty_id), None); // kitty的parent不存在

		crate::NextKittyId::<Test>::set(crate::KittyId::max_value());
		assert_noop!(
			KittiesModule::create(RuntimeOrigin::signed(account_id)),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn breed_kitties_works() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id),
			Error::<Test>::SameKittyId
		); // 两个kittyid不能相同
		assert_noop!(
			KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id + 1),
			Error::<Test>::InvalidKittyId
		);

		// 创建两个kitty
    assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));
    assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));

    assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2);

    // 开始breed
    assert_ok!(KittiesModule::breed(RuntimeOrigin::signed(account_id), kitty_id, kitty_id+1));

    // 链上状态检查
    let breed_kitty_id = 2;
    
    assert_eq!(KittiesModule::next_kitty_id(), breed_kitty_id + 1); // 下一个kitty_id正确
    
    assert_eq!(KittiesModule::kitties(breed_kitty_id).is_some(), true); // 能找到对应kitty_id的kitty
    
    assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id)); // breed产生的kitty的owner是accout_id

    assert_eq!(KittiesModule::kitty_parents(breed_kitty_id), Some((kitty_id, kitty_id + 1))); // 验证parent
	});
}


#[test]
fn transfer_kitties_works() {
  new_test_ext().execute_with(|| {
    let kitty_id = 0;
    let account_id = 1;
    let receiver_id = 2;
    

    assert_eq!(KittiesModule::next_kitty_id(), kitty_id);

    // kitty不存在
    assert_noop!(
      KittiesModule::transfer(RuntimeOrigin::signed(account_id), receiver_id, kitty_id),
      Error::<Test>::InvalidKittyId
    );

    // 开始创建kitty
    assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id)));

    // 发起方不是kitty的owner
    assert_noop!(
      KittiesModule::transfer(RuntimeOrigin::signed(receiver_id), account_id + 2, kitty_id),
      Error::<Test>::NotOwner
    );

    // 开始转移
    assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(account_id), receiver_id, kitty_id));

    // 转移完成，校验链上数据
    // kitty的owner
    assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(receiver_id));
  });
}