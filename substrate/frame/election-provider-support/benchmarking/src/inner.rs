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

//! Election provider support pallet benchmarking.
//! This is separated into its own crate to avoid bloating the size of the runtime.

use alloc::vec::Vec;
use codec::Decode;
use frame_benchmarking::v2::*;
use frame_election_provider_support::{NposSolver, PhragMMS, SequentialPhragmen};
use sp_runtime::Perbill;

const VOTERS: [u32; 2] = [1_000, 2_000];
const TARGETS: [u32; 2] = [500, 1_000];
const VOTES_PER_VOTER: [u32; 2] = [5, 16];
const SEED: u32 = 999;

pub trait Config: frame_system::Config {}

pub struct Pallet<T: Config>(frame_system::Pallet<T>);

fn set_up_voters_targets<AccountId: Decode + Clone>(
	voters_len: u32,
	targets_len: u32,
	degree: usize,
) -> (Vec<(AccountId, u64, impl Clone + IntoIterator<Item = AccountId>)>, Vec<AccountId>) {
	// fill targets.
	let mut targets = (0..targets_len)
		.map(|i| frame_benchmarking::account::<AccountId>("Target", i, SEED))
		.collect::<Vec<_>>();
	assert!(targets.len() > degree, "we should always have enough voters to fill");
	targets.truncate(degree);

	// fill voters.
	let voters = (0..voters_len)
		.map(|i| {
			let voter = frame_benchmarking::account::<AccountId>("Voter", i, SEED);
			(voter, 1_000, targets.clone())
		})
		.collect::<Vec<_>>();

	(voters, targets)
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn phragmen(
		// Number of votes in snapshot.
		v: Linear<{ VOTERS[0] }, { VOTERS[1] }>,
		// Number of targets in snapshot.
		t: Linear<{ TARGETS[0] }, { TARGETS[1] }>,
		// Number of votes per voter (ie the degree).
		d: Linear<{ VOTES_PER_VOTER[0] }, { VOTES_PER_VOTER[1] }>,
	) {
		let (voters, targets) = set_up_voters_targets::<T::AccountId>(v, t, d as _);
		let result;

		#[block]
		{
			result = SequentialPhragmen::<T::AccountId, Perbill>::solve(d as _, targets, voters);
		}

		assert!(result.is_ok());
	}

	#[benchmark]
	fn phragmms(
		// Number of votes in snapshot.
		v: Linear<{ VOTERS[0] }, { VOTERS[1] }>,
		// Number of targets in snapshot.
		t: Linear<{ TARGETS[0] }, { TARGETS[1] }>,
		// Number of votes per voter (ie the degree).
		d: Linear<{ VOTES_PER_VOTER[0] }, { VOTES_PER_VOTER[1] }>,
	) {
		let (voters, targets) = set_up_voters_targets::<T::AccountId>(v, t, d as _);
		let result;

		#[block]
		{
			result = PhragMMS::<T::AccountId, Perbill>::solve(d as _, targets, voters);
		}

		assert!(result.is_ok());
	}
}
