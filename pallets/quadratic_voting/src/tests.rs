use crate::{mock::*, Error};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use codec::{Encode, Decode};
use frame_support::{assert_noop, assert_ok};
use frame_support::{pallet_prelude::*, traits::ReservableCurrency, traits::Currency};
use frame_system::pallet_prelude::*;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
	let mut s = DefaultHasher::new();
	t.hash(&mut s);
	s.finish()
}



#[test]
fn kitty_side_effect() {
	ExtBuilder::build().execute_with(|| {
		let origin = Origin::signed(1);
		// Ensure the expected error is thrown when no value is present.
		assert_ok!(QuadraticVoting::create_kitty(origin.clone()));
		run_to_block(10);
		assert_noop!(QuadraticVoting::create_kitty(origin.clone()), crypto_kitties::Error::<Test>::TooManyOwned);
	});
}
