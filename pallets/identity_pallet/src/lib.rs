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
	use frame_support::{pallet_prelude::*, Blake2_128, BoundedVec};
	use frame_system::pallet_prelude::*;

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
	pub(super) type VouchedForSet<T: Config> = StorageMap<_, Blake2_128, T::AccountId, VouchersFor<T>>;

	/// Non storage items
	pub(super) type VouchersFor<T> = BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MinVouches>;

	// Pallets events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Original member of social graph added
		OriginatorAdded(T::AccountId),
		/// A voter has vouched for a non voter [voter, nonVoter]
		VoterVouchedForNonVoter(T::AccountId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Proposed originator is already in the voter set
		AlreadyInSet,
		/// Tried to add social graph originators when there are enough
		NoNeedForAdditionalOriginators,
		/// Voucher isn't in the voter set
		VoucherNotInVoterSet,
		/// Voter can only vouch for each other once
		VouchedForSameTwice,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Any social graph needs to start somewhere. Add this caller to the set if set size below minVouches.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn try_add_as_social_graph_originator(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::add_originator_impl(sender)?;
			Ok(())
		}

		/// Vouch for another public key. If not enough to add them to voter set, then creates or 
		/// increments entry in vouched for set.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn vouch_for(origin: OriginFor<T>, other: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::vouch_for_impl(sender, other)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn add_originator_impl(sender: T::AccountId) -> Result<(), DispatchError> {
			ensure!(!VoterSet::<T>::contains_key(sender.clone()), Error::<T>::AlreadyInSet);
			ensure!(VoterSet::<T>::count() < T::MinVouches::get(), Error::<T>::NoNeedForAdditionalOriginators);
			VoterSet::<T>::insert(sender.clone(), ());
			Self::deposit_event(Event::OriginatorAdded(sender));
			Ok(())
		}

		fn vouch_for_impl(sender: T::AccountId, other: T::AccountId) -> Result<(), DispatchError> {
			ensure!(VoterSet::<T>::contains_key(sender.clone()), Error::<T>::VoucherNotInVoterSet);
			if VoterSet::<T>::contains_key(other.clone()) { return Ok(()); }
			if let Some(vouchers) = VouchedForSet::<T>::get(other.clone()) {
				ensure!(!vouchers.contains(&sender), Error::<T>::VouchedForSameTwice);
				if vouchers.len() + 1 >= T::MinVouches::get() as usize {
					VoterSet::<T>::insert(other.clone(), ());
					VouchedForSet::<T>::remove(other.clone());
				}
				else {
					VouchedForSet::<T>::try_append(&other, sender.clone())
						.expect("Already checked that there is room"); 
				}
			} else {
				VouchedForSet::<T>::try_append(&other, sender.clone())
						.expect("Already checked that there is room");
			};
			Self::deposit_event(Event::VoterVouchedForNonVoter(sender, other));
			Ok(())
		}
	}
}
