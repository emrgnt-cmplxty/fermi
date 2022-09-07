use fastcrypto::traits::KeyPair;
use gdex_types::{
    account::AccountKeyPair,
    block::BlockDigest,
    order_book::OrderSide,
    proto::{Empty, RelayerClient, RelayerGetLatestBlockInfoRequest, TransactionSubmitterClient},
    new_transaction::{NewSignedTransaction, new_create_payment_transaction,
                      new_create_create_orderbook_transaction, new_create_create_asset_transaction,
                      new_create_limit_order_transaction, sign_transaction, ConsensusTransaction},
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use tokio::time::{sleep, Duration};
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tracing::{info, warn};
use url::Url;

const BLOCK_INFO_REQUEST: RelayerGetLatestBlockInfoRequest = RelayerGetLatestBlockInfoRequest {};
const MATCH_FREQUENCY: u64 = 100;

fn create_signed_payment_transaction(
    kp_sender: &AccountKeyPair,
    kp_receiver: &AccountKeyPair,
    asset_id: u64,
    amount: u64,
    block_digest: BlockDigest,
) -> NewSignedTransaction {
    let gas: u64 = 1000;
    let new_transaction = new_create_payment_transaction(kp_sender.public().clone(), kp_receiver.public(), asset_id, amount, gas, block_digest);
    let new_signed_transaction = match sign_transaction(kp_sender, new_transaction) {
        Ok(t) => t,
        _ => panic!("Error signing transaction"),
    };
    new_signed_transaction
}

fn create_signed_asset_creation_transaction(
    kp_sender: &AccountKeyPair,
    block_digest: BlockDigest,
    dummy: u8,
) -> NewSignedTransaction {
    let gas: u64 = 1000;
    let new_transaction = new_create_create_asset_transaction(kp_sender.public().clone(), dummy as u64, gas, block_digest);
    let new_signed_transaction = match sign_transaction(kp_sender, new_transaction) {
        Ok(t) => t,
        _ => panic!("Error signing transaction"),
    };
    new_signed_transaction
}

fn create_signed_orderbook_transaction(
    kp_sender: &AccountKeyPair,
    base_asset_id: u64,
    quote_asset_id: u64,
    block_digest: BlockDigest,
) -> NewSignedTransaction {
    let gas: u64 = 1000;
    let new_transaction = new_create_create_orderbook_transaction(kp_sender.public().clone(), base_asset_id, quote_asset_id, gas, block_digest);
    let new_signed_transaction = match sign_transaction(kp_sender, new_transaction) {
        Ok(t) => t,
        _ => panic!("Error signing transaction"),
    };
    new_signed_transaction
}

fn create_signed_limit_order_transaction(
    kp_sender: &AccountKeyPair,
    base_asset_id: u64,
    quote_asset_id: u64,
    order_side: OrderSide,
    price: u64,
    amount: u64,
    block_digest: BlockDigest,
) -> NewSignedTransaction {
    let local_timestamp: u64 = 16000000;
    let gas: u64 = 1000;
    let new_transaction = new_create_limit_order_transaction(kp_sender.public().clone(), base_asset_id, quote_asset_id, order_side as u64, price, amount, local_timestamp, gas, block_digest);
    let new_signed_transaction = match sign_transaction(kp_sender, new_transaction) {
        Ok(t) => t,
        _ => panic!("Error signing transaction"),
    };
    new_signed_transaction
}

pub struct BenchHelper {
    primary_keypair: AccountKeyPair,
    accounts: Vec<AccountKeyPair>,
    validator_client: Option<TransactionSubmitterClient<Channel>>,
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
            validator_client: None::<TransactionSubmitterClient<Channel>>,
            relayer_client: None::<RelayerClient<Channel>>,
            // TODO - avoid hard coding by directly calculating created assets...
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
        new_signed_tranasction: NewSignedTransaction,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        self.validator_client
            .as_mut()
            .expect("Validator not initialized")
            .submit_transaction(new_signed_tranasction)
            .await
    }

    // PUBLIC

    pub async fn burst_orderbook(&mut self, burst: u64) {
        info!("bursting client now...");
        let recent_block_hash = self.get_recent_block_digest().await;

        // prepare copies of self variables before moving into closure
        let keypair_copy = self.primary_keypair.copy();
        let base_asset_id = self.base_asset_id;
        let quote_asset_id = self.quote_asset_id;

        let stream = tokio_stream::iter(0..burst).map(move |x| {
            let mut amount = rand::thread_rng().gen_range(1_u64..100_u64);

            let (order_side, mut price) = if x % 2 == 0 {
                (OrderSide::Bid, rand::thread_rng().gen_range(101_u64..200_u64))
            } else {
                (OrderSide::Ask, rand::thread_rng().gen_range(1_u64..100_u64))
            };

            // cross the spread for one unit of quanitty at MATCH_FREQUENCY
            if x % MATCH_FREQUENCY == 0 && x > 0 {
                price = 100;
                amount = 1;
            }

            let signed_transaction = create_signed_limit_order_transaction(
                &keypair_copy.copy(),
                base_asset_id,
                quote_asset_id,
                order_side,
                price,
                amount,
                recent_block_hash,
            );

            signed_transaction            
        });

        if let Err(e) = self
            .validator_client
            .as_mut()
            .expect("Failed to load the validator client")
            .submit_transaction_stream(stream)
            .await
        {
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
    // TODO - Fetch created asset number to build this stack properly.
    pub async fn create_new_asset(&mut self, dummy: u8) {
        let recent_block_hash = self.get_recent_block_digest().await;
        let transaction = create_signed_asset_creation_transaction(&self.primary_keypair, recent_block_hash, dummy);
        self.submit_transaction(transaction)
            .await
            .expect("Failed to successfully submit asset creation transaction");
    }

    /// Create a new asset in the bench helper
    // TODO - Fetch created asset number to build this stack properly.
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
            // initialize the account by sending the primary asset
            let transaction = create_signed_payment_transaction(
                &self.primary_keypair,
                receiver_keypair,
                0,
                1_000_000,
                recent_block_hash,
            );
            transactions.push(transaction);
            let transaction = create_signed_payment_transaction(
                &self.primary_keypair,
                receiver_keypair,
                self.base_asset_id,
                Self::AMOUNT_TO_FUND,
                recent_block_hash,
            );
            transactions.push(transaction);
            let transaction = create_signed_payment_transaction(
                &self.primary_keypair,
                receiver_keypair,
                self.quote_asset_id,
                Self::AMOUNT_TO_FUND,
                recent_block_hash,
            );
            transactions.push(transaction);
        }

        for transaction in transactions {
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
            TransactionSubmitterClient::connect(validator_url.as_str().to_owned())
                .await
                .unwrap(),
        );
        self.relayer_client = Some(RelayerClient::connect(relayer_url.as_str().to_owned()).await.unwrap());
        self.generate_accounts(seed, accounts_to_generate);

        // log the transaction size to help python client calculate throughput
        // note, any transaction type works here because all enumes have the same size
        let recent_block_hash = self.get_recent_block_digest().await;

        let signed_transaction = create_signed_asset_creation_transaction(&self.primary_keypair, recent_block_hash, 0);
        
        let serialized_consensus_transaction = match ConsensusTransaction::new(&signed_transaction).serialize() {
            Ok(t) => t,
            _ => panic!("Error serializing transaction"),
        };

        let transaction_size = serialized_consensus_transaction.len();
        
        info!("Transactions size: {transaction_size} B");
    }

    pub async fn prepare_orderbook(&mut self) {
        info!("Creating first asset...");
        self.create_new_asset(0).await;
        info!("Creating second asset...");
        self.create_new_asset(1).await;
        sleep(Duration::from_millis(2_000)).await;
        info!("Creating orderbook...");
        self.create_orderbook().await;
        info!("Funding accounts...");
        self.fund_accounts().await;
    }
}
