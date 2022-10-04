// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use clap::{crate_name, crate_version, App, AppSettings, ArgMatches, SubCommand};
use gdex_core::validator::spawner::ValidatorSpawner;
use multiaddr::Multiaddr;
use std::{path::Path, str::FromStr};
use tracing::info;

const DEFAULT_VALIDATOR_MULTIADDR: &str = "/dns/localhost/tcp/62000/http";
const DEFAULT_METRICS_MULTIADDR: &str = "/dns/localhost/tcp/63000/http";

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
                .args_from_usage("--key-path=<FILE> 'The file containing the node keys'")
                .args_from_usage("--genesis-dir=<FOLDER> 'The folder containing the genesis blob'")
                .args_from_usage("--name=<NAME> 'The validator name'")
                .args_from_usage("--grpc-address=<ADDR> 'The validator grpc address'")
                .args_from_usage("--jsonrpc-address=<ADDR> 'The validator jsonrpc address'")
                .args_from_usage("--metrics-address=<ADDR> 'The metrics address'"),
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

    let key_path = Path::new(matches.value_of("key-path").unwrap()).to_path_buf();

    let name = matches.value_of("name").unwrap();

    let grpc_address = matches.value_of("grpc-address").unwrap_or(DEFAULT_VALIDATOR_MULTIADDR);
    let grpc_address = Multiaddr::from_str(grpc_address).unwrap();

    let jsonrpc_address = matches
        .value_of("jsonrpc-address")
        .unwrap_or(DEFAULT_VALIDATOR_MULTIADDR);
    let jsonrpc_address = Multiaddr::from_str(jsonrpc_address).unwrap();

    let metrics_address = matches.value_of("metrics-address").unwrap_or(DEFAULT_METRICS_MULTIADDR);
    let metrics_address = Multiaddr::from_str(metrics_address).unwrap();

    info!("Spawning validator and json rpc");
    let mut validator_spawner = ValidatorSpawner::new(
        /* db_path */ db_path.clone(),
        /* key_path */ key_path,
        /* genesis_path */ genesis_path.clone(),
        /* grpc_address */ grpc_address.clone(),
        /* jsonrpc_address */ jsonrpc_address.clone(),
        /* metrics_address */ metrics_address,
        /* validator_name */ name.to_string(),
    );
    validator_spawner.spawn_validator().await;

    validator_spawner.await_handles().await;
}
