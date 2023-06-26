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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::BuildGenesisConfig;
use sp_genesis_builder::Result as BuildResult;
use sp_runtime::format_runtime_string;

/// Helper for implementing [`sp_genesis_builder::GenesisBuilder`] for runtimes.
///
/// Provides common logic. For more info refer to [`sp_genesis_builder::GenesisBuilder`].
pub struct GenesisBuilderHelper<GC>(sp_std::marker::PhantomData<GC>);

impl<GC> GenesisBuilderHelper<GC>
where
	GC: BuildGenesisConfig,
{
	/// Get the default `GenesisConfig` as a JSON blob. For more info refer to
	/// [`sp_genesis_builder::GenesisBuilder::get_default_as_json`]
	pub fn create_default_config() -> sp_std::vec::Vec<u8> {
		serde_json::to_string(&GC::default())
			.expect("serialization to json is expected to work. qed.")
			.into_bytes()
	}

	/// Patch default `GenesisConfig` using given JSON patch and store it in the storage. For more
	/// info refer to [`sp_genesis_builder::GenesisBuilder::build_config_no_defaults`].
	pub fn build_config(json: sp_std::vec::Vec<u8>) -> BuildResult {
		let gc = serde_json::from_slice::<GC>(&json)
			.map_err(|e| format_runtime_string!("Invalid JSON blob: {}", e))?;
		<GC as BuildGenesisConfig>::build(&gc);
		Ok(())
	}
}
