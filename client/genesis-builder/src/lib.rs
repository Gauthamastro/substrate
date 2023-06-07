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

use frame_support::traits::GenesisBuild;
use serde_json::Value;

/// Helper for implementing [`sp_genesis_builder::GenesisBuilder`] for runtimes. Provides common
/// logic. For more info refer to [`sp_genesis_builder::GenesisBuilder`].
pub struct GenesisBuilderHelper<R, GC>(sp_std::marker::PhantomData<(R, GC)>);

impl<R, GC> GenesisBuilderHelper<R, GC>
where
	GC: Default + GenesisBuild<R>,
{
	/// Get the default `GenesisConfig` as a JSON blob.
	pub fn get_default_as_json() -> sp_std::vec::Vec<u8> {
		serde_json::to_string(&GC::default())
			.expect("serialization to json is expected to work. qed.")
			.into_bytes()
	}

	/// Build `GenesisConfig` from a JSON blob and store it in the storage.
	pub fn build_from_json(json: sp_std::vec::Vec<u8>) {
		let gc = serde_json::from_slice::<GC>(&json)
			.expect("provided json blob is expected to be valid. qed.");
		<GC as GenesisBuild<R>>::build(&gc);
	}

	/// Patch default `GenesisConfig` using given JSON patch and store it in the storage.
	pub fn build_with_patch(patch_json: sp_std::vec::Vec<u8>) {
		let mut json = serde_json::to_value(&GC::default())
			.expect("serialization to json is expected to work. qed.");

		let patch: Value = serde_json::from_slice(&patch_json)
			.expect("provided json patch is expected to be valid. qed.");

		merge(&mut json, patch);

		let gc = serde_json::from_value::<GC>(json)
			.expect("patched default config json is expected to be valid. qed.");

		<GC as GenesisBuild<R>>::build(&gc);
	}
}

/// Recursively merges two JSON objects, `a` and `b`, into a single object.
///
/// If a key exists in both objects, the value from `b` will override the value from `a`.
/// If a key exists in `b` with a `null` value, it will be removed from `a`.
/// If a key exists only in `b` and not in `a`, it will be added to `a`.
///
/// # Arguments
///
/// * `a` - A mutable reference to the target JSON object to merge into.
/// * `b` - The JSON object to merge with `a`.
fn merge(a: &mut Value, b: Value) {
	match (a, b) {
		(Value::Object(a), Value::Object(b)) =>
			for (k, v) in b {
				if v.is_null() {
					a.remove(&k);
				} else {
					merge(a.entry(k).or_insert(Value::Null), v);
				}
			},
		(a, b) => *a = b,
	};
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test1_simple_merge() {
		let mut j1 = json!({ "a":123 });
		merge(&mut j1, json!({ "b":256 }));
		assert_eq!(j1, json!({ "a":123, "b":256 }));
	}

	#[test]
	fn test2_patch_simple_merge_nested() {
		let mut j1 = json!({
			"a": {
				"name": "xxx",
				"value": 123
			},
			"b": { "c" : { "inner_name": "yyy" } }
		});

		let j2 = json!({
			"a": {
				"keys": ["a", "b", "c" ]
			}
		});

		merge(&mut j1, j2);
		assert_eq!(
			j1,
			json!({"a":{"keys":["a","b","c"],"name":"xxx","value":123}, "b": { "c" : { "inner_name": "yyy" } }})
		);
	}

	#[test]
	fn test3_patch_overrides_existing_keys() {
		let mut j1 = json!({
			"a": {
				"name": "xxx",
				"value": 123,
				"keys": ["d"]

			}
		});

		let j2 = json!({
			"a": {
				"keys": ["a", "b", "c" ]
			}
		});

		merge(&mut j1, j2);
		assert_eq!(j1, json!({"a":{"keys":["a","b","c"],"name":"xxx","value":123}}));
	}

	#[test]
	fn test4_patch_overrides_existing_keys() {
		let mut j1 = json!({
			"a": {
				"name": "xxx",
				"value": 123,
				"b" : {
					"inner_name": "yyy"
				}
			}
		});

		let j2 = json!({
			"a": {
				"name": "new_name",
				"b" : {
					"inner_name": "inner_new_name"
				}
			}
		});

		merge(&mut j1, j2);
		assert_eq!(
			j1,
			json!({ "a": {"name":"new_name", "value":123, "b" : { "inner_name": "inner_new_name" }} })
		);
	}

	#[test]
	fn test5_patch_overrides_existing_nested_keys() {
		let mut j1 = json!({
			"a": {
				"name": "xxx",
				"value": 123,
				"b": {
					"c": {
						"d": {
							"name": "yyy",
							"value": 256
						}
					}
				}
			},
		});

		let j2 = json!({
			"a": {
				"value": 456,
				"b": {
					"c": {
						"d": {
							"name": "new_name"
						}
					}
				}
			}
		});

		merge(&mut j1, j2);
		assert_eq!(
			j1,
			json!({ "a": {"name":"xxx", "value":456, "b": { "c": { "d": { "name": "new_name", "value": 256 }}}}})
		);
	}

	#[test]
	fn test6_patch_removes_keys_if_null() {
		let mut j1 = json!({
			"a": {
				"name": "xxx",
				"value": 123,
				"enum_variant_1": {
					"name": "yyy",
				}
			},
		});

		let j2 = json!({
			"a": {
				"value": 456,
				"enum_variant_1": null,
				"enum_variant_2": 32,
			}
		});

		merge(&mut j1, j2);
		assert_eq!(j1, json!({ "a": {"name":"xxx", "value":456, "enum_variant_2": 32 }}));
	}
}
