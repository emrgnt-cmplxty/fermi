// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
// benchmark % ../target/release/benchmark_orderbook_client http://localhost:3003 --relayer http://localhost:3004 --validator_key_fpath ../.proto/validator-3.key --rate 12500

use anyhow::{Context, Result};
use benchmark_gdex::bench_helper::BenchHelper;
use clap::{crate_name, crate_version, App, AppSettings};
use futures::future::join_all;
use gdex_types::utils::read_keypair_from_file;
use std::path::PathBuf;
use tokio::{
    net::TcpStream,
    time::{interval, sleep, Duration, Instant},
};
use tracing::{info, subscriber::set_global_default, warn};
use tracing_subscriber::filter::EnvFilter;
use url::Url;

const ACCOUNTS_TO_GENERATE: u64 = 100;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("Benchmark client for GDEX Orderbook.")
        .args_from_usage("<ADDR> 'The network address of the node where to send txs'")
        .args_from_usage("--relayer=<ADDR> 'Relayer address to send requests to'")
        .args_from_usage("--rate=<INT> 'The rate (txs/s) at which to send the transactions'")
        .args_from_usage("--validator_key_fpath=<FILE> 'The validator key file'")
        .args_from_usage("--nodes=[ADDR]... 'Network addresses that must be reachable before starting the benchmark.'")
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    cfg_if::cfg_if! {
        if #[cfg(feature = "benchmark")] {
            let timer = tracing_subscriber::fmt::time::UtcTime::rfc_3339();
            let subscriber_builder = tracing_subscriber::fmt::Subscriber::builder()
                                     .with_env_filter(env_filter)
                                     .with_timer(timer).with_ansi(false);
        } else {
            let subscriber_builder = tracing_subscriber::fmt::Subscriber::builder().with_env_filter(env_filter);
        }
    }
    let subscriber = subscriber_builder.with_writer(std::io::stderr).finish();

    set_global_default(subscriber).expect("Failed to set subscriber");

    let target_str = matches.value_of("ADDR").unwrap();
    let validator_url = target_str
        .parse::<Url>()
        .with_context(|| format!("Invalid url format {target_str}"))?;
    let relayer_url = matches
        .value_of("relayer")
        .unwrap()
        .parse::<Url>()
        .context("Invalid relayer url")?;
    let rate = matches
        .value_of("rate")
        .unwrap()
        .parse::<u64>()
        .context("The rate of transactions must be a non-negative integer")?;
    let validator_key_fpath = matches
        .value_of("validator_key_fpath")
        .unwrap()
        .parse::<PathBuf>()
        .context("The path to the validator key.")?;
    let nodes = matches
        .values_of("nodes")
        .unwrap_or_default()
        .into_iter()
        .map(|x| x.parse::<Url>())
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("Invalid url format {target_str}"))?;

    info!("Node URL: {validator_url}");
    info!("Relayer URL: {relayer_url}");
    info!("Transactions rate: {rate} tx/s");

    let primary_keypair = read_keypair_from_file(validator_key_fpath).unwrap();

    let mut client = Client {
        rate,
        nodes,
        bench_helper: BenchHelper::new(primary_keypair),
    };

    // Wait for all nodes to be online and synchronized.
    client.wait().await;

    // initialize the orderbook if running validator 0
    // TODO - find a more intelligent way to avoid double-initialization of the orderbook client
    client.initialize(validator_url, relayer_url).await.unwrap();

    info!("Starting to send transactions...");

    // Start the benchmark.
    client.send().await;

    // This line should never execute as the benchmark runs forever.
    Ok(())
}

struct Client {
    rate: u64,
    nodes: Vec<Url>,
    bench_helper: BenchHelper,
}

impl Client {
    pub async fn initialize(&mut self, validator_url: Url, relayer_url: Url) -> Result<()> {
        self.bench_helper
            .initialize(validator_url, relayer_url, [0u8; 32], ACCOUNTS_TO_GENERATE)
            .await;

        self.bench_helper.prepare_orderbook().await;

        Ok(())
    }

    // send continuously bursts transactions until the process is killed
    pub async fn send(&mut self) {
        const PRECISION: u64 = 20; // Sample precision.
        const BURST_DURATION: u64 = 1000 / PRECISION;

        // but, not so large that we can exhaust the primary senders balance
        let interval = interval(Duration::from_millis(BURST_DURATION));

        // NOTE: This log entry is used to compute performance.
        info!("Start sending transactions");

        tokio::pin!(interval);
        loop {
            interval.as_mut().tick().await;
            let now = Instant::now();

            let burst = self.rate / PRECISION;
            self.bench_helper.burst_orderbook(burst).await;

            if now.elapsed().as_millis() > BURST_DURATION as u128 {
                // NOTE: This log entry is used to compute performance.
                warn!("Transaction rate too high for this client");
            }
        }
    }

    pub async fn wait(&self) {
        // Wait for all nodes to be online.
        info!("Waiting for all nodes to be online...");
        join_all(self.nodes.iter().cloned().map(|address| {
            tokio::spawn(async move {
                while TcpStream::connect(&*address.socket_addrs(|| None).unwrap())
                    .await
                    .is_err()
                {
                    sleep(Duration::from_millis(10)).await;
                }
            })
        }))
        .await;
    }
}
