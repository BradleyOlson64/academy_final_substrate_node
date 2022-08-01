use crate as pallet_template;
use frame_support::traits::{ConstU16, ConstU32, ConstU64, ConstU128};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

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
		TemplateModule: pallet_template::{Pallet, Call, Storage, Event<T>},
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_template::Config for Test {
	type Event = Event;
	type MinVouches = ConstU32<2>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub struct ExtBuilder;
impl ExtBuilder {
	pub fn build() -> sp_io::TestExternalities {
	 let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
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