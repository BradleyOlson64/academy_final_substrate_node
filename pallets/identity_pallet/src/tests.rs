use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn can_seed_original_voters() {
	ExtBuilder::build().execute_with(|| {
		
	});
}

#[test]
fn can_not_seed_if_enough_voters() {
	ExtBuilder::build().execute_with(|| {
		
	});
}

#[test]
fn voter_can_vouch_for_non_voter() {
	ExtBuilder::build().execute_with(|| {
		
	});
}

#[test]
fn non_voter_can_not_vouch() {
	ExtBuilder::build().execute_with(|| {
		
	});
}
