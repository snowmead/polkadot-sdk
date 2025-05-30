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

//! General tests for construct_runtime macro, test for:
//! * error declared with decl_error works
//! * integrity test is generated

#![recursion_limit = "128"]

use codec::MaxEncodedLen;
use frame_support::{
	derive_impl, parameter_types, traits::PalletInfo as _, weights::RuntimeDbWeight,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	DispatchEventInfo,
};
use scale_info::TypeInfo;
use sp_core::sr25519;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, ValidateUnsigned, Verify},
	DispatchError, ModuleError,
};
use sp_version::RuntimeVersion;

parameter_types! {
	pub static IntegrityTestExec: u32 = 0;
}

#[frame_support::pallet(dev_mode)]
mod module1 {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		pub fn fail(_origin: OriginFor<T>) -> DispatchResult {
			Err(Error::<T, I>::Something.into())
		}
	}

	#[pallet::origin]
	#[derive(
		Clone,
		PartialEq,
		Eq,
		RuntimeDebug,
		Encode,
		Decode,
		DecodeWithMemTracking,
		MaxEncodedLen,
		TypeInfo,
	)]
	#[scale_info(skip_type_params(I))]
	pub struct Origin<T, I = ()>(pub PhantomData<(T, I)>);

	#[pallet::event]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		A(<T as frame_system::Config>::AccountId),
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		Something,
	}
}

#[frame_support::pallet(dev_mode)]
mod module2 {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			IntegrityTestExec::mutate(|i| *i += 1);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		pub fn fail(_origin: OriginFor<T>) -> DispatchResult {
			Err(Error::<T>::Something.into())
		}
	}

	#[pallet::origin]
	#[derive(
		Clone,
		PartialEq,
		Eq,
		RuntimeDebug,
		Encode,
		Decode,
		DecodeWithMemTracking,
		MaxEncodedLen,
		TypeInfo,
	)]
	pub struct Origin;

	#[pallet::event]
	pub enum Event<T> {
		A,
	}

	#[pallet::error]
	pub enum Error<T> {
		Something,
	}
}

mod nested {
	use super::*;

	#[frame_support::pallet(dev_mode)]
	pub mod module3 {
		use super::*;
		use frame_support::pallet_prelude::*;
		use frame_system::pallet_prelude::*;

		#[pallet::pallet]
		pub struct Pallet<T>(_);

		#[pallet::config]
		pub trait Config: frame_system::Config {
			type RuntimeEvent: From<Event<Self>>
				+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
		}

		#[pallet::hooks]
		impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
			fn integrity_test() {
				IntegrityTestExec::mutate(|i| *i += 1);
			}
		}

		#[pallet::call]
		impl<T: Config> Pallet<T> {
			pub fn fail(_origin: OriginFor<T>) -> DispatchResult {
				Err(Error::<T>::Something.into())
			}
		}

		#[pallet::origin]
		#[derive(
			Clone,
			PartialEq,
			Eq,
			RuntimeDebug,
			Encode,
			Decode,
			DecodeWithMemTracking,
			MaxEncodedLen,
			TypeInfo,
		)]
		pub struct Origin;

		#[pallet::event]
		pub enum Event<T> {
			A,
		}

		#[pallet::error]
		pub enum Error<T> {
			Something,
		}

		#[pallet::genesis_config]
		#[derive(frame_support::DefaultNoBound)]
		pub struct GenesisConfig<T: Config> {
			#[serde(skip)]
			pub _config: core::marker::PhantomData<T>,
		}

		#[pallet::genesis_build]
		impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
			fn build(&self) {}
		}

		#[pallet::validate_unsigned]
		impl<T: Config> ValidateUnsigned for Pallet<T> {
			type Call = Call<T>;
			fn validate_unsigned(
				_source: TransactionSource,
				_call: &Self::Call,
			) -> TransactionValidity {
				Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
			}
		}
	}
}

#[frame_support::pallet(dev_mode)]
pub mod module3 {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		pub fn fail(_origin: OriginFor<T>) -> DispatchResult {
			Err(Error::<T>::Something.into())
		}
		pub fn aux_1(_origin: OriginFor<T>, #[pallet::compact] _data: u32) -> DispatchResult {
			unreachable!()
		}
		pub fn aux_2(
			_origin: OriginFor<T>,
			_data: i32,
			#[pallet::compact] _data2: u32,
		) -> DispatchResult {
			unreachable!()
		}
		#[pallet::weight(0)]
		pub fn aux_3(_origin: OriginFor<T>, _data: i32, _data2: String) -> DispatchResult {
			unreachable!()
		}
		#[pallet::weight(3)]
		pub fn aux_4(_origin: OriginFor<T>) -> DispatchResult {
			unreachable!()
		}
		#[pallet::weight((5, DispatchClass::Operational))]
		pub fn operational(_origin: OriginFor<T>) -> DispatchResult {
			unreachable!()
		}
	}

	#[pallet::origin]
	#[derive(
		Clone,
		PartialEq,
		Eq,
		RuntimeDebug,
		Encode,
		Decode,
		DecodeWithMemTracking,
		MaxEncodedLen,
		TypeInfo,
	)]
	pub struct Origin<T>(pub PhantomData<T>);

	#[pallet::event]
	pub enum Event<T> {
		A,
	}

	#[pallet::error]
	pub enum Error<T> {
		Something,
	}

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		#[serde(skip)]
		pub _config: core::marker::PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {}
	}

	#[pallet::storage]
	pub type Storage<T> = StorageValue<_, u32>;

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;
		fn validate_unsigned(
			_source: TransactionSource,
			_call: &Self::Call,
		) -> TransactionValidity {
			Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
		}
	}
}

pub type BlockNumber = u64;
pub type Signature = sr25519::Signature;
pub type AccountId = <Signature as Verify>::Signer;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<u32, RuntimeCall, Signature, ()>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

#[frame_support::runtime(legacy_ordering)]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask,
		RuntimeViewFunction
	)]
	pub struct Runtime;

	#[runtime::pallet_index(30)]
	pub type System = frame_system + Pallet + Call + Event<T> + Origin<T>;

	#[runtime::pallet_index(31)]
	pub type Module1_1 = module1<Instance1>;

	#[runtime::pallet_index(32)]
	pub type Module2 = module2;

	#[runtime::pallet_index(33)]
	pub type Module1_2 = module1<Instance2>;

	#[runtime::pallet_index(34)]
	pub type NestedModule3 = nested::module3;

	#[runtime::pallet_index(35)]
	#[runtime::disable_unsigned]
	pub type Module3 = self::module3;

	#[runtime::pallet_index(6)]
	#[runtime::disable_call]
	pub type Module1_3 = module1<Instance3>;

	#[runtime::pallet_index(3)]
	pub type Module1_4 = module1<Instance4>;

	#[runtime::pallet_index(4)]
	#[runtime::disable_call]
	pub type Module1_5 = module1<Instance5>;

	#[runtime::pallet_index(1)]
	pub type Module1_6 = module1<Instance6>;

	#[runtime::pallet_index(2)]
	pub type Module1_7 = module1<Instance7>;

	#[runtime::pallet_index(12)]
	pub type Module1_8 = module1<Instance8>;

	#[runtime::pallet_index(13)]
	pub type Module1_9 = module1<Instance9>;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Lookup = sp_runtime::traits::IdentityLookup<AccountId>;
	type BaseCallFilter = frame_support::traits::Everything;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletInfo = PalletInfo;
	type OnSetCode = ();
	type Block = Block;
}

impl module1::Config<module1::Instance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl module1::Config<module1::Instance2> for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl module1::Config<module1::Instance3> for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl module1::Config<module1::Instance4> for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl module1::Config<module1::Instance5> for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl module1::Config<module1::Instance6> for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl module1::Config<module1::Instance7> for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl module1::Config<module1::Instance8> for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl module1::Config<module1::Instance9> for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl module2::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl nested::module3::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}
impl module3::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

fn test_pub() -> AccountId {
	AccountId::from_raw([0; 32])
}

#[test]
fn check_modules_error_type() {
	sp_io::TestExternalities::default().execute_with(|| {
		assert_eq!(
			Module1_1::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 31,
				error: [0; 4],
				message: Some("Something")
			})),
		);
		assert_eq!(
			Module2::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 32,
				error: [0; 4],
				message: Some("Something")
			})),
		);
		assert_eq!(
			Module1_2::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 33,
				error: [0; 4],
				message: Some("Something")
			})),
		);
		assert_eq!(
			NestedModule3::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 34,
				error: [0; 4],
				message: Some("Something")
			})),
		);
		assert_eq!(
			Module1_3::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 6,
				error: [0; 4],
				message: Some("Something")
			})),
		);
		assert_eq!(
			Module1_4::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 3,
				error: [0; 4],
				message: Some("Something")
			})),
		);
		assert_eq!(
			Module1_5::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 4,
				error: [0; 4],
				message: Some("Something")
			})),
		);
		assert_eq!(
			Module1_6::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 1,
				error: [0; 4],
				message: Some("Something")
			})),
		);
		assert_eq!(
			Module1_7::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 2,
				error: [0; 4],
				message: Some("Something")
			})),
		);
		assert_eq!(
			Module1_8::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 12,
				error: [0; 4],
				message: Some("Something")
			})),
		);
		assert_eq!(
			Module1_9::fail(frame_system::Origin::<Runtime>::Root.into()),
			Err(DispatchError::Module(ModuleError {
				index: 13,
				error: [0; 4],
				message: Some("Something")
			})),
		);
	})
}

#[test]
fn integrity_test_works() {
	__construct_runtime_integrity_test::runtime_integrity_tests();
	assert_eq!(IntegrityTestExec::get(), 2);
}

#[test]
fn origin_codec() {
	use codec::Encode;

	let origin = OriginCaller::system(frame_system::RawOrigin::None);
	assert_eq!(origin.encode()[0], 30);

	let origin = OriginCaller::Module1_1(module1::Origin(Default::default()));
	assert_eq!(origin.encode()[0], 31);

	let origin = OriginCaller::Module2(module2::Origin);
	assert_eq!(origin.encode()[0], 32);

	let origin = OriginCaller::Module1_2(module1::Origin(Default::default()));
	assert_eq!(origin.encode()[0], 33);

	let origin = OriginCaller::NestedModule3(nested::module3::Origin);
	assert_eq!(origin.encode()[0], 34);

	let origin = OriginCaller::Module3(module3::Origin(Default::default()));
	assert_eq!(origin.encode()[0], 35);

	let origin = OriginCaller::Module1_6(module1::Origin(Default::default()));
	assert_eq!(origin.encode()[0], 1);

	let origin = OriginCaller::Module1_7(module1::Origin(Default::default()));
	assert_eq!(origin.encode()[0], 2);

	let origin = OriginCaller::Module1_8(module1::Origin(Default::default()));
	assert_eq!(origin.encode()[0], 12);

	let origin = OriginCaller::Module1_9(module1::Origin(Default::default()));
	assert_eq!(origin.encode()[0], 13);
}

#[test]
fn event_codec() {
	use codec::Encode;

	let event = frame_system::Event::<Runtime>::ExtrinsicSuccess {
		dispatch_info: DispatchEventInfo {
			weight: Default::default(),
			class: Default::default(),
			pays_fee: Default::default(),
		},
	};
	assert_eq!(RuntimeEvent::from(event).encode()[0], 30);

	let event = module1::Event::<Runtime, module1::Instance1>::A(test_pub());
	assert_eq!(RuntimeEvent::from(event).encode()[0], 31);

	let event = module2::Event::A;
	assert_eq!(RuntimeEvent::from(event).encode()[0], 32);

	let event = module1::Event::<Runtime, module1::Instance2>::A(test_pub());
	assert_eq!(RuntimeEvent::from(event).encode()[0], 33);

	let event = nested::module3::Event::A;
	assert_eq!(RuntimeEvent::from(event).encode()[0], 34);

	let event = module3::Event::A;
	assert_eq!(RuntimeEvent::from(event).encode()[0], 35);

	let event = module1::Event::<Runtime, module1::Instance5>::A(test_pub());
	assert_eq!(RuntimeEvent::from(event).encode()[0], 4);

	let event = module1::Event::<Runtime, module1::Instance6>::A(test_pub());
	assert_eq!(RuntimeEvent::from(event).encode()[0], 1);

	let event = module1::Event::<Runtime, module1::Instance7>::A(test_pub());
	assert_eq!(RuntimeEvent::from(event).encode()[0], 2);

	let event = module1::Event::<Runtime, module1::Instance8>::A(test_pub());
	assert_eq!(RuntimeEvent::from(event).encode()[0], 12);

	let event = module1::Event::<Runtime, module1::Instance9>::A(test_pub());
	assert_eq!(RuntimeEvent::from(event).encode()[0], 13);
}

#[test]
fn call_codec() {
	use codec::Encode;
	assert_eq!(RuntimeCall::System(frame_system::Call::remark { remark: vec![1] }).encode()[0], 30);
	assert_eq!(RuntimeCall::Module1_1(module1::Call::fail {}).encode()[0], 31);
	assert_eq!(RuntimeCall::Module2(module2::Call::fail {}).encode()[0], 32);
	assert_eq!(RuntimeCall::Module1_2(module1::Call::fail {}).encode()[0], 33);
	assert_eq!(RuntimeCall::NestedModule3(nested::module3::Call::fail {}).encode()[0], 34);
	assert_eq!(RuntimeCall::Module3(module3::Call::fail {}).encode()[0], 35);
	assert_eq!(RuntimeCall::Module1_4(module1::Call::fail {}).encode()[0], 3);
	assert_eq!(RuntimeCall::Module1_6(module1::Call::fail {}).encode()[0], 1);
	assert_eq!(RuntimeCall::Module1_7(module1::Call::fail {}).encode()[0], 2);
	assert_eq!(RuntimeCall::Module1_8(module1::Call::fail {}).encode()[0], 12);
	assert_eq!(RuntimeCall::Module1_9(module1::Call::fail {}).encode()[0], 13);
}

#[test]
fn call_compact_attr() {
	use codec::Encode;
	let call: module3::Call<Runtime> = module3::Call::aux_1 { data: 1 };
	let encoded = call.encode();
	assert_eq!(2, encoded.len());
	assert_eq!(vec![1, 4], encoded);

	let call: module3::Call<Runtime> = module3::Call::aux_2 { data: 1, data2: 2 };
	let encoded = call.encode();
	assert_eq!(6, encoded.len());
	assert_eq!(vec![2, 1, 0, 0, 0, 8], encoded);
}

#[test]
fn call_encode_is_correct_and_decode_works() {
	use codec::{Decode, Encode};
	let call: module3::Call<Runtime> = module3::Call::fail {};
	let encoded = call.encode();
	assert_eq!(vec![0], encoded);
	let decoded = module3::Call::<Runtime>::decode(&mut &encoded[..]).unwrap();
	assert_eq!(decoded, call);

	let call: module3::Call<Runtime> = module3::Call::aux_3 { data: 32, data2: "hello".into() };
	let encoded = call.encode();
	assert_eq!(vec![3, 32, 0, 0, 0, 20, 104, 101, 108, 108, 111], encoded);
	let decoded = module3::Call::<Runtime>::decode(&mut &encoded[..]).unwrap();
	assert_eq!(decoded, call);
}

#[test]
fn call_weight_should_attach_to_call_enum() {
	use frame_support::{
		dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays},
		weights::Weight,
	};
	// operational.
	assert_eq!(
		module3::Call::<Runtime>::operational {}.get_dispatch_info(),
		DispatchInfo {
			call_weight: Weight::from_parts(5, 0),
			extension_weight: Default::default(),
			class: DispatchClass::Operational,
			pays_fee: Pays::Yes
		},
	);
	// custom basic
	assert_eq!(
		module3::Call::<Runtime>::aux_4 {}.get_dispatch_info(),
		DispatchInfo {
			call_weight: Weight::from_parts(3, 0),
			extension_weight: Default::default(),
			class: DispatchClass::Normal,
			pays_fee: Pays::Yes
		},
	);
}

#[test]
fn call_name() {
	use frame_support::traits::GetCallName;
	let name = module3::Call::<Runtime>::aux_4 {}.get_call_name();
	assert_eq!("aux_4", name);
}

#[test]
fn call_metadata() {
	use frame_support::traits::{CallMetadata, GetCallMetadata};
	let call = RuntimeCall::Module3(module3::Call::<Runtime>::aux_4 {});
	let metadata = call.get_call_metadata();
	let expected = CallMetadata { function_name: "aux_4".into(), pallet_name: "Module3".into() };
	assert_eq!(metadata, expected);
}

#[test]
fn get_call_names() {
	use frame_support::traits::GetCallName;
	let call_names = module3::Call::<Runtime>::get_call_names();
	assert_eq!(["fail", "aux_1", "aux_2", "aux_3", "aux_4", "operational"], call_names);
}

#[test]
fn get_module_names() {
	use frame_support::traits::GetCallMetadata;
	let module_names = RuntimeCall::get_module_names();
	assert_eq!(
		[
			"System",
			"Module1_1",
			"Module2",
			"Module1_2",
			"NestedModule3",
			"Module3",
			"Module1_4",
			"Module1_6",
			"Module1_7",
			"Module1_8",
			"Module1_9",
		],
		module_names
	);
}

#[test]
fn call_subtype_conversion() {
	use frame_support::{dispatch::CallableCallFor, traits::IsSubType};
	let call = RuntimeCall::Module3(module3::Call::<Runtime>::fail {});
	let subcall: Option<&CallableCallFor<Module3, Runtime>> = call.is_sub_type();
	let subcall_none: Option<&CallableCallFor<Module2, Runtime>> = call.is_sub_type();
	assert_eq!(Some(&module3::Call::<Runtime>::fail {}), subcall);
	assert_eq!(None, subcall_none);

	let from = RuntimeCall::from(subcall.unwrap().clone());
	assert_eq!(from, call);
}

#[test]
fn test_metadata() {
	use frame_metadata::{
		v14::{StorageEntryType::Plain, *},
		*,
	};
	use scale_info::meta_type;
	use sp_core::Encode;
	use sp_metadata_ir::StorageEntryModifierIR::Optional;

	fn maybe_docs(doc: Vec<&'static str>) -> Vec<&'static str> {
		if cfg!(feature = "no-metadata-docs") {
			vec![]
		} else {
			doc
		}
	}

	let pallets = vec![
		PalletMetadata {
			name: "System",
			storage: None,
			calls: Some(meta_type::<frame_system::Call<Runtime>>().into()),
			event: Some(meta_type::<frame_system::Event<Runtime>>().into()),
			constants: vec![
				PalletConstantMetadata {
					name: "BlockWeights",
					ty: meta_type::<BlockWeights>(),
					value: BlockWeights::default().encode(),
					docs: maybe_docs(vec![" Block & extrinsics weights: base values and limits."]),
				},
				PalletConstantMetadata {
					name: "BlockLength",
					ty: meta_type::<BlockLength>(),
					value: BlockLength::default().encode(),
					docs: maybe_docs(vec![" The maximum length of a block (in bytes)."]),
				},
				PalletConstantMetadata {
					name: "BlockHashCount",
					ty: meta_type::<u64>(),
					value: 10u64.encode(),
					docs: maybe_docs(vec![" Maximum number of block number to block hash mappings to keep (oldest pruned first)."]),
				},
				PalletConstantMetadata {
					name: "DbWeight",
					ty: meta_type::<RuntimeDbWeight>(),
					value: RuntimeDbWeight::default().encode(),
					docs: maybe_docs(vec![" The weight of runtime database operations the runtime can invoke.",]),
				},
				PalletConstantMetadata {
					name: "Version",
					ty: meta_type::<RuntimeVersion>(),
					value: RuntimeVersion::default().encode(),
					docs: maybe_docs(vec![ " Get the chain's in-code version."]),
				},
				PalletConstantMetadata {
					name: "SS58Prefix",
					ty: meta_type::<u16>(),
					value: 0u16.encode(),
					docs: maybe_docs(vec![
						" The designated SS58 prefix of this chain.",
						"",
						" This replaces the \"ss58Format\" property declared in the chain spec. Reason is",
						" that the runtime should know about the prefix in order to make use of it as",
						" an identifier of the chain.",
					]),
				},
			],
			error: Some(meta_type::<frame_system::Error<Runtime>>().into()),
			index: 30,
		},
		PalletMetadata {
			name: "Module1_1",
			storage: None,
			calls: Some(meta_type::<module1::Call<Runtime, module1::Instance1>>().into()),
			event: Some(meta_type::<module1::Event<Runtime, module1::Instance1>>().into()),
			constants: vec![],
			error: Some(meta_type::<module1::Error<Runtime>>().into()),
			index: 31,
		},
		PalletMetadata {
			name: "Module2",
			storage: None,
			calls: Some(meta_type::<module2::Call<Runtime>>().into()),
			event: Some(meta_type::<module2::Event<Runtime>>().into()),
			constants: vec![],
			error: Some(meta_type::<module2::Error<Runtime>>().into()),
			index: 32,
		},
		PalletMetadata {
			name: "Module1_2",
			storage: None,
			calls: Some(meta_type::<module1::Call<Runtime, module1::Instance2>>().into()),
			event: Some(meta_type::<module1::Event<Runtime, module1::Instance2>>().into()),
			constants: vec![],
			error: Some(meta_type::<module1::Error<Runtime, module1::Instance2>>().into()),
			index: 33,
		},
		PalletMetadata {
			name: "NestedModule3",
			storage: None,
			calls: Some(meta_type::<nested::module3::Call<Runtime>>().into()),
			event: Some(meta_type::<nested::module3::Event<Runtime>>().into()),
			constants: vec![],
			error: Some(meta_type::<nested::module3::Error<Runtime>>().into()),
			index: 34,
		},
		PalletMetadata {
			name: "Module3",
			storage: Some(PalletStorageMetadata {
				prefix: "Module3",
				entries: vec![
					StorageEntryMetadata {
						name: "Storage",
						modifier: Optional.into(),
						ty: Plain(meta_type::<u32>().into()),
						default: vec![0],
						docs: vec![],
					},
				]
			}),
			calls: Some(meta_type::<module3::Call<Runtime>>().into()),
			event: Some(meta_type::<module3::Event<Runtime>>().into()),
			constants: vec![],
			error: Some(meta_type::<module3::Error<Runtime>>().into()),
			index: 35,
		},
		PalletMetadata {
			name: "Module1_3",
			storage: None,
			calls: None,
			event: Some(meta_type::<module1::Event<Runtime, module1::Instance3>>().into()),
			constants: vec![],
			error: Some(meta_type::<module1::Error<Runtime, module1::Instance3>>().into()),
			index: 6,
		},
		PalletMetadata {
			name: "Module1_4",
			storage: None,
			calls: Some(meta_type::<module1::Call<Runtime, module1::Instance4>>().into()),
			event: Some(meta_type::<module1::Event<Runtime, module1::Instance4>>().into()),
			constants: vec![],
			error: Some(meta_type::<module1::Error<Runtime, module1::Instance4>>().into()),
			index: 3,
		},
		PalletMetadata {
			name: "Module1_5",
			storage: None,
			calls: None,
			event: Some(meta_type::<module1::Event<Runtime, module1::Instance5>>().into()),
			constants: vec![],
			error: Some(meta_type::<module1::Error<Runtime, module1::Instance5>>().into()),
			index: 4,
		},
		PalletMetadata {
			name: "Module1_6",
			storage:None,
			calls: Some(meta_type::<module1::Call<Runtime, module1::Instance6>>().into()),
			event: Some(meta_type::<module1::Event<Runtime, module1::Instance6>>().into()),
			constants: vec![],
			error: Some(meta_type::<module1::Error<Runtime, module1::Instance6>>().into()),
			index: 1,
		},
		PalletMetadata {
			name: "Module1_7",
			storage: None,
			calls: Some(meta_type::<module1::Call<Runtime, module1::Instance7>>().into()),
			event: Some(meta_type::<module1::Event<Runtime, module1::Instance7>>().into()),
			constants: vec![],
			error: Some(meta_type::<module1::Error<Runtime, module1::Instance7>>().into()),
			index: 2,
		},
		PalletMetadata {
			name: "Module1_8",
			storage: None,
			calls: Some(meta_type::<module1::Call<Runtime, module1::Instance8>>().into()),
			event: Some(meta_type::<module1::Event<Runtime, module1::Instance8>>().into()),
			constants: vec![],
			error: Some(meta_type::<module1::Error<Runtime, module1::Instance8>>().into()),
			index: 12,
		},
		PalletMetadata {
			name: "Module1_9",
			storage: None,
			calls: Some(meta_type::<module1::Call<Runtime, module1::Instance9>>().into()),
			event: Some(meta_type::<module1::Event<Runtime, module1::Instance9>>().into()),
			constants: vec![],
			error: Some(meta_type::<module1::Error<Runtime, module1::Instance9>>().into()),
			index: 13,
		},
	];

	let extrinsic = ExtrinsicMetadata {
		ty: meta_type::<UncheckedExtrinsic>(),
		version: 4,
		signed_extensions: vec![SignedExtensionMetadata {
			identifier: "UnitTransactionExtension",
			ty: meta_type::<()>(),
			additional_signed: meta_type::<()>(),
		}],
	};

	let expected_metadata: RuntimeMetadataPrefixed =
		RuntimeMetadataLastVersion::new(pallets, extrinsic, meta_type::<Runtime>()).into();
	let actual_metadata = Runtime::metadata();

	pretty_assertions::assert_eq!(actual_metadata, expected_metadata);
}

#[test]
fn pallet_in_runtime_is_correct() {
	assert_eq!(PalletInfo::index::<System>().unwrap(), 30);
	assert_eq!(PalletInfo::name::<System>().unwrap(), "System");
	assert_eq!(PalletInfo::module_name::<System>().unwrap(), "frame_system");
	assert!(PalletInfo::crate_version::<System>().is_some());

	assert_eq!(PalletInfo::index::<Module1_1>().unwrap(), 31);
	assert_eq!(PalletInfo::name::<Module1_1>().unwrap(), "Module1_1");
	assert_eq!(PalletInfo::module_name::<Module1_1>().unwrap(), "module1");
	assert!(PalletInfo::crate_version::<Module1_1>().is_some());

	assert_eq!(PalletInfo::index::<Module2>().unwrap(), 32);
	assert_eq!(PalletInfo::name::<Module2>().unwrap(), "Module2");
	assert_eq!(PalletInfo::module_name::<Module2>().unwrap(), "module2");
	assert!(PalletInfo::crate_version::<Module2>().is_some());

	assert_eq!(PalletInfo::index::<Module1_2>().unwrap(), 33);
	assert_eq!(PalletInfo::name::<Module1_2>().unwrap(), "Module1_2");
	assert_eq!(PalletInfo::module_name::<Module1_2>().unwrap(), "module1");
	assert!(PalletInfo::crate_version::<Module1_2>().is_some());

	assert_eq!(PalletInfo::index::<NestedModule3>().unwrap(), 34);
	assert_eq!(PalletInfo::name::<NestedModule3>().unwrap(), "NestedModule3");
	assert_eq!(PalletInfo::module_name::<NestedModule3>().unwrap(), "nested::module3");
	assert!(PalletInfo::crate_version::<NestedModule3>().is_some());

	assert_eq!(PalletInfo::index::<Module3>().unwrap(), 35);
	assert_eq!(PalletInfo::name::<Module3>().unwrap(), "Module3");
	assert_eq!(PalletInfo::module_name::<Module3>().unwrap(), "self::module3");
	assert!(PalletInfo::crate_version::<Module3>().is_some());

	assert_eq!(PalletInfo::index::<Module1_3>().unwrap(), 6);
	assert_eq!(PalletInfo::name::<Module1_3>().unwrap(), "Module1_3");
	assert_eq!(PalletInfo::module_name::<Module1_3>().unwrap(), "module1");
	assert!(PalletInfo::crate_version::<Module1_3>().is_some());

	assert_eq!(PalletInfo::index::<Module1_4>().unwrap(), 3);
	assert_eq!(PalletInfo::name::<Module1_4>().unwrap(), "Module1_4");
	assert_eq!(PalletInfo::module_name::<Module1_4>().unwrap(), "module1");
	assert!(PalletInfo::crate_version::<Module1_4>().is_some());

	assert_eq!(PalletInfo::index::<Module1_5>().unwrap(), 4);
	assert_eq!(PalletInfo::name::<Module1_5>().unwrap(), "Module1_5");
	assert_eq!(PalletInfo::module_name::<Module1_5>().unwrap(), "module1");
	assert!(PalletInfo::crate_version::<Module1_5>().is_some());

	assert_eq!(PalletInfo::index::<Module1_6>().unwrap(), 1);
	assert_eq!(PalletInfo::name::<Module1_6>().unwrap(), "Module1_6");
	assert_eq!(PalletInfo::module_name::<Module1_6>().unwrap(), "module1");
	assert!(PalletInfo::crate_version::<Module1_6>().is_some());

	assert_eq!(PalletInfo::index::<Module1_7>().unwrap(), 2);
	assert_eq!(PalletInfo::name::<Module1_7>().unwrap(), "Module1_7");
	assert_eq!(PalletInfo::module_name::<Module1_7>().unwrap(), "module1");
	assert!(PalletInfo::crate_version::<Module1_7>().is_some());

	assert_eq!(PalletInfo::index::<Module1_8>().unwrap(), 12);
	assert_eq!(PalletInfo::name::<Module1_8>().unwrap(), "Module1_8");
	assert_eq!(PalletInfo::module_name::<Module1_8>().unwrap(), "module1");
	assert!(PalletInfo::crate_version::<Module1_8>().is_some());

	assert_eq!(PalletInfo::index::<Module1_9>().unwrap(), 13);
	assert_eq!(PalletInfo::name::<Module1_9>().unwrap(), "Module1_9");
	assert_eq!(PalletInfo::module_name::<Module1_9>().unwrap(), "module1");
	assert!(PalletInfo::crate_version::<Module1_9>().is_some());
}

#[test]
fn test_validate_unsigned() {
	use frame_support::pallet_prelude::*;

	let call = RuntimeCall::NestedModule3(nested::module3::Call::fail {});
	let validity = Runtime::validate_unsigned(TransactionSource::Local, &call).unwrap_err();
	assert_eq!(validity, TransactionValidityError::Invalid(InvalidTransaction::Call));

	let call = RuntimeCall::Module3(module3::Call::fail {});
	let validity = Runtime::validate_unsigned(TransactionSource::Local, &call).unwrap_err();
	assert_eq!(
		validity,
		TransactionValidityError::Unknown(UnknownTransaction::NoUnsignedValidator)
	);
}
