// Copyright 2019-2020 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.


use sp_core::{Pair, Public, sr25519, H160, U256};
use moonbeam_runtime::{
    AccountId, AuraConfig, BalancesConfig, EVMConfig, EthereumConfig, GenesisConfig, GrandpaConfig,
    CouncilConfig, SudoConfig, SystemConfig, WASM_BINARY, Signature
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::ChainType;
use std::collections::BTreeMap;
use std::str::FromStr;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
        TPublic::Pair::from_string(&format!("//{}", seed), None)
                .expect("static values are valid; qed")
                .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
        AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
        AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
        (
                get_from_seed::<AuraId>(s),
                get_from_seed::<GrandpaId>(s),
        )
}

pub fn development_config() -> Result<ChainSpec, String> {
        let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

        Ok(ChainSpec::from_genesis(
                // Name
                "Development",
                // ID
                "dev",
                ChainType::Development,
                move || testnet_genesis(
                        wasm_binary,
                        // Initial PoA authorities
                        vec![
                                authority_keys_from_seed("Alice"),
                        ],
                        // Sudo account
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        // Pre-funded accounts
                        vec![
                                get_account_id_from_seed::<sr25519::Public>("Alice"),
                                get_account_id_from_seed::<sr25519::Public>("Bob"),
                                get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                                get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                        ],
                        true,
                ),
                // Bootnodes
                vec![],
                // Telemetry
                None,
                // Protocol ID
                None,
                // Properties
                None,
                // Extensions
                None,
        ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
        let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

        Ok(ChainSpec::from_genesis(
                // Name
                "Local Testnet",
                // ID
                "local_testnet",
                ChainType::Local,
                move || testnet_genesis(
                        wasm_binary,
                        // Initial PoA authorities
                        vec![
                                authority_keys_from_seed("Alice"),
                                authority_keys_from_seed("Bob"),
                        ],
                        // Sudo account
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        // Pre-funded accounts
                        vec![
                                get_account_id_from_seed::<sr25519::Public>("Alice"),
                                get_account_id_from_seed::<sr25519::Public>("Bob"),
                                get_account_id_from_seed::<sr25519::Public>("Charlie"),
                                get_account_id_from_seed::<sr25519::Public>("Dave"),
                                get_account_id_from_seed::<sr25519::Public>("Eve"),
                                get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                                get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                                get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                                get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                                get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                                get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                                get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                        ],
                        true,
                ),
                // Bootnodes
                vec![],
                // Telemetry
                None,
                // Protocol ID
                None,
                // Properties
                None,
                // Extensions
                None,
        ))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
        wasm_binary: &[u8],
        initial_authorities: Vec<(AuraId, GrandpaId)>,
        root_key: AccountId,
        endowed_accounts: Vec<AccountId>,
        _enable_println: bool,
) -> GenesisConfig {
        let alice_evm_account_id = H160::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap();
        let mut evm_accounts = BTreeMap::new();
        evm_accounts.insert(
                alice_evm_account_id,
                evm::GenesisAccount {
                        nonce: 0.into(),
                        balance: U256::from(123456_123_000_000_000_000_000u128),
                        storage: BTreeMap::new(),
                        code: vec![],
                },
        );
        GenesisConfig {
                system: Some(SystemConfig {
                        // Add Wasm runtime to storage.
                        code: wasm_binary.to_vec(),
                        changes_trie_config: Default::default(),
                }),
                balances: Some(BalancesConfig {
                        // Configure endowed accounts with initial balance of 1 << 60.
                        balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
                }),
                aura: Some(AuraConfig {
                        authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
                }),
                grandpa: Some(GrandpaConfig {
                        authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
                }),
                sudo: Some(SudoConfig {
                        // Assign network admin rights.
                        key: root_key,
                }),
                evm: Some(EVMConfig {
                        accounts: evm_accounts,
                }),
                ethereum: Some(EthereumConfig {}),
                collective_Instance1: Some(CouncilConfig::default()),
                pallet_session: None,
                staking: None,
        }
}
