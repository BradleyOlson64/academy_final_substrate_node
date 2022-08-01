use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn can_seed_original_voters() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		let origin2 = Origin::signed(2);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		run_to_block(10);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin2.clone()));
		assert_eq!(IdentityPallet::get_voter_from_set(1), Some(()));
		assert_eq!(IdentityPallet::get_voter_from_set(2), Some(()));
	});
}

#[test]
fn can_not_seed() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		let origin2 = Origin::signed(2);
		let origin3 = Origin::signed(3);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		assert_noop!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()), Error::<Test>::AlreadyInSet);
		run_to_block(10);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin2.clone()));
		assert_noop!(IdentityPallet::try_add_as_social_graph_originator(origin3.clone()), Error::<Test>::NoNeedForAdditionalOriginators);
	});
}

#[test]
fn voter_can_vouch_for_non_voter() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		let origin2 = Origin::signed(2);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin2.clone()));
		assert_ok!(IdentityPallet::vouch_for(origin.clone(), 3));
		assert_ok!(IdentityPallet::vouch_for(origin2.clone(), 3));
		assert_eq!(IdentityPallet::get_voter_from_set(3), Some(()));
	});
}

#[test]
fn vouch_fail_situations() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		let origin2 = Origin::signed(2);
		assert_noop!(IdentityPallet::vouch_for(origin.clone(), 3), Error::<Test>::VoucherNotInVoterSet);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin2.clone()));
		assert_ok!(IdentityPallet::vouch_for(origin2.clone(), 2));
		assert_ok!(IdentityPallet::vouch_for(origin.clone(), 3));
		assert_noop!(IdentityPallet::vouch_for(origin.clone(), 3), Error::<Test>::VouchedForSameTwice);
		assert_eq!(IdentityPallet::get_voter_from_set(1), Some(()));
		assert_eq!(IdentityPallet::get_voter_from_set(2), Some(()));
		assert_eq!(IdentityPallet::get_voter_from_set(3), None);
		assert_ok!(IdentityPallet::vouch_for(origin2.clone(), 3));
		assert_eq!(IdentityPallet::get_voter_from_set(3), Some(()));
	});
}
