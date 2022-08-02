#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
use frame_support::pallet_prelude::DispatchResult;
use frame_support::{pallet_prelude::*, traits::ReservableCurrency, traits::Currency};
	use frame_support::storage::types::StorageValue;
	use frame_system::{pallet_prelude::*};
	use frame_support::sp_runtime::traits::IntegerSquareRoot;
	use brads_soft_coupling::{ KittiesInterface, IdentityInterface};
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type CurrencyAmount<T> = <<T as Config>::Token as Currency<AccountIdOf<T>>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Token: ReservableCurrency<Self::AccountId>; //Loose coupling. This is some notion of token that satisfies a trait
		#[pallet::constant]
		type MaxProposals: Get<u32>;
		#[pallet::constant]
		type MaxProposalLength: Get<u32>;
		#[pallet::constant]
		type BlocksPerVote: Get<u32>;
		#[pallet::constant]
		type MinReserveAmount: Get<<Self::Token as Currency<Self::AccountId>>::Balance>;
		/// Soft coupled custom pallets
		type Identity: IdentityInterface<Self::Origin, Self::AccountId, DispatchResult>;
		type Kitties: KittiesInterface<Self::Origin, Self::AccountId, <Self::Token as Currency<Self::AccountId>>::Balance, BoundedVec<[u8;16], ConstU32<1>>,  DispatchResult>;
	}
	// Pallets use events to inform users when important changes are made.
	// Event documentation should end with an array that provides descriptive names for parameters.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Proposal submitted [submitter, proposal]
		ProposalSubmitted(T::AccountId, String),
		/// Event emitted when voting power is reserved by locking currency [who, amount]
		VotingPowerReserved(T::AccountId, CurrencyAmount<T>),
		/// Event emitted when voting power is released by unlocking currency [who]
		AllVotingPowerReleased(T::AccountId),
		/// Voted on current proposal [user, (ayeVotes, nayVotes)]
		VotedOnCurrentProposal(T::AccountId, (u128, u128)),
	}
	#[pallet::error]
	pub enum Error<T> {
		/// Doesn't make sense to add an empty proposal
		TriedToAddEmptyProposal,
		/// Too many proposals in the queue. Can't add one.
		TooManyProposals,
		/// The reserve amount is either more than the sender can afford, or 0
		InvalidReserveAmount,
		/// The user in question didn't have any voting reserves
		NoVotingPowerToRelease,
		/// Tried to vote with no voting power
		VotedWithNoVotingPower,
		/// Tried to vote while not in voter set
		NotInVoterSet,
		/// Failed converting balance type to vote number
		BalanceToVoteConvertFailed,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_proposals)]
	/// A value containing a bounded vec of proposal strings as byet vectors
	pub(super) type Proposals<T: Config> = StorageValue<_, BoundedVec<BoundedVec<u8, T::MaxProposalLength>, T::MaxProposals>, ValueQuery>;
	
	#[pallet::storage]
	/// The set containing associated reserves for each account id. Determines voting power.
	pub(super) type ReserveSet<T: Config> = CountedStorageMap<_, Blake2_128, T::AccountId, CurrencyAmount<T>, ValueQuery>;
	
	#[pallet::storage]
	/// The vote tally for the current proposal
	pub(super) type Tally<T: Config> = StorageValue<_, (u128, u128), ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(b: BlockNumberFor<T>) -> u64{
			// Every few blocks finalize the vote on another proposal
			if (b % T::BlocksPerVote::get().into()) == BlockNumberFor::<T>::from(0u32) {
				Self::finalize_current_vote();
			}
			0
		}
	}

	// Dispatchable functions allow users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult{
			T::Kitties::create_kitty(origin.clone())?;
			Ok(())
		}

		#[pallet::weight(1)]
		pub fn add_proposal(origin: OriginFor<T>, proposal_string: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::add_proposal_impl(sender, proposal_string)?;
			Ok(())
		}

		#[pallet::weight(1)]
		pub fn reserve_voting_power(origin: OriginFor<T>, amount: CurrencyAmount<T>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::reserve_voting_power_impl(sender, amount)?;
			Ok(())
		}

		#[pallet::weight(1)]
		pub fn release_all_voting_power(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::release_all_voting_power_impl(sender)?;
			Ok(())
		}

		#[pallet::weight(1)]
		pub fn vote_on_current_proposal(origin: OriginFor<T>, verdict: bool) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::vote_on_current_proposal_impl(sender, verdict)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn add_proposal_impl(sender: T::AccountId, proposal_string: Vec<u8>) -> Result<(), DispatchError> {
			ensure!(proposal_string.len() != 0, Error::<T>::TriedToAddEmptyProposal);
			let proposal_as_bounded: BoundedVec<u8, T::MaxProposalLength> = BoundedVec::truncate_from(proposal_string);
			let proposal_escaped = proposal_as_bounded.escape_ascii().to_string();
			Proposals::<T>::try_append(proposal_as_bounded)
				.map_err(|()| Error::<T>::TooManyProposals)?;
			Self::deposit_event(Event::ProposalSubmitted(sender, proposal_escaped));
			Ok(())
		}

		fn reserve_voting_power_impl(sender: T::AccountId, amount: CurrencyAmount<T>) -> Result<(), DispatchError> {
			let zero_as_balance: CurrencyAmount<T> = CurrencyAmount::<T>::from(0u32); // Must be a better way to do this, but I don't have time
			ensure!(amount > zero_as_balance, Error::<T>::InvalidReserveAmount); 
			T::Token::reserve(&sender, amount)?;
			ReserveSet::<T>::insert(sender.clone(), amount);

			Self::deposit_event(Event::VotingPowerReserved(sender.clone(), amount));
			Ok(())
		}

		fn release_all_voting_power_impl(sender: T::AccountId) -> Result<(), DispatchError> {
			ensure!(ReserveSet::<T>::contains_key(sender.clone()), Error::<T>::NoVotingPowerToRelease);
			let amount = ReserveSet::<T>::get(sender.clone());
			T::Token::unreserve(&sender, amount);
			ReserveSet::<T>::remove(sender.clone());

			Self::deposit_event(Event::AllVotingPowerReleased(sender.clone()));
			Ok(())
		}

		fn vote_on_current_proposal_impl(sender: T::AccountId, verdict: bool) -> Result<(), DispatchError> {
			ensure!(ReserveSet::<T>::contains_key(sender.clone()), Error::<T>::VotedWithNoVotingPower);
			ensure!(T::Identity::get_voter_from_set(sender.clone()) == Some(()), Error::<T>::NotInVoterSet);
			let voter_reserve = ReserveSet::<T>::get(sender.clone());
			let voter_power: u128 = Self::calc_voter_power_from_reserve(voter_reserve)?;
			let mut current_tally = Tally::<T>::get();
			if verdict == true {
				current_tally.0 += voter_power;
			} else {
				current_tally.1 += voter_power;
			}
			Tally::<T>::put(current_tally);
			Self::deposit_event(Event::VotedOnCurrentProposal(sender.clone(), current_tally));
			Ok(())
		}

		pub fn get_current_proposal_impl() -> Option<BoundedVec<u8, T::MaxProposalLength>> {
			let proposals = Proposals::<T>::get();
			if let Some(proposal) = proposals.get(0) {
				let result = proposal.clone();
				return Some(result);
			};

			None
		}

		fn calc_voter_power_from_reserve(reserve: CurrencyAmount<T>) -> Result<u128, Error::<T>> {
			reserve.integer_sqrt().try_into().map_err(|_err| Error::<T>::BalanceToVoteConvertFailed)
		}

		fn finalize_current_vote() {

		}
	}
}