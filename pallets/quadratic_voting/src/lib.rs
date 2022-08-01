#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency, traits::Currency};
	use frame_system::{pallet_prelude::*};
	use sp_std::vec::Vec; // Step 3.1 will include this in `Cargo.toml`
	use crypto_kitties;
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type CurrencyAmount<T> = <<T as Config>::Token as Currency<AccountIdOf<T>>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + crypto_kitties::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Token: ReservableCurrency<Self::AccountId>; //Loose coupling. This is some notion of token that satisfies a trait
		#[pallet::constant]
		type MinReserveAmount: Get<<Self::Token as Currency<Self::AccountId>>::Balance>;
	}
	// Pallets use events to inform users when important changes are made.
	// Event documentation should end with an array that provides descriptive names for parameters.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when voting power is reserved by locking currency [who, amount]
		VotingPowerReserved(T::AccountId),
		/// Event emitted when voting power is released by unlocking currency [who, amount]
		VotingPowerReleased(T::AccountId),
	}
	#[pallet::error]
	pub enum Error<T> {

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_proofs)]
	#[pallet::unbounded]
	pub(super) type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, CurrencyAmount<T> , OptionQuery>;
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
	// Dispatchable functions allow users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult{
			crypto_kitties::Pallet::<T>::create_kitty(origin.clone())?;
			Ok(())
		}
	}
}