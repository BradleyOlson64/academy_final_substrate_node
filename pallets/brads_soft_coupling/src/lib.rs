#![cfg_attr(not(feature = "std"), no_std)]

/// Interface usable to expose identity pallet functionality without requiring the identity pallet itself as a dependency.
pub trait IdentityInterface<Origin, AccountId, DispatchResult> {
    fn try_add_as_social_graph_originator(origin: Origin) -> DispatchResult;

    fn vouch_for(origin: Origin, other: AccountId) -> DispatchResult;

    fn get_voter_from_set(account_id: AccountId) -> Option<()>;
}

/// Interface usable to expose kitties pallet functionality without requiring the kitties pallet itself as a dependency
pub trait KittiesInterface<Origin, AccountId, Balance, BoundedVec, DispatchResult> {
    fn buy_kitty(origin: Origin, kitty_id: [u8; 16], bid_price: Balance) -> DispatchResult;

    fn set_price(origin: Origin, kitty_id: [u8; 16], new_price: Option<Balance>) -> DispatchResult;

    fn transfer(origin: Origin, to: AccountId, kitty_id: [u8; 16]) -> DispatchResult;

    fn create_kitty(origin: Origin) -> DispatchResult;

    fn free_create_kitty(origin: Origin, recipient: AccountId) -> DispatchResult;

    fn get_kitties_owned(account_id: AccountId) -> BoundedVec;
}