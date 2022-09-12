// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use anyhow::{Context, Result};
use clap::{crate_name, crate_version, App, AppSettings};
use fastcrypto::traits::KeyPair;
use futures::{future::join_all, StreamExt};
use gdex_types::{
    account::{AccountKeyPair, ValidatorKeyPair},
    block::BlockDigest,
    proto::{RelayerClient, RelayerGetLatestBlockInfoRequest, TransactionSubmitterClient},
    transaction::{create_payment_transaction, ConsensusTransaction, SignedTransaction},
    utils::read_keypair_from_file,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::path::PathBuf;
use tokio::{
    net::TcpStream,
    time::{interval, sleep, Duration, Instant},
};
use tracing::{info, subscriber::set_global_default, warn};
use tracing_subscriber::filter::EnvFilter;
use url::Url;

const PRIMARY_ASSET_ID: u64 = 0;

#[cfg(not(tarpaulin))]
fn keys(seed: [u8; 32]) -> Vec<AccountKeyPair> {
    let mut rng = StdRng::from_seed(seed);
    (0..4).map(|_| AccountKeyPair::generate(&mut rng)).collect()
}

fn create_signed_transaction(
    kp_sender: &AccountKeyPair,
    kp_receiver: &AccountKeyPair,
    amount: u64,
    block_digest: BlockDigest,
) -> SignedTransaction {
    // use a dummy batch digest for initial benchmarking
    let gas: u64 = 1000;
    let transaction = create_payment_transaction(
        kp_sender.public().clone(),
        kp_receiver.public(),
        PRIMARY_ASSET_ID,
        amount,
        gas,
        block_digest,
    );
    transaction.sign(kp_sender).unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("Benchmark client for Narwhal and Tusk.")
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
    let target = target_str
        .parse::<Url>()
        .with_context(|| format!("Invalid url format {target_str}"))?;
    let relayer = matches
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

    info!("Node address: {target}");
    info!("Transactions rate: {rate} tx/s");

    let client = Client {
        target,
        rate,
        nodes,
        relayer,
        validator_key_fpath,
    };

    // Wait for all nodes to be online and synchronized.
    client.wait().await;

    // Start the benchmark.
    client.send().await.context("Failed to submit transactions")
}

/// TODO - cleanup client to use bench helper
struct Client {
    target: Url,
    relayer: Url,
    rate: u64,
    nodes: Vec<Url>,
    validator_key_fpath: PathBuf,
}

impl Client {
    pub async fn send(&self) -> Result<()> {
        const PRECISION: u64 = 20; // Sample precision.
        const BURST_DURATION: u64 = 1000 / PRECISION;

        let mut client = TransactionSubmitterClient::connect(self.target.as_str().to_owned())
            .await
            .unwrap();
        let mut relayer_client = RelayerClient::connect(self.relayer.as_str().to_owned()).await.unwrap();
        let block_info_request = RelayerGetLatestBlockInfoRequest {};

        // Submit all transactions.
        let burst = self.rate / PRECISION;
        let mut counter = 0;
        // but, not so large that we can exhaust the primary senders balance
        let interval = interval(Duration::from_millis(BURST_DURATION));
        tokio::pin!(interval);

        // send payments from validator with assets
        // read in private key of validator who will send payment txns
        let keypair: ValidatorKeyPair = read_keypair_from_file(self.validator_key_fpath.clone())?;

        // NOTE: This log entry is used to compute performance.
        info!("Start sending transactions");
        loop {
            // fetch recent block digest before starting another round of payments
            let response = relayer_client.get_latest_block_info(block_info_request.clone()).await;

            let mut block_digest = BlockDigest::new([0; 32]);

            if let Ok(relayer_block_response) = response {
                if let Some(block_info) = relayer_block_response.into_inner().block_info {
                    block_digest = bincode::deserialize(block_info.digest.as_ref()).unwrap()
                }
            };
    
            interval.as_mut().tick().await;
            let now = Instant::now();
            let keypair = keypair.copy();
            let kp_receiver = keys([1; 32]).pop().unwrap();

            if counter == 0 {
                let signed_transaction = create_signed_transaction(&keypair, &kp_receiver, 1, block_digest.clone());
                let serialized_consensus_transaction = match ConsensusTransaction::new(&signed_transaction).serialize()
                {
                    Ok(t) => t,
                    _ => panic!("Error serializing transaction"),
                };
                let transaction_size = serialized_consensus_transaction.len();
                info!("Transactions size: {transaction_size} B");
            }

            let stream = tokio_stream::iter(0..burst).map(move |x| {
                let amount = rand::thread_rng().gen_range(100_000 as u64..5_000_000 as u64);
                if x == counter % burst {
                    // NOTE: This log entry is used to compute performance.
                    info!("Sending sample transaction {counter}");
                }
                let signed_tranasction =
                    create_signed_transaction(&keypair, &kp_receiver, amount, block_digest.clone());
                signed_tranasction
            });

            if let Err(e) = client.submit_transaction_stream(stream).await {
                warn!("Failed to send transaction: {e}");
            }

            info!("now.elapsed().as_millis()={}", now.elapsed().as_millis());
            info!("submitted tps={}", burst * 1000 / (now.elapsed().as_millis() as u64));
            if now.elapsed().as_millis() > BURST_DURATION as u128 {
                // NOTE: This log entry is used to compute performance.
                warn!("Transaction rate too high for this client");
            }
            counter += 1;
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
