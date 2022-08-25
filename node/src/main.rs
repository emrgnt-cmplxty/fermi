// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use clap::{crate_name, crate_version, App, AppSettings, ArgMatches, SubCommand};
use gdex_core::{relayer::spawner::RelayerSpawner, validator::spawner::ValidatorSpawner};
use multiaddr::Multiaddr;
use std::{path::Path, str::FromStr, sync::Arc};
use tracing::info;

const DEFAULT_RELAY_MULTIADDR: &str = "/dns/localhost/tcp/62000/http";
const DEFAULT_VALIDATOR_MULTIADDR: &str = "/dns/localhost/tcp/63000/http";

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("A research implementation of Narwhal and Tusk.")
        .args_from_usage("-v... 'Sets the level of verbosity'")
        .subcommand(
            SubCommand::with_name("run")
                .about("Run a node")
                .args_from_usage("--db-dir=<FOLDER> 'The folder containing a the database'")
                .args_from_usage("--key-dir=<FOLDER> 'The file containing the node keys'")
                .args_from_usage("--genesis-dir=<FOLDER> 'The folder containing the genesis blob'")
                .args_from_usage("--validator-name=<NAME> 'The validator name'")
                .args_from_usage("--validator-address=<PORT> 'The validator port'")
                .args_from_usage("--relayer-address=<PORT> 'The relayer port'"),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    // In benchmarks, transactions are not deserializable => many errors at the debug level
    // Moreover, we need RFC 3339 timestamps to parse properly => we use a custom subscriber.

    let tracing_level = match matches.occurrences_of("v") {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        _ => "trace",
    };

    // some of the network is very verbose, so we require more 'v's
    let network_tracing_level = match matches.occurrences_of("v") {
        0 | 1 => "error",
        2 => "warn",
        3 => "info",
        4 => "debug",
        _ => "trace",
    };

    let log_filter = format!("{tracing_level},h2={network_tracing_level},tower={network_tracing_level},hyper={network_tracing_level},tonic::transport={network_tracing_level}");

    let _guard = telemetry_subscribers::TelemetryConfig::new("gdex-node")
        // load env variables
        .with_env()
        // load special log filter
        .with_log_level(&log_filter)
        .init();

    match matches.subcommand() {
        ("run", Some(sub_matches)) => run(sub_matches).await,
        _ => unreachable!(),
    }
    Ok(())
}

async fn run(matches: &ArgMatches<'_>) {
    let db_dir = matches.value_of("db-dir").unwrap();
    let db_path = Path::new(db_dir).to_path_buf();

    let genesis_dir = matches.value_of("genesis-dir").unwrap();
    let genesis_path = Path::new(genesis_dir).to_path_buf();

    let key_dir = matches.value_of("key-dir").unwrap();
    let key_path = Path::new(key_dir).to_path_buf();

    let validator_name = matches.value_of("validator-name").unwrap();

    let validator_address = matches
        .value_of("validator-address")
        .unwrap_or(DEFAULT_VALIDATOR_MULTIADDR);
    let validator_address = Multiaddr::from_str(validator_address).unwrap();

    info!("Spawning validator and relayer");
    let mut validator_spawner = ValidatorSpawner::new(
        /* db_path */ db_path.clone(),
        /* key_path */ key_path.clone(),
        /* genesis_path */ genesis_path.clone(),
        /* validator_address */ validator_address.clone(),
        /* validator_name */ validator_name.to_string(),
    );

    validator_spawner.spawn_validator().await;

    let validator_state = validator_spawner.get_validator_state().unwrap();
    validator_state.unhalt_validator();

    let relayer_address = matches.value_of("relayer-address").unwrap_or(DEFAULT_RELAY_MULTIADDR);
    let relayer_address = Multiaddr::from_str(relayer_address).unwrap();

    let mut relayer_spawner = RelayerSpawner::new(Arc::clone(&validator_state), relayer_address);
    relayer_spawner.spawn_relayer().await.unwrap();

    validator_spawner.await_handles().await;
    relayer_spawner.await_handles().await;
}
