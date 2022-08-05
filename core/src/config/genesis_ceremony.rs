//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! This file is largely inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui/src/genesis_ceremony.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc

use super::genesis::{Builder, Genesis, MasterController};
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use gdex_types::account::{AccountPubKey, AuthorityKeyPair, AuthorityPubKey, AuthorityPubKeyBytes, AuthoritySignature};
use gdex_types::{
    asset::PRIMARY_ASSET_ID,
    crypto::{KeypairTraits, Signer, ToFromBytes, Verifier},
    node::ValidatorInfo,
    utils,
};
use multiaddr::Multiaddr;
use std::convert::{TryFrom, TryInto};
use std::{fs, path::PathBuf};
pub const GENESIS_FILENAME: &str = "genesis.blob";
const GENESIS_BUILDER_SIGNATURE_DIR: &str = "signatures";
const VALIDATOR_FUNDING_AMOUNT: u64 = 1_000_000;

#[derive(Parser)]
pub struct Ceremony {
    #[clap(value_parser, long)]
    path: Option<PathBuf>,

    #[clap(subcommand)]
    command: CeremonyCommand,
}

impl Ceremony {
    pub fn run(self) -> Result<()> {
        run(self)
    }
}

#[derive(Parser)]
pub enum CeremonyCommand {
    Init,

    AddValidator {
        #[clap(value_parser, long)]
        name: String,
        #[clap(value_parser, long)]
        key_file: PathBuf,
        #[clap(value_parser, long)]
        network_address: Multiaddr,
        #[clap(value_parser, long)]
        narwhal_primary_to_primary: Multiaddr,
        #[clap(value_parser, long)]
        narwhal_worker_to_primary: Multiaddr,
        #[clap(value_parser, long)]
        narwhal_primary_to_worker: Multiaddr,
        #[clap(value_parser, long)]
        narwhal_worker_to_worker: Multiaddr,
        #[clap(value_parser, long)]
        narwhal_consensus_address: Multiaddr,
    },

    AddControllers,

    Build,

    VerifyAndSign {
        #[clap(value_parser, long)]
        key_file: PathBuf,
    },

    Finalize,
}

pub fn run(cmd: Ceremony) -> Result<()> {
    let dir = if let Some(path) = cmd.path {
        path
    } else {
        std::env::current_dir()?
    };
    let dir = Utf8PathBuf::try_from(dir)?;

    match cmd.command {
        CeremonyCommand::Init => {
            let builder = Builder::new();
            builder.save(dir)?;
        }

        CeremonyCommand::AddValidator {
            name,
            key_file,
            network_address,
            narwhal_primary_to_primary,
            narwhal_worker_to_primary,
            narwhal_primary_to_worker,
            narwhal_worker_to_worker,
            narwhal_consensus_address,
        } => {
            let mut builder = Builder::load(&dir)?;
            let keypair: AuthorityKeyPair = utils::read_keypair_from_file(key_file)?;
            builder = builder.add_validator(ValidatorInfo {
                name,
                public_key: keypair.public().into(),
                stake: 1,
                delegation: 0,
                network_address,
                narwhal_primary_to_primary,
                narwhal_worker_to_primary,
                narwhal_primary_to_worker,
                narwhal_worker_to_worker,
                narwhal_consensus_address,
            });
            builder.save(dir)?;
        }

        CeremonyCommand::AddControllers => {
            let builder = Builder::load(&dir)?;

            // initialize controllers to default state
            let mut master_controller = MasterController::default();

            // create base asset of the blockchain with the null address as the owner
            let null_creator = AuthorityPubKey::try_from(AuthorityPubKeyBytes::from_bytes(&[0; 32])?)?;
            master_controller
                .bank_controller
                .borrow_mut()
                .create_asset(&null_creator)?;

            // fund and stake the validators with the VALIDATOR_FUNDING_AMOUNT
            for (_key, validator) in builder.validators.iter() {
                let validator_key = AuthorityPubKey::try_from(validator.public_key).unwrap();
                master_controller.bank_controller.borrow_mut().transfer(
                    &null_creator,
                    &validator_key,
                    PRIMARY_ASSET_ID,
                    VALIDATOR_FUNDING_AMOUNT,
                )?;
                master_controller
                    .stake_controller
                    .stake(&validator_key, VALIDATOR_FUNDING_AMOUNT)?;
            }

            builder.set_master_controller(master_controller).save(dir)?;
        }

        CeremonyCommand::Build => {
            let builder = Builder::load(&dir)?;

            let genesis = builder.build();

            genesis.save(dir.join(GENESIS_FILENAME))?;
        }

        CeremonyCommand::VerifyAndSign { key_file } => {
            let keypair: AuthorityKeyPair = utils::read_keypair_from_file(key_file)?;
            let loaded_genesis = Genesis::load(dir.join(GENESIS_FILENAME))?;
            let loaded_genesis_bytes = loaded_genesis.to_bytes();

            let builder = Builder::load(&dir)?;

            let built_genesis = builder.build();
            let built_genesis_bytes = built_genesis.to_bytes();

            if built_genesis != loaded_genesis || built_genesis_bytes != loaded_genesis_bytes {
                return Err(anyhow::anyhow!("loaded genesis does not match built genesis"));
            }

            if !built_genesis
                .validator_set()
                .iter()
                .any(|validator| validator.public_key() == AuthorityPubKeyBytes::from(keypair.public()))
            {
                return Err(anyhow::anyhow!(
                    "provided keypair does not correspond to a validator in the validator set"
                ));
            }

            // Sign the genesis bytes
            let signature: AuthoritySignature = keypair.try_sign(&built_genesis_bytes)?;

            let signature_dir = dir.join(GENESIS_BUILDER_SIGNATURE_DIR);
            std::fs::create_dir_all(&signature_dir)?;

            let hex_name = utils::encode_bytes_hex(&AuthorityPubKeyBytes::from(keypair.public()));
            fs::write(signature_dir.join(hex_name), signature)?;
        }

        CeremonyCommand::Finalize => {
            let genesis = Genesis::load(dir.join(GENESIS_FILENAME))?;
            let genesis_bytes = genesis.to_bytes();

            let mut signatures = std::collections::BTreeMap::new();

            for entry in dir.join(GENESIS_BUILDER_SIGNATURE_DIR).read_dir_utf8()? {
                let entry = entry?;
                if entry.file_name().starts_with('.') {
                    continue;
                }

                let path = entry.path();
                let signature_bytes = fs::read(path)?;
                let signature: AuthoritySignature = AuthoritySignature::from_bytes(&signature_bytes)?;
                let name = path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid signature file"))?;
                let public_key = AuthorityPubKeyBytes::from_bytes(&utils::decode_bytes_hex::<Vec<u8>>(name)?[..])?;
                signatures.insert(public_key, signature);
            }

            for validator in genesis.validator_set() {
                let signature = signatures
                    .remove(&validator.public_key())
                    .ok_or_else(|| anyhow::anyhow!("missing signature for validator {}", validator.name()))?;

                let pk: AccountPubKey = validator.public_key().try_into()?;

                pk.verify(&genesis_bytes, &signature)
                    .with_context(|| format!("failed to validate signature for validator {}", validator.name()))?;
            }

            if !signatures.is_empty() {
                return Err(anyhow::anyhow!(
                    "found extra signatures from entities not in the validator set"
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use gdex_types::{
        account::AuthorityKeyPair,
        crypto::{get_key_pair_from_rng, KeypairTraits},
        node::ValidatorInfo,
        utils,
    };

    #[test]
    fn ceremony() -> Result<()> {
        let dir = tempfile::TempDir::new().unwrap();

        let validators = (0..10)
            .map(|i| {
                let keypair: AuthorityKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
                let info = ValidatorInfo {
                    name: format!("validator-{i}"),
                    public_key: AuthorityPubKeyBytes::from(keypair.public()),
                    stake: 1,
                    delegation: 0,
                    network_address: utils::new_network_address(),
                    narwhal_primary_to_primary: utils::new_network_address(),
                    narwhal_worker_to_primary: utils::new_network_address(),
                    narwhal_primary_to_worker: utils::new_network_address(),
                    narwhal_worker_to_worker: utils::new_network_address(),
                    narwhal_consensus_address: utils::new_network_address(),
                };
                let key_file = dir.path().join(format!("{}.key", info.name));
                utils::write_keypair_to_file(&keypair, &key_file).unwrap();
                (key_file, info)
            })
            .collect::<Vec<_>>();

        // Initialize
        let command = Ceremony {
            path: Some(dir.path().into()),
            command: CeremonyCommand::Init,
        };
        command.run()?;

        // Add the validators
        for (key_file, validator) in &validators {
            let command = Ceremony {
                path: Some(dir.path().into()),
                command: CeremonyCommand::AddValidator {
                    name: validator.name().to_owned(),
                    key_file: key_file.into(),
                    network_address: validator.network_address().to_owned(),
                    narwhal_primary_to_primary: validator.narwhal_primary_to_primary.clone(),
                    narwhal_worker_to_primary: validator.narwhal_worker_to_primary.clone(),
                    narwhal_primary_to_worker: validator.narwhal_primary_to_worker.clone(),
                    narwhal_worker_to_worker: validator.narwhal_worker_to_worker.clone(),
                    narwhal_consensus_address: validator.narwhal_consensus_address.clone(),
                },
            };
            command.run()?;
        }

        let command = Ceremony {
            path: Some(dir.path().into()),
            command: CeremonyCommand::AddControllers,
        };

        command.run()?;

        // Build the Genesis object
        let command = Ceremony {
            path: Some(dir.path().into()),
            command: CeremonyCommand::Build,
        };
        command.run()?;

        // Have all the validators verify and sign genesis
        for (key, _validator) in &validators {
            let command = Ceremony {
                path: Some(dir.path().into()),
                command: CeremonyCommand::VerifyAndSign { key_file: key.into() },
            };
            command.run()?;
        }

        // Finalize the Ceremony
        let command = Ceremony {
            path: Some(dir.path().into()),
            command: CeremonyCommand::Finalize,
        };
        command.run()?;

        Ok(())
    }
}
