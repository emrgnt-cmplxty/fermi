use futures::stream::{Map, Iter};
use gdex_types::{
    account::AccountKeyPair,
    block::BlockDigest,
    order_book::OrderSide,
    proto::{Empty, RelayerClient, RelayerGetLatestBlockInfoRequest, TransactionProto, TransactionsClient},
    transaction::{
        create_asset_creation_transaction, create_orderbook_creation_transaction, create_payment_transaction,
        create_place_limit_order_transaction, SignedTransaction, Transaction
    },
};
use narwhal_crypto::{
    traits::{KeyPair, Signer},
    Hash,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::ops::Range;
use tokio::time::{sleep, Duration};
use tokio_stream::{StreamExt};
use tonic::transport::Channel;
use tracing::{info, warn};
use url::Url;

const BLOCK_INFO_REQUEST: RelayerGetLatestBlockInfoRequest = RelayerGetLatestBlockInfoRequest {};

fn create_signed_payment_transaction(
    kp_sender: &AccountKeyPair,
    kp_receiver: &AccountKeyPair,
    asset_id: u64,
    amount: u64,
    block_digest: BlockDigest,
) -> SignedTransaction {
    let transaction = create_payment_transaction(&kp_sender, &kp_receiver, asset_id, amount, block_digest);
    let signed_digest = kp_sender.sign(&transaction.digest().get_array()[..]);
    let signed_transaction = SignedTransaction::new(kp_sender.public().clone(), transaction.clone(), signed_digest);
    signed_transaction
}

fn create_signed_asset_creation_transaction(
    kp_sender: &AccountKeyPair,
    block_digest: BlockDigest,
) -> SignedTransaction {
    let transaction = create_asset_creation_transaction(&kp_sender, block_digest);
    let signed_digest = kp_sender.sign(&transaction.digest().get_array()[..]);
    let signed_transaction = SignedTransaction::new(kp_sender.public().clone(), transaction.clone(), signed_digest);
    signed_transaction
}

fn create_signed_orderbook_transaction(
    kp_sender: &AccountKeyPair,
    base_asset_id: u64,
    quote_asset_id: u64,
    block_digest: BlockDigest,
) -> SignedTransaction {
    let transaction = create_orderbook_creation_transaction(&kp_sender, base_asset_id, quote_asset_id, block_digest);
    let signed_digest = kp_sender.sign(&transaction.digest().get_array()[..]);
    let signed_transaction = SignedTransaction::new(kp_sender.public().clone(), transaction.clone(), signed_digest);
    signed_transaction
}

fn create_signed_limit_order_transaction(
    kp_sender: &AccountKeyPair,
    base_asset_id: u64,
    quote_asset_id: u64,
    order_side: OrderSide,
    price: u64,
    amount: u64,
    block_digest: BlockDigest,
) -> SignedTransaction {
    let transaction = create_place_limit_order_transaction(
        &kp_sender,
        base_asset_id,
        quote_asset_id,
        order_side,
        price,
        amount,
        block_digest,
    );
    let signed_digest = kp_sender.sign(&transaction.digest().get_array()[..]);
    let signed_transaction = SignedTransaction::new(kp_sender.public().clone(), transaction.clone(), signed_digest);
    signed_transaction
}

pub struct BenchHelper {
    primary_keypair: AccountKeyPair,
    accounts: Vec<AccountKeyPair>,
    validator_client: Option<TransactionsClient<Channel>>,
    relayer_client: Option<RelayerClient<Channel>>,
    base_asset_id: u64,
    quote_asset_id: u64,
}

impl BenchHelper {
    // leave room to fund 1_000 accounts
    const AMOUNT_TO_FUND: u64 = 1_000_000_000_000;

    // constructor
    pub fn new(primary_keypair: AccountKeyPair) -> Self {
        BenchHelper {
            primary_keypair,
            accounts: Vec::new(),
            validator_client: None::<TransactionsClient<Channel>>,
            relayer_client: None::<RelayerClient<Channel>>,
            base_asset_id: 1,
            quote_asset_id: 2,
        }
    }

    // PRIVATE

    async fn get_recent_block_digest(&mut self) -> BlockDigest {
        // fetch recent block digest before starting another round of payments
        let response = self
            .relayer_client
            .as_mut()
            .expect("Relayer client not initialized")
            .get_latest_block_info(BLOCK_INFO_REQUEST.clone())
            .await
            .unwrap()
            .into_inner();

        let block_digest: BlockDigest = if response.successful && response.block_info.is_some() {
            bincode::deserialize(response.block_info.unwrap().digest.as_ref()).unwrap()
        } else {
            warn!("Failed to get latest block digest, returning default");
            BlockDigest::new([0; 32])
        };
        block_digest
    }

    async fn submit_transaction(
        &mut self,
        signed_tranasction: SignedTransaction,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let transaction_proto = TransactionProto {
            transaction: signed_tranasction.serialize().unwrap().into(),
        };

        self.validator_client
            .as_mut()
            .expect("Validator not initialized")
            .submit_transaction(transaction_proto)
            .await
    }

    // PUBLIC

    pub async fn burst_orderbook(&mut self, burst: u64) { 
        info!("bursting client now...");
        let recent_block_hash = self.get_recent_block_digest().await;

        let keypair_copy = self.primary_keypair.copy();
        let base_asset_id = self.base_asset_id;
        let quote_asset_id = self.quote_asset_id;

        let stream = tokio_stream::iter(0..burst).map(move |x| {
            let amount = rand::thread_rng().gen_range(100_000 as u64..5_000_000 as u64);
            let price = rand::thread_rng().gen_range(100_000 as u64..200_000 as u64);

            let order_side = if x%2 == 0 { OrderSide::Bid } else { OrderSide::Ask };
            let signed_transaction = create_signed_limit_order_transaction(
                &keypair_copy.copy(),
                base_asset_id,
                quote_asset_id,
                order_side,
                price,
                amount,
                recent_block_hash,
            );

            TransactionProto {
                transaction: signed_transaction.clone().serialize().unwrap().into(),
            }
        });
        
        if let Err(e) = self.validator_client.as_mut().expect("Failed to load the validator client").submit_transaction_stream(stream).await {
            warn!("Failed to send transaction: {e}");
        }
    }

    /// Load new keypairs into the bench helper
    pub fn generate_accounts(&mut self, seed: [u8; 32], number_to_generate: u64) {
        let mut rng = StdRng::from_seed(seed);
        for _ in 0..number_to_generate {
            self.accounts.push(AccountKeyPair::generate(&mut rng));
        }
    }

    /// Create a new asset in the bench helper
    // TODO - We need a way to get the created asset number to build this stack properly?
    pub async fn create_new_asset(&mut self) {
        let recent_block_hash = self.get_recent_block_digest().await;
        let transaction = create_signed_asset_creation_transaction(&self.primary_keypair, recent_block_hash);
        self.submit_transaction(transaction)
            .await
            .expect("Failed to successfully submit asset creation transaction");
    }

    /// Create a new asset in the bench helper
    // TODO - We need a way to get the created asset number to build this stack properly?
    pub async fn create_orderbook(&mut self) {
        let recent_block_hash = self.get_recent_block_digest().await;
        let transaction = create_signed_orderbook_transaction(
            &self.primary_keypair,
            self.base_asset_id,
            self.quote_asset_id,
            recent_block_hash,
        );
        self.submit_transaction(transaction)
            .await
            .expect("Failed to successfully submit orderbook creation transaction");
    }

    pub async fn fund_accounts(&mut self) {
        // preload transactions to avoid mutability issues
        let mut transactions = Vec::new();
        // TODO - check if this causes failures, if so we need a smarter way to fund accounts as we cannot borrow inside loop below
        let recent_block_hash = self.get_recent_block_digest().await;

        for receiver_keypair in &self.accounts {
            let transaction = create_signed_payment_transaction(
                &self.primary_keypair,
                &receiver_keypair,
                self.base_asset_id,
                Self::AMOUNT_TO_FUND,
                recent_block_hash.clone(),
            );
            // self.submit_transaction(transaction).await.expect("Failed to successfuly fund account {account} with asset 1");
            transactions.push(transaction);
            let transaction = create_signed_payment_transaction(
                &self.primary_keypair,
                &receiver_keypair,
                self.quote_asset_id,
                Self::AMOUNT_TO_FUND,
                recent_block_hash.clone(),
            );
            // self.submit_transaction(transaction).await.expect("Failed to successfuly fund account {account} with asset 2");
            transactions.push(transaction);
        }

        for transaction in transactions {
            info!("Executing a transaction...");

            self.submit_transaction(transaction)
                .await
                .expect("Failed to successfuly submit a funding transaction");
        }
    }

    /// Initialize the bench helper
    pub async fn initialize(
        &mut self,
        validator_url: Url,
        relayer_url: Url,
        seed: [u8; 32],
        accounts_to_generate: u64,
    ) {
        self.validator_client = Some(
            TransactionsClient::connect(validator_url.as_str().to_owned())
                .await
                .unwrap(),
        );
        self.relayer_client = Some(RelayerClient::connect(relayer_url.as_str().to_owned()).await.unwrap());
        self.generate_accounts(seed, accounts_to_generate);

        // log the transaction size to help python client calculate throughput
        // note, any transaction type works here because all enumes have the same size
        let recent_block_hash = self.get_recent_block_digest().await;
        let transaction_size = create_signed_asset_creation_transaction(&self.primary_keypair, recent_block_hash)
            .serialize()
            .unwrap()
            .len();
        info!("Transactions size: {transaction_size} B");
    }

    pub async fn prepare_orderbook(&mut self) {
        info!("Creating first asset...");
        self.create_new_asset().await;
        sleep(Duration::from_millis(2_000)).await;
        info!("Creating second asset...");
        self.create_new_asset().await;
        sleep(Duration::from_millis(2_000)).await;
        info!("Creating orderbook...");
        self.create_orderbook().await;
        info!("Funding accounts...");
        self.fund_accounts().await;
    }
}
