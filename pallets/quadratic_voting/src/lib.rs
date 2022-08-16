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
	use frame_support::traits::OriginTrait;
	use sp_std::vec::Vec;
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
		/// Configurable constant for max number of proposals in queue to be voted on
		#[pallet::constant]
		type MaxProposals: Get<u32>;
		/// Configurable constant for max length of any single proposal in bytes
		#[pallet::constant]
		type MaxProposalLength: Get<u32>;
		/// Configurable constant for number of blocks that elapse between finalization of each current proposals at front of proposal queue
		#[pallet::constant]
		type BlocksPerVote: Get<u32>;
		/// Configurable constant for voting power threshold. If not enough voting power votes on the current proposal, it fails regardless
		/// of yae/nay ratio
		#[pallet::constant]
		type ParticipationThreshold: Get<u128>;
		/// Soft coupled interface for identity-pallet
		type Identity: IdentityInterface<Self::Origin, Self::AccountId, DispatchResult>;
		/// Soft coupled interface for crypto-kitties pallet
		type Kitties: KittiesInterface<Self::Origin, Self::AccountId, <Self::Token as Currency<Self::AccountId>>::Balance, BoundedVec<[u8;16], ConstU32<1>>,  DispatchResult>;
	}
	// Pallets use events to inform users when important changes are made.
	// Event documentation should end with an array that provides descriptive names for parameters.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Proposal submitted [submitter, proposal]
		ProposalSubmitted(T::AccountId, BoundedVec<u8, T::MaxProposalLength>),
		/// Event emitted when voting power is reserved by locking currency [who, amount]
		VotingPowerReserved(T::AccountId, CurrencyAmount<T>),
		/// Event emitted when voting power is released by unlocking currency [who]
		AllVotingPowerReleased(T::AccountId),
		/// Voted on current proposal [user, (ayeVotes, nayVotes)]
		VotedOnCurrentProposal(T::AccountId, (u128, u128)),
		/// Vote on current proposal finalized [proposal, (ayeVotes, nayVotes)]
		VoteOnCurrentProposalPassed(BoundedVec<u8, T::MaxProposalLength>, (u128, u128)),
		/// Vote on current proposal failed [proposal, (ayeVotes, nayVotes)]
		CurrentProposalRejected(BoundedVec<u8, T::MaxProposalLength>, (u128, u128)),
	}
	// All the errors that can prevent successful execution of this pallet's calls
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
		/// Tried to vote but there are no proposals
		NoProposalToVoteFor,
		/// Failed converting balance type to vote number
		BalanceToVoteConvertFailed,
	}

	// The struct on which all this pallet's logic is implemented
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_proposals)]
	/// A structure containing a bounded vec of proposal strings as byet vectors in order earliest to latest
	pub(super) type Proposals<T: Config> = StorageValue<_, BoundedVec<BoundedVec<u8, T::MaxProposalLength>, T::MaxProposals>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_proposers)]
	/// A storage value containing the bounded vec of all proposers in order earliest to latest
	pub(super) type Proposers<T: Config> = StorageValue<_, BoundedVec<T::AccountId, T::MaxProposalLength>, ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn get_reserve)]
	/// The map containing associated reserves for each account id. Determines voting power.
	pub(super) type ReserveSet<T: Config> = CountedStorageMap<_, Blake2_128, T::AccountId, CurrencyAmount<T>, ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn get_tally)]
	/// The vote tally for the current proposal
	pub(super) type Tally<T: Config> = StorageValue<_, (u128, u128), ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Every `BlocksPerVote` blocks a vote is finalized. The finalization process is triggered by on_initialize
		fn on_initialize(b: BlockNumberFor<T>) -> u64{
			// Every few blocks finalize the vote on another proposal
			if (b % T::BlocksPerVote::get().into()) == BlockNumberFor::<T>::from(0u32) {
				Self::finalize_current_vote();
			}
			10_000
		}
	}

	// Calls for this pallet
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Adds a new proposal to the end of the queue.
		/// Events: ProposalSubmitted
		/// Errors: TriedToAddEmptyProposal, TooManyProposals
		#[pallet::weight(1_000)]
		pub fn add_proposal(origin: OriginFor<T>, proposal_string: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::add_proposal_impl(sender, proposal_string)?;
			Ok(())
		}

		/// Reserves voting power for a particular account id by reserving tokens
		/// Events: VotingPowerReserved
		/// Errors: InvalidReserveAmount
		#[pallet::weight(1_000)]
		pub fn reserve_voting_power(origin: OriginFor<T>, amount: CurrencyAmount<T>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::reserve_voting_power_impl(sender, amount)?;
			Ok(())
		}

		/// Releases all voting power for a particular account, also unreserving their tokens
		/// Events: AllVotingPowerReleased
		/// Errors: NoVotingPowerToRelease
		#[pallet::weight(1_000)]
		pub fn release_all_voting_power(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::release_all_voting_power_impl(sender)?;
			Ok(())
		}

		/// Votes aye or nay on current proposal, adding the square root of your reserve to either voting side
		/// Events: VotedOnCurrentProposal
		/// Errors: VotedWithNoVotingPower, NotInVoterSet, NoProposalToVoteFor
		#[pallet::weight(1_000)]
		pub fn vote_on_current_proposal(origin: OriginFor<T>, verdict: bool) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			Self::vote_on_current_proposal_impl(sender, verdict)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn add_proposal_impl(sender: T::AccountId, proposal_string: Vec<u8>) -> Result<(), DispatchError> {
			// Check that proposal is non-empty and that proposer is in voter set
			ensure!(proposal_string.len() != 0, Error::<T>::TriedToAddEmptyProposal);
			ensure!(T::Identity::get_voter_from_set(sender.clone()) == Some(()), Error::<T>::NotInVoterSet);
			// Add to proposals and proposers BoundedVecs
			let proposal_as_bounded: BoundedVec<u8, T::MaxProposalLength> = BoundedVec::truncate_from(proposal_string);
			Proposals::<T>::try_append(proposal_as_bounded.clone())
				.map_err(|()| Error::<T>::TooManyProposals)?;
			Proposers::<T>::try_append(sender.clone())
				.map_err(|()| Error::<T>::TooManyProposals)?;
			// Send success event
			Self::deposit_event(Event::ProposalSubmitted(sender, proposal_as_bounded));
			Ok(())
		}

		fn reserve_voting_power_impl(sender: T::AccountId, amount: CurrencyAmount<T>) -> Result<(), DispatchError> {
			// Check for valid reserve amount. 0 is not valid
			let zero_as_balance: CurrencyAmount<T> = CurrencyAmount::<T>::from(0u32); // Must be a better way to do this, but I don't have time
			ensure!(amount > zero_as_balance, Error::<T>::InvalidReserveAmount); 
			// Reserve tokens and add to or create new reserve set entry
			T::Token::reserve(&sender, amount)?;
			if ReserveSet::<T>::contains_key(sender.clone()) {
				let current = ReserveSet::<T>::get(sender.clone());
				ReserveSet::<T>::insert(sender.clone(), amount + current);
			}
			else {
				ReserveSet::<T>::insert(sender.clone(), amount);
			}
			// Send success event
			Self::deposit_event(Event::VotingPowerReserved(sender.clone(), amount));
			Ok(())
		}

		fn release_all_voting_power_impl(sender: T::AccountId) -> Result<(), DispatchError> {
			// Make sure there is voting power to release
			ensure!(ReserveSet::<T>::contains_key(sender.clone()), Error::<T>::NoVotingPowerToRelease);
			// Remove sender from ReserveSet and unreserve their tokens
			let amount = ReserveSet::<T>::get(sender.clone());
			T::Token::unreserve(&sender, amount);
			ReserveSet::<T>::remove(sender.clone());
			// Send success event
			Self::deposit_event(Event::AllVotingPowerReleased(sender.clone()));
			Ok(())
		}

		fn vote_on_current_proposal_impl(sender: T::AccountId, verdict: bool) -> Result<(), DispatchError> {
			// Check failure conditions
			ensure!(ReserveSet::<T>::contains_key(sender.clone()), Error::<T>::VotedWithNoVotingPower);
			if let None = T::Identity::get_voter_from_set(sender.clone()) {
				return Result::Err(frame_support::dispatch::DispatchError::from(Error::<T>::NotInVoterSet));
			}
			ensure!(Proposals::<T>::get().len() > 0, Error::<T>::NoProposalToVoteFor);
			// Calculate voter power from reserve and add that power to the tally of whichever side they supported
			let voter_reserve = ReserveSet::<T>::get(sender.clone());
			let voter_power: u128 = Self::calc_voter_power_from_reserve(voter_reserve)?;
			let mut current_tally = Tally::<T>::get();
			if verdict == true {
				current_tally.0 += voter_power;
			} else {
				current_tally.1 += voter_power;
			}
			Tally::<T>::put(current_tally);
			// Send success event
			Self::deposit_event(Event::VotedOnCurrentProposal(sender.clone(), current_tally));
			Ok(())
		}

		// Calculate voter power as square root of reserve. Fails if reserve is somehow negative
		fn calc_voter_power_from_reserve(reserve: CurrencyAmount<T>) -> Result<u128, Error::<T>> {
			reserve.integer_sqrt().try_into().map_err(|_err| Error::<T>::BalanceToVoteConvertFailed)
		}

		fn finalize_current_vote() {
			// Getting current proposal if any and removing from storage
			let mut proposers = Proposers::<T>::get();
			let mut proposals = Proposals::<T>::get();
			if proposals.len() == 0 || proposers.len() == 0 { 
				return; 
			}
			let proposal_bounded = proposals.get(0).expect("Already checked length").clone();
			let proposer = proposers.get(0).expect("Same as proposals length").clone(); // Grab account id from proposal queue
			proposals.remove(0);
			proposers.remove(0);
			Proposals::<T>::set(proposals);
			Proposers::<T>::set(proposers);

			// Getting current tally and resetting it
			let tally = Tally::<T>::take();
			let twice_nay = match tally.1.checked_mul(2) {
				Some(x) => x,
				None => u128::MAX // Sensable default
			};
			let yae_and_nay = match tally.0.checked_add(tally.1) {
				Some(x) => x,
				None => 0
			};
			// If > 2/3 approval and over participation threshold then pass
			if tally.0 > twice_nay && yae_and_nay > T::ParticipationThreshold::get() {
				// Use proposal to trigger proposal action if any
				if proposal_bounded.to_vec() == b"mint a kitty".to_vec() {
					let origin = OriginFor::<T>::root();
					T::Kitties::free_create_kitty(origin, proposer).unwrap(); // Honestly don't know what to do with this return value. No time to solve
				}
				// Send success event
				Self::deposit_event(Event::VoteOnCurrentProposalPassed(proposal_bounded, tally));
			} else {
				// Send failure event
				Self::deposit_event(Event::CurrentProposalRejected(proposal_bounded, tally));
			}
			

			
		}
	}
}