use crate as crypto_kitties;
use frame_support::traits::{ConstU16, ConstU64, ConstU128};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use pallet_balances;
use pallet_randomness_collective_flip;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		SubstrateKitties: crypto_kitties,
		Balances: pallet_balances,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl crypto_kitties::Config for Test {
	type Event = Event;
	type Currency = Balances;
    type KittyRandomness = RandomnessCollectiveFlip;
    type MaxKittiesOwned = frame_support::pallet_prelude::ConstU32<1>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = u128;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_randomness_collective_flip::Config for Test {}

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build() -> sp_io::TestExternalities {
	 let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	 pallet_balances::GenesisConfig::<Test> {
	  balances: vec![
	   (1, 1000000000000),
	   (2, 1000000000000),
	   (3, 1000000000000),
	   (4, 1000000000000),
	   (5, 1000000000000),
	   (6, 1000000000000)
	  ],
	 }
	  .assimilate_storage(&mut t)
	  .unwrap();
   
	 let mut ext = sp_io::TestExternalities::new(t);
	 ext.execute_with(|| System::set_block_number(2));
	 ext
	}
   }

   pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		let head = System::finalize();
		System::set_block_number(System::block_number() + 1);
		System::initialize(&System::block_number(), &head.parent_hash, &head.digest);
	}
   }
