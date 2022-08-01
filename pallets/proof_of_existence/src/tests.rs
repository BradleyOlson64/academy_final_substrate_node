use crate::{mock::*, Error};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use codec::{Encode, Decode};
use frame_support::{assert_noop, assert_ok};
use frame_support::{pallet_prelude::*, traits::ReservableCurrency, traits::Currency};
use frame_system::pallet_prelude::*;

#[test]
fn it_works_for_default_value() {
	ExtBuilder::build().execute_with(|| {
		// Dispatch a signed extrinsic.

		let origin = Origin::signed(1);
		let sender = ensure_signed(origin.clone()).unwrap();
		let proof = Encode::encode(&calculate_hash(&1u64));
		
		assert_ok!(POEModule::create_claim(origin, proof.clone()));
		run_to_block(10);
		// Read pallet storage and assert an expected result.
		assert_eq!(POEModule::get_proofs(proof.clone()).unwrap().0, 1);
		assert_eq!(sender, 1);
	});
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
	let mut s = DefaultHasher::new();
	t.hash(&mut s);
	s.finish()
}

#[test]
fn correct_error_for_none_value() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		let origin2 = Origin::signed(2);
		let proof = Encode::encode(&calculate_hash(&1u64));
		// Ensure the expected error is thrown when no value is present.
		assert_ok!(POEModule::create_claim(origin, proof.clone()));
		assert_noop!(POEModule::create_claim(origin2, proof.clone()), Error::<Test>::ProofAlreadyClaimed);
	});
}

#[test]
fn kitty_side_effect() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		// Ensure the expected error is thrown when no value is present.
		assert_ok!(POEModule::create_kitty(origin.clone()));
		run_to_block(10);
		assert_noop!(POEModule::create_kitty(origin.clone()), crypto_kitties::Error::<Test>::TooManyOwned);
	});
}
