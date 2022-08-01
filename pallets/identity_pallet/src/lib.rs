#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, Blake2_128};
	use frame_system::pallet_prelude::*;
	use frame_support::traits::ReservableCurrency;
	use frame_support::traits::Currency;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		#[pallet::constant]
		type MinVouches: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//Pallet storage items
	#[pallet::storage]
	#[pallet::getter(fn get_voter_set)]
	/// Full voter set with weights matching their balances
	pub(super) type VoterSet<T: Config> = CountedStorageMap<_, Blake2_128, T::AccountId, (), OptionQuery>;

	#[pallet::storage]
	pub(super) type VouchedForSet<T: Config> = CountedStorageMap<_, Blake2_128, T::AccountId, u32>;

	// Pallets events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Tried to add social graph originators when there are enough
		NoNeedForAdditionalOriginators,

	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Any social graph needs to start somewhere. Add this caller to the set if set size below minVouches.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn try_add_as_social_graph_originator(origin: OriginFor<T>) -> DispatchResult {
			
			Ok(())
		}

		/// Vouch for another public key. If not enough to add them to voter set, then creates or 
		/// increments entry in vouched for set.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn vouch_for(origin: OriginFor<T>) -> DispatchResult {

			Ok(())
		}
	}
}
