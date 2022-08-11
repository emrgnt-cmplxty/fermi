//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use anyhow::{anyhow, bail};
use clap::Parser;
use gdex_core::{
    builder::network_config::NetworkConfigBuilder,
    config::{
        gateway::GatewayConfig,
        gdex_config_dir,
        genesis::GenesisConfig,
        network::NetworkConfig,
        node::{default_json_rpc_address, default_websocket_address},
        Config, PersistedConfig, GDEX_CLIENT_CONFIG, GDEX_FULLNODE_CONFIG, GDEX_GATEWAY_CONFIG, GDEX_GENESIS_FILENAME,
        GDEX_KEYSTORE_FILENAME, GDEX_NETWORK_CONFIG,
    },
    genesis_ceremony::{run, Ceremony, CeremonyCommand},
};
use gdex_types::{
    account::AccountKeyPair,
    crypto::{get_key_pair_from_rng, KeypairTraits, ToFromBytes},
    utils,
};
use multiaddr::Multiaddr;
use std::{fs, io::Write, num::NonZeroUsize, path::PathBuf};
use tracing::info;

/// Note, the code in this struct is inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui/src/sui_commands.rs
/// commit #e91604e0863c86c77ea1def8d9bd116127bee0bc.
/// This enum is responsible for routing input commands to the GDEX CLI

/// a for loop iterator function
pub fn generate_for_loop_on_vector(vec: &Vec<u64>) {
    for i in vec {
        println!("{}", i);
    }
}

#[derive(Parser)]
pub enum GDEXCommand {
    /// GDEX networking
    /// Starts a validator node for the GDEX network
    #[clap(name = "start")]
    Start {
        #[clap(value_parser, long = "network.config")]
        config: Option<PathBuf>,
        #[clap(
            value_parser,
            long,
            help = "Specify the maximum number of ticks to run the network for, helpful for testing and debugging."
        )]
        debug_max_ticks: Option<u64>,
    },
    /// Initialize generation of genesis blob
    #[clap(name = "init-genesis")]
    InitGenesis {
        #[clap(value_parser, long, help = "Path to save genesis blob file")]
        path: Option<PathBuf>,
    },
    /// Add validator to the genesis blob
    #[clap(name = "add-validator-genesis")]
    AddValidatorGenesis {
        #[clap(value_parser, long, help = "Path to save genesis blob file")]
        path: Option<PathBuf>,
        #[clap(value_parser, long, help = "validator name")]
        name: String,
        #[clap(value_parser, long, help = "Validator keystore path")]
        key_file: PathBuf,
        #[clap(value_parser, long, help = "Network address")]
        network_address: Option<Multiaddr>,
        #[clap(value_parser, long, help = "Narwhal primary to primary port")]
        narwhal_primary_to_primary: Option<Multiaddr>,
        #[clap(value_parser, long, help = "Network worker to primary")]
        narwhal_worker_to_primary: Option<Multiaddr>,
        #[clap(value_parser, long, help = "Network primary to worker")]
        narwhal_primary_to_worker: Option<Multiaddr>,
        #[clap(value_parser, long, help = "Network worker to worker")]
        narwhal_worker_to_worker: Option<Multiaddr>,
        #[clap(value_parser, long, help = "Network consensus address")]
        narwhal_consensus_address: Option<Multiaddr>,
    },
    /// Add controllers to the genesis blob
    #[clap(name = "add-controllers-genesis")]
    AddControllersGenesis {
        #[clap(value_parser, long, help = "Path to save genesis blob file")]
        path: Option<PathBuf>,
    },
    #[clap(name = "build-genesis")]
    BuildGenesis {
        #[clap(value_parser, long, help = "Path to save genesis blob file")]
        path: Option<PathBuf>,
    },
    #[clap(name = "verify-and-sign-genesis")]
    VerifyAndSignGenesis {
        #[clap(value_parser, long, help = "Path to save genesis blob file")]
        path: Option<PathBuf>,
        #[clap(value_parser, long, help = "Validator keystore path")]
        key_file: PathBuf,
    },
    #[clap(name = "finalize-genesis")]
    FinalizeGenesis {
        #[clap(value_parser, long, help = "Path to save genesis blob file")]
        path: Option<PathBuf>,
    },
    /// Bootstraps and initializes a new GDEX network genesis config
    #[clap(name = "genesis")]
    Genesis {
        #[clap(value_parser, long, help = "Start genesis with a given config file")]
        from_config: Option<PathBuf>,
        #[clap(
            value_parser,
            long,
            help = "Build a genesis config, write it to the specified path, and exit"
        )]
        write_config: Option<PathBuf>,
        #[clap(value_parser, long)]
        working_dir: Option<PathBuf>,
        #[clap(value_parser, short, long, help = "Forces overwriting existing configuration")]
        force: bool,
    },
    /// Participate in the genesis ceremony
    GenesisCeremony(Ceremony),

    /// Generate a persistant keystore for future use
    GenerateKeystore {
        #[clap(value_parser, help = "Specify a path for the keystore")]
        keystore_path: Option<PathBuf>,
        #[clap(value_parser, help = "Specify a name for the keystore")]
        keystore_name: Option<String>,
    },
}

impl GDEXCommand {
    /// Execute an input command via a simple match
    pub async fn execute(self) -> Result<(), anyhow::Error> {
        match self {
            GDEXCommand::Start {
                config,
                debug_max_ticks,
            } => {
                // Load the config of the GDEX authority.
                let network_config_path = config.clone().unwrap_or(gdex_config_dir()?.join(GDEX_NETWORK_CONFIG));
                let _network_config: NetworkConfig = PersistedConfig::read(&network_config_path).map_err(|err| {
                    err.context(format!(
                        "Cannot open GDEX network config file at {:?}",
                        network_config_path
                    ))
                })?;

                // Commenting out the swarm code, here we launch associated network infra
                // let mut swarm =
                //     Swarm::builder().from_network_config(gdex_config_dir()?, network_config);
                // swarm.launch().await?;

                let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                let mut i_tick: u64 = 0;
                let debug_max_ticks = debug_max_ticks.unwrap_or(u64::MAX);
                loop {
                    // for node in swarm.validators_mut() {
                    //     node.health_check().await?;
                    // }
                    i_tick += 1;
                    if i_tick >= debug_max_ticks {
                        break Ok(());
                    }
                    interval.tick().await;
                }
            }
            GDEXCommand::InitGenesis { path } => {
                let ceremony = Ceremony {
                    path: path,
                    command: CeremonyCommand::Init,
                };
                ceremony.run().unwrap();
                Ok(())
            }
            GDEXCommand::AddValidatorGenesis {
                path,
                name,
                key_file,
                network_address,
                narwhal_primary_to_primary,
                narwhal_worker_to_primary,
                narwhal_primary_to_worker,
                narwhal_worker_to_worker,
                narwhal_consensus_address,
            } => {
                let ceremony = Ceremony {
                    path: path,
                    command: CeremonyCommand::AddValidator {
                        name,
                        key_file,
                        network_address: network_address.unwrap_or(utils::new_network_address()),
                        narwhal_primary_to_primary: narwhal_primary_to_primary.unwrap_or(utils::new_network_address()),
                        narwhal_worker_to_primary: narwhal_worker_to_primary.unwrap_or(utils::new_network_address()),
                        narwhal_primary_to_worker: narwhal_primary_to_worker.unwrap_or(utils::new_network_address()),
                        narwhal_worker_to_worker: narwhal_worker_to_worker.unwrap_or(utils::new_network_address()),
                        narwhal_consensus_address: narwhal_consensus_address.unwrap_or(utils::new_network_address()),
                    },
                };
                ceremony.run().unwrap();
                Ok(())
            }
            GDEXCommand::AddControllersGenesis { path } => {
                let ceremony = Ceremony {
                    path: path,
                    command: CeremonyCommand::AddControllers,
                };
                ceremony.run().unwrap();
                Ok(())
            }
            GDEXCommand::BuildGenesis { path } => {
                let ceremony = Ceremony {
                    path: path,
                    command: CeremonyCommand::Build,
                };
                ceremony.run().unwrap();
                Ok(())
            }
            GDEXCommand::VerifyAndSignGenesis { path, key_file } => {
                let ceremony = Ceremony {
                    path: path,
                    command: CeremonyCommand::VerifyAndSign { key_file },
                };
                ceremony.run().unwrap();
                Ok(())
            }
            GDEXCommand::FinalizeGenesis { path } => {
                let ceremony = Ceremony {
                    path: path,
                    command: CeremonyCommand::Finalize,
                };
                ceremony.run().unwrap();
                Ok(())
            }
            GDEXCommand::Genesis {
                working_dir,
                force,
                from_config,
                write_config,
            } => {
                let gdex_config_dir = &match working_dir {
                    // if a directory is specified, it must exist (it
                    // will not be created)
                    Some(v) => v,
                    // create default GDEX config dir if not specified
                    // on the command line and if it does not exist
                    // yet
                    None => {
                        let config_path = gdex_config_dir()?;
                        fs::create_dir_all(&config_path)?;
                        config_path
                    }
                };

                // if GDEX config dir is not empty then either clean it
                // up (if --force/-f option was specified or report an
                if write_config.is_none()
                // error
                    && gdex_config_dir
                        .read_dir()
                        .map_err(|err| {
                            anyhow!(err).context(format!("Cannot open GDEX config dir {:?}", gdex_config_dir))
                        })?
                        .next()
                        .is_some()
                {
                    if force {
                        fs::remove_dir_all(gdex_config_dir).map_err(|err| {
                            anyhow!(err).context(format!("Cannot remove GDEX config dir {:?}", gdex_config_dir))
                        })?;
                        fs::create_dir(gdex_config_dir).map_err(|err| {
                            anyhow!(err).context(format!("Cannot create GDEX config dir {:?}", gdex_config_dir))
                        })?;
                    } else {
                        bail!("Cannot run genesis with non-empty GDEX config directory {}, please use --force/-f option to remove existing configuration", gdex_config_dir.to_str().unwrap());
                    }
                }

                let network_path = gdex_config_dir.join(GDEX_NETWORK_CONFIG);
                let genesis_path = gdex_config_dir.join(GDEX_GENESIS_FILENAME);
                let client_path = gdex_config_dir.join(GDEX_CLIENT_CONFIG);
                let gateway_path = gdex_config_dir.join(GDEX_GATEWAY_CONFIG);
                let keystore_path = gdex_config_dir.join(GDEX_KEYSTORE_FILENAME);
                let gateway_db_folder_path = gdex_config_dir.join("gateway_client_db");

                let mut genesis_conf = match from_config {
                    Some(path) => PersistedConfig::read(&path)?,
                    None => GenesisConfig::for_local_testing(),
                };

                if let Some(path) = write_config {
                    let persisted = genesis_conf.persisted(&path);
                    persisted.save()?;
                    return Ok(());
                }

                let validator_info = genesis_conf.validator_genesis_info.take();
                let mut network_config = if let Some(validators) = validator_info {
                    NetworkConfigBuilder::new(gdex_config_dir)
                        .initial_accounts_config(genesis_conf)
                        .build_with_validators(validators)
                } else {
                    NetworkConfigBuilder::new(gdex_config_dir)
                        .committee_size(NonZeroUsize::new(genesis_conf.committee_size).unwrap())
                        .initial_accounts_config(genesis_conf)
                        .build()
                };

                // Commenting out account and keystore logic as we have not yet ported this over
                // let db_folder_path = gdex_config_dir.join("client_db");
                // let mut accounts: Vec<ValidatorPubKey> = Vec::new();
                // let mut keystore = GDEXKeystore::default();

                // for key in &network_config.account_keys {
                //     let address = key.public().into();
                //     accounts.push(address);
                //     keystore.add_key(address, key.copy())?;
                // }

                network_config.genesis.save(&genesis_path)?;
                for validator in &mut network_config.validator_configs {
                    validator.genesis = gdex_core::config::Genesis::new_from_file(&genesis_path);
                }

                info!("Network genesis completed.");
                network_config.save(&network_path)?;
                info!("Network config file is stored in {:?}.", network_path);

                // keystore.set_path(&keystore_path);
                // keystore.save()?;
                info!("Client keystore is stored in {:?}.", keystore_path);

                // Use the first address if any
                // let active_address = accounts.get(0).copied();

                let validator_set = network_config.validator_set();

                GatewayConfig {
                    db_folder_path: gateway_db_folder_path,
                    validator_set: validator_set.to_owned(),
                    ..Default::default()
                }
                .save(&gateway_path)?;
                info!("Gateway config file is stored in {:?}.", gateway_path);

                // Commenting out wallet and Gateway logic as we have not yet ported this over
                // let wallet_gateway_config = GatewayConfig {
                //     db_folder_path,
                //     validator_set: validator_set.to_owned(),
                //     ..Default::default()
                // };

                // let wallet_config = GDEXClientConfig {
                //     accounts,
                //     keystore: KeystoreType::File(keystore_path),
                //     gateway: ClientType::Embedded(wallet_gateway_config),
                //     active_address,
                // };

                // wallet_config.save(&client_path)?;
                info!("Client config file is stored in {:?}.", client_path);

                let mut fullnode_config = network_config.generate_fullnode_config();
                fullnode_config.json_rpc_address = default_json_rpc_address();
                fullnode_config.websocket_address = default_websocket_address();
                fullnode_config.save(gdex_config_dir.join(GDEX_FULLNODE_CONFIG))?;

                for (i, validator) in network_config.into_validator_configs().into_iter().enumerate() {
                    let path = gdex_config_dir.join(format!("validator-config-{}.yaml", i));
                    validator.save(path)?;
                }

                Ok(())
            }
            GDEXCommand::GenesisCeremony(cmd) => run(cmd),
            GDEXCommand::GenerateKeystore {
                keystore_path,
                keystore_name,
            } => {
                let kp_sender: AccountKeyPair = get_key_pair_from_rng(&mut rand::rngs::OsRng).1;
                let private_keys = kp_sender.private().as_bytes().to_vec();
                let keystore_path = keystore_path.unwrap_or(gdex_config_dir()?);
                let keystore_name = keystore_name.unwrap_or_else(|| String::from(GDEX_KEYSTORE_FILENAME));

                if !keystore_path.exists() {
                    fs::create_dir_all(&keystore_path)?;
                }

                let file_result = fs::File::create(&keystore_path.join(&keystore_name));
                match file_result {
                    Ok(mut file) => {
                        file.write_all(&private_keys)?;
                    }
                    Err(..) => {
                        println!("A keystore already exists at {:?}.", &keystore_path);
                    }
                }

                Ok(())
            }
        }
    }
}
