// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for the benchmark macro for instantiable modules

#![cfg(test)]

use frame_support::{derive_impl, traits::ConstU32};
use sp_runtime::{
	testing::H256,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

#[frame_support::pallet]
mod pallet_test {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	pub trait OtherConfig {
		type OtherEvent;
	}

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + OtherConfig {
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type LowerBound: Get<u32>;
		type UpperBound: Get<u32>;
	}

	#[pallet::storage]
	pub(crate) type Value<T: Config<I>, I: 'static = ()> = StorageValue<_, u32, OptionQuery>;

	#[pallet::event]
	pub enum Event<T: Config<I>, I: 'static = ()> {}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I>
	where
		<T as OtherConfig>::OtherEvent: Into<<T as Config<I>>::RuntimeEvent>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn set_value(origin: OriginFor<T>, n: u32) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			assert!(n >= T::LowerBound::get());
			Value::<T, I>::put(n);
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({0})]
		pub fn dummy(origin: OriginFor<T>, _n: u32) -> DispatchResult {
			let _sender = ensure_none(origin)?;
			Ok(())
		}
	}
}

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		TestPallet: pallet_test,
		TestPallet2: pallet_test::<Instance2>,
	}
);

crate::define_benchmarks!(
	[pallet_test, TestPallet]
	[pallet_test, TestPallet2]
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ();
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_test::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type LowerBound = ConstU32<1>;
	type UpperBound = ConstU32<100>;
}

impl pallet_test::Config<pallet_test::Instance2> for Test {
	type RuntimeEvent = RuntimeEvent;
	type LowerBound = ConstU32<50>;
	type UpperBound = ConstU32<100>;
}

impl pallet_test::OtherConfig for Test {
	type OtherEvent = RuntimeEvent;
}

fn new_test_ext() -> sp_io::TestExternalities {
	RuntimeGenesisConfig::default().build_storage().unwrap().into()
}

mod benchmarks {
	use super::pallet_test::{self, Value};
	use crate::account;
	use frame_support::ensure;
	use frame_system::RawOrigin;
	use sp_core::Get;

	// Additional used internally by the benchmark macro.
	use super::pallet_test::{Call, Config, Pallet};

	crate::benchmarks_instance_pallet! {
		where_clause {
			where
				<T as pallet_test::OtherConfig>::OtherEvent: Clone
					+ Into<<T as pallet_test::Config<I>>::RuntimeEvent>,
				<T as pallet_test::Config<I>>::RuntimeEvent: Clone,
		}

		set_value {
			let b in ( <T as Config<I>>::LowerBound::get() ) .. ( <T as Config<I>>::UpperBound::get() );
			let caller = account::<T::AccountId>("caller", 0, 0);
		}: _ (RawOrigin::Signed(caller), b.into())
		verify {
			assert_eq!(Value::<T, I>::get(), Some(b));
		}

		other_name {
			let b in 1 .. 1000;
		}: dummy (RawOrigin::None, b.into())

		sort_vector {
			let x in 1 .. 10000;
			let mut m = Vec::<u32>::new();
			for i in (0..x).rev() {
				m.push(i);
			}
		}: {
			m.sort();
		} verify {
			ensure!(m[0] == 0, "You forgot to sort!")
		}

		impl_benchmark_test_suite!(
			Pallet,
			crate::tests_instance::new_test_ext(),
			crate::tests_instance::Test
		)
	}
}

#[test]
fn ensure_correct_instance_is_selected() {
	use crate::utils::Benchmarking;

	let whitelist = vec![];

	let mut batches = Vec::<crate::BenchmarkBatch>::new();
	let config = crate::BenchmarkConfig {
		pallet: "pallet_test".bytes().collect::<Vec<_>>(),
		// We only want that this `instance` is used.
		// Otherwise the wrong components are used.
		instance: "TestPallet".bytes().collect::<Vec<_>>(),
		benchmark: "set_value".bytes().collect::<Vec<_>>(),
		selected_components: TestPallet::benchmarks(false)
			.into_iter()
			.find_map(|b| {
				if b.name == "set_value".as_bytes() {
					Some(b.components.into_iter().map(|c| (c.0, c.1)).collect::<Vec<_>>())
				} else {
					None
				}
			})
			.unwrap(),
		verify: false,
		internal_repeats: 1,
	};
	let params = (&config, &whitelist);

	let state = sc_client_db::BenchmarkingState::<sp_runtime::traits::BlakeTwo256>::new(
		Default::default(),
		None,
		false,
		false,
	)
	.unwrap();

	let mut overlay = Default::default();
	let mut ext = sp_state_machine::Ext::new(&mut overlay, &state, None);
	sp_externalities::set_and_run_with_externalities(&mut ext, || {
		add_benchmarks!(params, batches);
		Ok::<_, crate::BenchmarkError>(())
	})
	.unwrap();
}
