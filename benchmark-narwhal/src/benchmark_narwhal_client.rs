// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use anyhow::{Context, Result};
use bytes::{BufMut as _, BytesMut};
use clap::{crate_name, crate_version, App, AppSettings};
use futures::{future::join_all, StreamExt};
use gdex_types::{
    account::AccountKeyPair,
    transaction::{PaymentRequest, SignedTransaction, Transaction, TransactionVariant},
};
use narwhal_crypto::{
    traits::{KeyPair, Signer},
    Hash, DIGEST_LEN,
};
use narwhal_types::{CertificateDigest, TransactionProto, TransactionsClient};
use rand::{rngs::StdRng, Rng, SeedableRng};
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

fn create_signed_padded_transaction(
    kp_sender: &AccountKeyPair,
    kp_receiver: &AccountKeyPair,
    amount: u64,
    transmission_size: usize,
    is_advanced_execution: bool,
) -> Vec<u8> {
    if is_advanced_execution {
        // use a dummy batch digest for initial benchmarking
        let dummy_certificate_digest = CertificateDigest::new([0; DIGEST_LEN]);

        let transaction_variant = TransactionVariant::PaymentTransaction(PaymentRequest::new(
            kp_receiver.public().clone(),
            PRIMARY_ASSET_ID,
            amount,
        ));
        let transaction = Transaction::new(
            kp_sender.public().clone(),
            dummy_certificate_digest,
            transaction_variant,
        );

        // sign digest and create signed transaction
        let signed_digest = kp_sender.sign(&transaction.digest().get_array()[..]);
        let signed_transaction = SignedTransaction::new(kp_sender.public().clone(), transaction.clone(), signed_digest);

        // serialize and resize the transaction for channel distribution
        let mut padded_signed_transaction = signed_transaction.serialize().unwrap();
        assert!(
            padded_signed_transaction.len() <= transmission_size,
            "please resize to a larger expected byte length"
        );
        padded_signed_transaction.resize(transmission_size, 0);
        padded_signed_transaction
    } else {
        let mut tx = BytesMut::with_capacity(transmission_size);
        tx.put_u8(0u8); // Sample txs start with 0.
        tx.put_u64(amount); // This counter identifies the tx.
        tx.resize(transmission_size, 0u8);
        tx.into()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("Benchmark client for Narwhal and Tusk.")
        .args_from_usage("<ADDR> 'The network address of the node where to send txs'")
        .args_from_usage("--size=<INT> 'The size of each transaction in bytes'")
        .args_from_usage("--rate=<INT> 'The rate (txs/s) at which to send the transactions'")
        .args_from_usage("--execution=<EXECUTION> 'The rate (txs/s) at which to send the transactions'")
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
    let size = matches
        .value_of("size")
        .unwrap()
        .parse::<usize>()
        .context("The size of transactions must be a non-negative integer")?;
    let rate = matches
        .value_of("rate")
        .unwrap()
        .parse::<u64>()
        .context("The rate of transactions must be a non-negative integer")?;
    let nodes = matches
        .values_of("nodes")
        .unwrap_or_default()
        .into_iter()
        .map(|x| x.parse::<Url>())
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("Invalid url format {target_str}"))?;
    let is_advanced_execution: bool = matches.value_of("execution").unwrap_or("advanced") == "advanced";

    info!("Node address: {target}");

    // NOTE: This log entry is used to compute performance.
    info!("Transactions size: {size} B");

    // NOTE: This log entry is used to compute performance.
    info!("Transactions rate: {rate} tx/s");

    let client = Client {
        target,
        size,
        rate,
        nodes,
        is_advanced_execution,
    };

    // Wait for all nodes to be online and synchronized.
    client.wait().await;

    // Start the benchmark.
    client.send().await.context("Failed to submit transactions")
}

/// TODO - add do_real_transaction as boolean field on client
struct Client {
    target: Url,
    size: usize,
    rate: u64,
    nodes: Vec<Url>,
    is_advanced_execution: bool,
}

impl Client {
    pub async fn send(&self) -> Result<()> {
        const PRECISION: u64 = 20; // Sample precision.
        const BURST_DURATION: u64 = 1000 / PRECISION;

        // The transaction size must be at least 16 bytes to ensure all txs are different.
        if self.size < 9 {
            return Err(anyhow::Error::msg("Transaction size must be at least 9 bytes"));
        }

        // Connect to the mempool.
        let mut transaction_client = TransactionsClient::connect(self.target.as_str().to_owned())
            .await
            .context(format!("failed to connect to {}", self.target))?;

        // Submit all transactions.
        let burst = self.rate / PRECISION;
        let mut counter = 0;
        // select a number from a range that is gaurenteed to be larger than the size of transactions submitted
        // but, not so large that we can exhaust the primary senders balance
        let mut r = rand::thread_rng().gen_range(self.size as u64..(2 * self.size) as u64);
        let interval = interval(Duration::from_millis(BURST_DURATION));
        tokio::pin!(interval);

        // NOTE: This log entry is used to compute performance.
        info!("Start sending transactions");

        'main: loop {
            interval.as_mut().tick().await;
            let now = Instant::now();

            // generate the keypairs
            // note that seed [0; 32] is also fed into the Executor where the BankController state is initiated
            // this means that as long as this keypair is the sender on all transactions there will sufficient balance
            let kp_sender = keys([0; 32]).pop().unwrap();
            let kp_receiver = keys([1; 32]).pop().unwrap();

            // copy into a new variablet o avoid we get lifetime errors in the stream
            let size = self.size;
            let is_advanced_execution = self.is_advanced_execution;

            let stream = tokio_stream::iter(0..burst).map(move |x| {
                let amount = if x == counter % burst {
                    // NOTE: This log entry is used to compute performance.
                    info!("Sending sample transaction {counter}");
                    counter
                } else {
                    r += 1;
                    r
                };

                let signed_tranasction = create_signed_padded_transaction(
                    &kp_sender,
                    &kp_receiver,
                    amount,
                    /* transmission_size */ size,
                    /* is_advanced_execution */ is_advanced_execution,
                );

                TransactionProto {
                    transaction: signed_tranasction.into(),
                }
            });

            if let Err(e) = transaction_client.submit_transaction_stream(stream).await {
                warn!("Failed to send transaction: {e}");
                break 'main;
            }

            if now.elapsed().as_millis() > BURST_DURATION as u128 {
                // NOTE: This log entry is used to compute performance.
                warn!("Transaction rate too high for this client");
            }
            counter += 1;
        }
        Ok(())
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
