// gdex
use gdex_core::{
    builder::network_config::NetworkConfigBuilder,
    config::{
        gateway::GatewayConfig,
        gdex_config_dir,
        genesis::GenesisConfig,
        network::NetworkConfig,
        node::{default_json_rpc_address, default_websocket_address},
        Config, PersistedConfig, GDEX_FULLNODE_CONFIG, GDEX_GATEWAY_CONFIG, GDEX_GENESIS_FILENAME,
        GDEX_KEYSTORE_FILENAME, GDEX_NETWORK_CONFIG,
    },
    genesis_ceremony::{run, Ceremony, CeremonyCommand},
};
use gdex_node::faucet_server::FAUCET_PORT;
use gdex_types::{
    account::ValidatorKeyPair,
    crypto::get_key_pair_from_rng,
    proto::{FaucetAirdropRequest, FaucetClient},
    utils,
};
// external
use anyhow::{anyhow, bail};
use clap::Parser;
use multiaddr::Multiaddr;
use std::{fs, num::NonZeroUsize, path::PathBuf};
use tracing::info;

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
        #[clap(value_parser, long, help = "validator stake")]
        stake: u64,
        #[clap(value_parser, long, help = "validator initial balance")]
        balance: u64,
        #[clap(value_parser, long, help = "Validator keystore path")]
        key_file: PathBuf,
        #[clap(value_parser, long, help = "Narwhal primary to primary port")]
        narwhal_primary_to_primary: Option<Multiaddr>,
        #[clap(value_parser, long, help = "Network worker to primary")]
        narwhal_worker_to_primary: Option<Multiaddr>,
        #[clap(value_parser, long, help = "Network primary to worker", value_delimiter = ',')]
        narwhal_primary_to_worker: Option<Vec<Multiaddr>>,
        #[clap(value_parser, long, help = "Network worker to worker", value_delimiter = ',')]
        narwhal_worker_to_worker: Option<Vec<Multiaddr>>,
        #[clap(value_parser, long, help = "Network consensus address", value_delimiter = ',')]
        narwhal_consensus_addresses: Option<Vec<Multiaddr>>,
    },
    /// Add controllers to the genesis blob
    #[clap(name = "add-controllers-genesis")]
    AddControllersGenesis {
        #[clap(value_parser, long, help = "Path to save genesis blob file")]
        path: Option<PathBuf>,
    },
    /// Build genesis blob
    #[clap(name = "build-genesis")]
    BuildGenesis {
        #[clap(value_parser, long, help = "Path to save genesis blob file")]
        path: Option<PathBuf>,
    },
    /// Verify and sign the genesis blob
    #[clap(name = "verify-and-sign-genesis")]
    VerifyAndSignGenesis {
        #[clap(value_parser, long, help = "Path to save genesis blob file")]
        path: Option<PathBuf>,
        #[clap(value_parser, long, help = "Validator keystore path")]
        key_file: PathBuf,
    },
    /// Finalize the genesis blob
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
    /// Airdrop someone a specific amount of funds
    Airdrop {
        #[clap(value_parser, help = "Specify how much you want to airdrop")]
        amount: u64,
        #[clap(value_parser, help = "Specify who you want to airdrop to")]
        airdrop_to: String,
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

                let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                let mut i_tick: u64 = 0;
                let debug_max_ticks = debug_max_ticks.unwrap_or(u64::MAX);
                loop {
                    i_tick += 1;
                    if i_tick >= debug_max_ticks {
                        break Ok(());
                    }
                    interval.tick().await;
                }
            }
            GDEXCommand::InitGenesis { path } => {
                let ceremony = Ceremony {
                    path,
                    command: CeremonyCommand::Init,
                };
                ceremony.run().unwrap();
                Ok(())
            }
            #[allow(clippy::redundant_closure)]
            GDEXCommand::AddValidatorGenesis {
                path,
                name,
                stake,
                balance,
                key_file,
                narwhal_primary_to_primary,
                narwhal_worker_to_primary,
                narwhal_primary_to_worker,
                narwhal_worker_to_worker,
                narwhal_consensus_addresses,
            } => {
                let ceremony = Ceremony {
                    path,
                    command: CeremonyCommand::AddValidator {
                        name,
                        key_file,
                        stake,
                        balance,
                        narwhal_primary_to_primary: narwhal_primary_to_primary
                            .unwrap_or_else(|| utils::new_network_address()),
                        narwhal_worker_to_primary: narwhal_worker_to_primary
                            .unwrap_or_else(|| utils::new_network_address()),
                        narwhal_primary_to_worker: narwhal_primary_to_worker
                            .unwrap_or_else(|| vec![utils::new_network_address()]),
                        narwhal_worker_to_worker: narwhal_worker_to_worker
                            .unwrap_or_else(|| vec![utils::new_network_address()]),
                        narwhal_consensus_addresses: narwhal_consensus_addresses
                            .unwrap_or_else(|| vec![utils::new_network_address()]),
                    },
                };
                ceremony.run().unwrap();
                Ok(())
            }
            GDEXCommand::AddControllersGenesis { path } => {
                let ceremony = Ceremony {
                    path,
                    command: CeremonyCommand::AddControllers,
                };
                ceremony.run().unwrap();
                Ok(())
            }
            GDEXCommand::BuildGenesis { path } => {
                let ceremony = Ceremony {
                    path,
                    command: CeremonyCommand::Build,
                };
                ceremony.run().unwrap();
                Ok(())
            }
            GDEXCommand::VerifyAndSignGenesis { path, key_file } => {
                let ceremony = Ceremony {
                    path,
                    command: CeremonyCommand::VerifyAndSign { key_file },
                };
                ceremony.run().unwrap();
                Ok(())
            }
            GDEXCommand::FinalizeGenesis { path } => {
                let ceremony = Ceremony {
                    path,
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
                    // If a directory is specified, it must exist
                    Some(v) => v,
                    // Create default GDEX config dir if not specified
                    None => {
                        let config_path = gdex_config_dir()?;
                        fs::create_dir_all(&config_path)?;
                        config_path
                    }
                };

                // If GDEX config dir is not empty then either clean it
                // up (if --force/-f option was specified or report an
                if write_config.is_none()
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
                let gateway_path = gdex_config_dir.join(GDEX_GATEWAY_CONFIG);
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

                network_config.genesis.save(&genesis_path)?;
                for validator in &mut network_config.validator_configs {
                    validator.genesis = gdex_core::config::Genesis::new_from_file(&genesis_path);
                }

                network_config.save(&network_path)?;

                let validator_set = network_config.validator_set();

                GatewayConfig {
                    db_folder_path: gateway_db_folder_path,
                    validator_set: validator_set.to_owned(),
                    ..Default::default()
                }
                .save(&gateway_path)?;
                info!("Gateway config file is stored in {:?}.", gateway_path);

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
                let keypair: ValidatorKeyPair =
                    get_key_pair_from_rng::<ValidatorKeyPair, rand::rngs::OsRng>(&mut rand::rngs::OsRng);

                let keystore_path = keystore_path.unwrap_or(gdex_config_dir()?);
                let keystore_name = keystore_name.unwrap_or_else(|| String::from(GDEX_KEYSTORE_FILENAME));
                let keystore_name_raw = format!("raw_{}", keystore_name);

                if !keystore_path.exists() {
                    fs::create_dir_all(&keystore_path)?;
                }

                let file_result = fs::File::create(&keystore_path.join(&keystore_name));
                match file_result {
                    Ok(..) => {
                        println!("Writing keypair to file {:?}", keystore_path.join(&keystore_name));

                        utils::write_keypair_to_file(&keypair, &keystore_path.join(&keystore_name))
                            .expect("An error occurred during key generation.");
                        utils::write_keypair_to_file_raw(&keypair, &keystore_path.join(&keystore_name_raw))
                            .expect("An error occurred during key generation.");
                    }
                    Err(..) => {
                        println!("Error, a keystore already exists at {:?}.", &keystore_path);
                    }
                }

                Ok(())
            }
            GDEXCommand::Airdrop { amount, airdrop_to } => {
                // Address for the faucet
                let addr = format!("http://127.0.0.1:{}", FAUCET_PORT);

                // Client to connect
                let mut client = FaucetClient::connect(addr.to_string()).await?;

                // Creating the gRPC request
                let request = tonic::Request::new(FaucetAirdropRequest {
                    airdrop_to: airdrop_to.to_owned(),
                    amount,
                });

                // Sending the request
                let _response = client.airdrop(request).await?;

                // Printing response and returning Ok(())
                Ok(())
            }
        }
    }
}
