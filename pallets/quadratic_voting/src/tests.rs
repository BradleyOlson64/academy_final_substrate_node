use crate::{mock::*, Error};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use codec::{Encode, Decode};
use frame_support::{assert_noop, assert_ok};
use frame_support::{pallet_prelude::*, traits::ReservableCurrency, traits::Currency};
use frame_system::pallet_prelude::*;

#[test]
fn can_add_proposal() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		assert_ok!(QuadraticVoting::add_proposal(origin, b"mint a kitty".to_vec()));
		assert_eq!(QuadraticVoting::get_proposals()[0], b"mint a kitty".to_vec());
		assert_eq!(QuadraticVoting::get_proposers()[0], 1);
	});
}

#[test]
fn proposal_errors() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		assert_noop!(QuadraticVoting::add_proposal(origin.clone(), b"".to_vec()), Error::<Test>::TriedToAddEmptyProposal);
		assert_noop!(QuadraticVoting::add_proposal(origin.clone(), b"ghq".to_vec()), Error::<Test>::NotInVoterSet);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		assert_ok!(QuadraticVoting::add_proposal(origin.clone(), b"mint a kitty".to_vec()));
		assert_noop!(QuadraticVoting::add_proposal(origin.clone(), b"ghq".to_vec()), Error::<Test>::TooManyProposals);
	});
}

#[test]
fn can_add_voting_power() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		assert_ok!(QuadraticVoting::reserve_voting_power(origin.clone(), 10_000_000u128));
		assert_eq!(QuadraticVoting::get_reserve(1), 10_000_000u128);
	});
}

#[test]
fn add_voting_power_errors() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		assert_noop!(QuadraticVoting::reserve_voting_power(origin.clone(), 0u128), Error::<Test>::InvalidReserveAmount);
		//assert_noop!(QuadraticVoting::reserve_voting_power(origin.clone(), 1_000_000_000_000u128), Error::<Test>::InvalidReserveAmount);
	});
}

#[test]
fn can_release_voting_power() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		assert_ok!(QuadraticVoting::reserve_voting_power(origin.clone(), 10_000_000u128));
		assert_eq!(QuadraticVoting::get_reserve(1), 10_000_000u128);
		assert_ok!(QuadraticVoting::release_all_voting_power(origin.clone()));
		assert_eq!(QuadraticVoting::get_reserve(1), 0u128);
	});
}

#[test]
fn release_voting_power_errors() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		assert_noop!(QuadraticVoting::release_all_voting_power(origin.clone()), Error::<Test>::NoVotingPowerToRelease);
	});
}

#[test]
fn can_vote() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		let origin2 = Origin::signed(2);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		assert_ok!(QuadraticVoting::reserve_voting_power(origin.clone(), 1_000_000u128));
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin2.clone()));
		assert_ok!(QuadraticVoting::reserve_voting_power(origin2.clone(), 1_000_000u128));
		assert_ok!(QuadraticVoting::add_proposal(origin.clone(), b"mint a kitty".to_vec()));
		assert_ok!(QuadraticVoting::vote_on_current_proposal(origin.clone(), true));
		assert_ok!(QuadraticVoting::vote_on_current_proposal(origin2.clone(), false));
		assert_eq!(QuadraticVoting::get_tally().0, 1_000);
		assert_eq!(QuadraticVoting::get_tally().1, 1_000);
	});
}

#[test]
fn vote_errors() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		let origin2 = Origin::signed(2);
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin.clone()));
		assert_ok!(QuadraticVoting::reserve_voting_power(origin.clone(), 1_000_000u128));
		assert_ok!(IdentityPallet::try_add_as_social_graph_originator(origin2.clone()));
		assert_ok!(QuadraticVoting::reserve_voting_power(origin2.clone(), 1_000_000u128));
		assert_ok!(QuadraticVoting::add_proposal(origin.clone(), b"mint a kitty".to_vec()));
		
	});
}
