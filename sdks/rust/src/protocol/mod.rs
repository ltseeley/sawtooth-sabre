// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod payload;
pub mod pike;
pub mod state;

use std::error::Error;

#[cfg(not(target_arch = "wasm32"))]
use transact::signing::{hash::HashSigner, Signer};

pub const ADMINISTRATORS_SETTING_ADDRESS: &str =
    "000000a87cb5eafdcca6a814e4add97c4b517d3c530c2f44b31d18e3b0c44298fc1c14";
pub const ADMINISTRATORS_SETTING_KEY: &str = "sawtooth.swa.administrators";

pub const NAMESPACE_REGISTRY_ADDRESS_PREFIX: &str = "00ec00";
pub const CONTRACT_REGISTRY_ADDRESS_PREFIX: &str = "00ec01";
pub const CONTRACT_ADDRESS_PREFIX: &str = "00ec02";
pub const SMART_PERMISSION_ADDRESS_PREFIX: &str = "00ec03";
pub const AGENT_ADDRESS_PREFIX: &str = "cad11d00";
pub const ORG_ADDRESS_PREFIX: &str = "cad11d01";

/// Compute a state address for a given namespace registry.
///
/// # Arguments
///
/// * `namespace` - the address prefix for this namespace
#[cfg(not(target_arch = "wasm32"))]
pub fn compute_namespace_registry_address(namespace: &str) -> Result<Vec<u8>, AddressingError> {
    let prefix = match namespace.get(..6) {
        Some(x) => x,
        None => {
            return Err(AddressingError::InvalidInput(format!(
                "namespace '{}' is less than 6 characters long",
                namespace,
            )));
        }
    };
    let hash = HashSigner::default()
        .sign(prefix.as_bytes())
        .map_err(|err| {
            AddressingError::HashError(format!(
                "failed to hash namespace registry address: {}",
                err
            ))
        })?;
    Ok([&parse_hex(NAMESPACE_REGISTRY_ADDRESS_PREFIX)?, &hash[..64]].concat())
}

/// Compute a state address for a given contract registry.
///
/// # Arguments
///
/// * `name` - the name of the contract registry
#[cfg(not(target_arch = "wasm32"))]
pub fn compute_contract_registry_address(name: &str) -> Result<Vec<u8>, AddressingError> {
    let hash = HashSigner::default().sign(name.as_bytes()).map_err(|err| {
        AddressingError::HashError(format!("failed to hash contract registry address: {}", err,))
    })?;
    Ok([&parse_hex(CONTRACT_REGISTRY_ADDRESS_PREFIX)?, &hash[..64]].concat())
}

/// Compute a state address for a given contract.
///
/// # Arguments
///
/// * `name` - the name of the contract
/// * `version` - the version of the contract
#[cfg(not(target_arch = "wasm32"))]
pub fn compute_contract_address(name: &str, version: &str) -> Result<Vec<u8>, AddressingError> {
    let s = String::from(name) + "," + version;
    let hash = HashSigner::default().sign(s.as_bytes()).map_err(|err| {
        AddressingError::HashError(format!("failed to hash contract address: {}", err))
    })?;
    Ok([&parse_hex(CONTRACT_ADDRESS_PREFIX)?, &hash[..64]].concat())
}

/// Compute a state address for a given smart permission.
///
/// # Arguments
///
/// * `org_id` - the organization's id
/// * `name` - smart permission name
#[cfg(not(target_arch = "wasm32"))]
pub fn compute_smart_permission_address(
    org_id: &str,
    name: &str,
) -> Result<Vec<u8>, AddressingError> {
    let signer = HashSigner::default();
    let org_id_hash = signer.sign(org_id.as_bytes()).map_err(|err| {
        AddressingError::HashError(format!("failed to hash pike org id: {}", err))
    })?;
    let name_hash = signer.sign(name.as_bytes()).map_err(|err| {
        AddressingError::HashError(format!("failed to hash smart permission name: {}", err))
    })?;
    Ok([
        &parse_hex(SMART_PERMISSION_ADDRESS_PREFIX)?,
        &org_id_hash[..6],
        &name_hash[..58],
    ]
    .concat())
}

/// Compute a state address for a given agent name.
///
/// # Arguments
///
/// * `name` - the agent's name
#[cfg(not(target_arch = "wasm32"))]
pub fn compute_agent_address(name: &[u8]) -> Result<Vec<u8>, AddressingError> {
    let hash = HashSigner::default().sign(name).map_err(|err| {
        AddressingError::HashError(format!("failed to hash pike agent address: {}", err))
    })?;
    Ok([&parse_hex(AGENT_ADDRESS_PREFIX)?, &hash[..62]].concat())
}

/// Compute a state address for a given organization id.
///
/// # Arguments
///
/// * `id` - the organization's id
#[cfg(not(target_arch = "wasm32"))]
pub fn compute_org_address(id: &str) -> Result<Vec<u8>, AddressingError> {
    let hash = HashSigner::default().sign(id.as_bytes()).map_err(|err| {
        AddressingError::HashError(format!("failed to hash pike org address: {}", err))
    })?;
    Ok([&parse_hex(ORG_ADDRESS_PREFIX)?, &hash[..62]].concat())
}

/// Convert a hex string to bytes.
fn parse_hex(hex: &str) -> Result<Vec<u8>, AddressingError> {
    if hex.len() % 2 != 0 {
        return Err(AddressingError::InvalidInput(format!(
            "hex string has odd number of digits: {}",
            hex
        )));
    }

    let mut res = vec![];
    for i in (0..hex.len()).step_by(2) {
        res.push(u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| {
            AddressingError::InvalidInput(format!("string contains invalid hex: {}", hex))
        })?);
    }

    Ok(res)
}

#[derive(Debug)]
pub enum AddressingError {
    HashError(String),
    InvalidInput(String),
}

impl Error for AddressingError {}

impl std::fmt::Display for AddressingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AddressingError::HashError(msg) => write!(f, "failed to produce hash: {}", msg),
            AddressingError::InvalidInput(msg) => write!(f, "addressing input is invalid: {}", msg),
        }
    }
}
