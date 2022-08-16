#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::DispatchResult;
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
		/// The pallet needs a minimum threshold to add a member to the social graph. Including it in the Config makes this threshold configurable
		#[pallet::constant]
		type MinVouches: Get<u32>;
	}

	// The struct on which we build all of our pallet logic
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//Pallet storage items
	#[pallet::storage]
	#[pallet::getter(fn get_voter_from_set)]
	/// The set of all social graph member account ids which can vote via the quadratic voting pallet and vouch for other ids to add to the graph
	pub(super) type VoterSet<T: Config> = CountedStorageMap<_, Blake2_128, T::AccountId, (), OptionQuery>;

	#[pallet::storage]
	/// The set of account ids which have one or more vouches from different ids, but which have not yet reached minVouches to be added to 
	/// the social graph as full members.
	pub(super) type VouchedForSet<T: Config> = StorageMap<_, Blake2_128, T::AccountId, VouchersFor<T>>;

	// Non storage items
	// The set of accountIds which have vouched for a particular accountId. Capped at MinVouches, since upon reaching MinVouches the vouched for
	// account id is added to the voter set and the VouchersFor structure is disposed of.
	pub(super) type VouchersFor<T> = BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MinVouches>;

	// Should introduce new structure to keep track of who has already voted on the current proposal
	
	// Pallets events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Original member of social graph added
		OriginatorAdded(T::AccountId),
		/// A voter has vouched for a non voter [voter, nonVoter]
		VoterVouchedForNonVoter(T::AccountId, T::AccountId),
	}

	// Pallet Errors
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

	// Callable extrinsic functions for this pallet
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
			// Check failure conditions, that the accountId to be added is already in the voter set or the voter set has >= MinVouches members
			ensure!(!VoterSet::<T>::contains_key(sender.clone()), Error::<T>::AlreadyInSet);
			ensure!(VoterSet::<T>::count() < T::MinVouches::get(), Error::<T>::NoNeedForAdditionalOriginators);
			// Add to voter set
			VoterSet::<T>::insert(sender.clone(), ());
			// Send success event
			Self::deposit_event(Event::OriginatorAdded(sender));
			Ok(())
		}

		fn vouch_for_impl(sender: T::AccountId, other: T::AccountId) -> Result<(), DispatchError> {
			// Check that voucher is in the social graph, and therefore has rights to vouch
			ensure!(VoterSet::<T>::contains_key(sender.clone()), Error::<T>::VoucherNotInVoterSet);
			// It doesn't seem like an error to vouch for someone already in the voter set, but it is grounds for an early return
			if VoterSet::<T>::contains_key(other.clone()) { return Ok(()); }
			// If other is in the VouchedForSet, then we want to try to run logic to add a vouch to their VouchersFor vec, or graduate them 
			// to the voter set
			if let Some(vouchers) = VouchedForSet::<T>::get(other.clone()) {
				// One account id can't vouch for another more than once. This would harm social graph security
				ensure!(!vouchers.contains(&sender), Error::<T>::VouchedForSameTwice);
				// If adding a voucher would graduate the address `other` to a full social graph member, then add other to the voter set and remove
				// from VouchedForSet
				if vouchers.len() + 1 >= T::MinVouches::get() as usize {
					VoterSet::<T>::insert(other.clone(), ());
					VouchedForSet::<T>::remove(other.clone());
				}
				// Else add to VouchersFor `other`
				else {
					VouchedForSet::<T>::try_append(&other, sender.clone())
						.expect("Already checked that there is room"); 
				}
				// Else create VouchersFor vec for `other`
			} else {
				VouchedForSet::<T>::try_append(&other, sender.clone())
						.expect("Already checked that there is room");
			};
			// Send success event
			Self::deposit_event(Event::VoterVouchedForNonVoter(sender, other));
			Ok(())
		}
	}
}

// Implementation which allows IdentityInterfaces to use full identity_pallet functionality
impl<T: Config> brads_soft_coupling::IdentityInterface<T::Origin, T::AccountId, DispatchResult> for Pallet<T> {
	fn try_add_as_social_graph_originator(origin: T::Origin) -> DispatchResult {
		Pallet::<T>::try_add_as_social_graph_originator(origin)?;
		Ok(())
	}

    fn vouch_for(origin: T::Origin, other: T::AccountId) -> DispatchResult {
		Pallet::<T>::vouch_for(origin, other)?;
		Ok(())
	}

	fn get_voter_from_set(account_id: T::AccountId) -> Option<()> {
		Pallet::<T>::get_voter_from_set(account_id)
	}
}
