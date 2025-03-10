// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Cumulus.
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

//! The definition of a [`ConsensusHook`] trait for consensus logic to manage the backlog
//! of parachain blocks ready to submit to the relay chain, as well as some basic implementations.

use super::relay_state_snapshot::RelayChainStateProof;
use core::num::NonZeroU32;
use frame_support::weights::Weight;

/// The possible capacity of the unincluded segment.
#[derive(Clone, Debug, PartialEq)]
pub struct UnincludedSegmentCapacity(UnincludedSegmentCapacityInner);

impl UnincludedSegmentCapacity {
	pub(crate) fn get(&self) -> u32 {
		match self.0 {
			UnincludedSegmentCapacityInner::ExpectParentIncluded => 1,
			UnincludedSegmentCapacityInner::Value(v) => v.get(),
		}
	}

	pub(crate) fn is_expecting_included_parent(&self) -> bool {
		match self.0 {
			UnincludedSegmentCapacityInner::ExpectParentIncluded => true,
			UnincludedSegmentCapacityInner::Value(_) => false,
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UnincludedSegmentCapacityInner {
	ExpectParentIncluded,
	Value(NonZeroU32),
}

impl From<NonZeroU32> for UnincludedSegmentCapacity {
	fn from(value: NonZeroU32) -> Self {
		UnincludedSegmentCapacity(UnincludedSegmentCapacityInner::Value(value))
	}
}

/// The consensus hook for dealing with the unincluded segment.
///
/// Higher-level and user-configurable consensus logic is more informed about the
/// desired unincluded segment length, as well as any rules for adapting it dynamically
/// according to the relay-chain state.
pub trait ConsensusHook {
	/// This hook is called partway through the `set_validation_data` inherent in parachain-system.
	///
	/// The hook is allowed to panic if customized consensus rules aren't met and is required
	/// to return a maximum capacity for the unincluded segment with weight consumed.
	fn on_state_proof(state_proof: &RelayChainStateProof) -> (Weight, UnincludedSegmentCapacity);
}

/// A special consensus hook for handling the migration to asynchronous backing gracefully,
/// even if collators haven't been updated to provide the last included parent in the state
/// proof yet.
///
/// This behaves as though the parent is included, even if the relay chain state proof doesn't
/// contain the included para head. If the para head is present in the state proof, this does ensure
/// the parent is included.
pub struct ExpectParentIncluded;

impl ConsensusHook for ExpectParentIncluded {
	fn on_state_proof(_state_proof: &RelayChainStateProof) -> (Weight, UnincludedSegmentCapacity) {
		(
			Weight::zero(),
			UnincludedSegmentCapacity(UnincludedSegmentCapacityInner::ExpectParentIncluded),
		)
	}
}

/// A consensus hook for a fixed unincluded segment length. This hook does nothing but
/// set the capacity of the unincluded segment to the constant N.
///
/// Since it is illegal to provide an unincluded segment length of 0, this sets a minimum of
/// 1.
pub struct FixedCapacityUnincludedSegment<const N: u32>;

impl<const N: u32> ConsensusHook for FixedCapacityUnincludedSegment<N> {
	fn on_state_proof(_state_proof: &RelayChainStateProof) -> (Weight, UnincludedSegmentCapacity) {
		(
			Weight::zero(),
			NonZeroU32::new(core::cmp::max(N, 1))
				.expect("1 is the minimum value and non-zero; qed")
				.into(),
		)
	}
}

/// A fixed-capacity unincluded segment hook, which requires that the parent block is
/// included prior to the current block being authored.
///
/// This is a simple type alias around a fixed-capacity unincluded segment with a size of 1.
pub type RequireParentIncluded = FixedCapacityUnincludedSegment<1>;
