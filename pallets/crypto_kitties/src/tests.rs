use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::pallet_prelude::*;

#[test]
fn minting_kitty() {
	ExtBuilder::build().execute_with(|| {
		// Dispatch a signed extrinsic.

		let origin = Origin::signed(1);
		let sender = ensure_signed(origin.clone()).unwrap();
		
		assert_ok!(SubstrateKitties::create_kitty(origin.clone()));
		run_to_block(10);
		// Read pallet storage and assert an expected result.
		assert_eq!(SubstrateKitties::get_kitties_owned(sender).len(), 1);
		assert_eq!(sender, 1);
	});
}

#[test]
fn breach_max_kitties() {
	ExtBuilder::build().execute_with(|| {
		// Dispatch a signed extrinsic.

		let origin = Origin::signed(1);
		let sender = ensure_signed(origin.clone()).unwrap();
		
		assert_ok!(SubstrateKitties::create_kitty(origin.clone()));
		run_to_block(10);
		// Read pallet storage and assert an expected result.
		assert_noop!(SubstrateKitties::create_kitty(origin.clone()), Error::<Test>::TooManyOwned);
		assert_eq!(SubstrateKitties::get_kitties_owned(sender).len(), 1);
		assert_eq!(sender, 1);
	});
}
