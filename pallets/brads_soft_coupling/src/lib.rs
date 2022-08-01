#![cfg_attr(not(feature = "std"), no_std)]
pub trait IdentityInterface<Origin, AccountId, DispatchResult> {
    fn try_add_as_social_graph_originator(origin: Origin) -> DispatchResult;

    fn vouch_for(origin: Origin, other: AccountId) -> DispatchResult;
}

pub trait KittiesInterface<Origin, AccountId, Balance, DispatchResult> {
    fn buy_kitty(origin: Origin, kitty_id: [u8; 16], bid_price: Balance) -> DispatchResult;

    fn set_price(origin: Origin, kitty_id: [u8; 16], new_price: Option<Balance>) -> DispatchResult;

    fn transfer(origin: Origin, to: AccountId, kitty_id: [u8; 16]) -> DispatchResult;

    fn create_kitty(origin: Origin) -> DispatchResult;
}