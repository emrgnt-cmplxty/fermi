// fermi
use crate::{
    block::{Block, BlockInfo, BlockNumber},
    order_book::OrderbookDepth,
};
// mysten
use mysten_store::{
    reopen, rocks,
    rocks::{open_cf, DBMap, TypedStoreError},
    traits::Map,
};
// external
use eyre::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    cmp::Eq,
    collections::{HashMap, VecDeque},
    hash::Hash,
};
use tokio::sync::{
    mpsc::{channel, Sender},
    oneshot,
};

pub type StoreError = rocks::TypedStoreError;
type StoreResult<T> = Result<T, StoreError>;

pub enum StoreCommand<Key, Value> {
    Write(Key, Value),
    WriteAll(Vec<(Key, Value)>, oneshot::Sender<StoreResult<()>>),
    Delete(Key),
    DeleteAll(Vec<Key>, oneshot::Sender<StoreResult<()>>),
    Read(Key, oneshot::Sender<StoreResult<Option<Value>>>),
    ReadAll(Vec<Key>, oneshot::Sender<StoreResult<Vec<Option<Value>>>>),
    NotifyRead(Key, oneshot::Sender<StoreResult<Option<Value>>>),
    Iter(
        #[allow(clippy::type_complexity)] Option<Box<dyn Fn(&(Key, Value)) -> bool + Send>>,
        oneshot::Sender<HashMap<Key, Value>>,
    ),
}

#[derive(Clone)]
pub struct Store<K, V> {
    channel: Sender<StoreCommand<K, V>>,
}

impl<Key, Value> Store<Key, Value>
where
    Key: Hash + Eq + Serialize + DeserializeOwned + Send + 'static,
    Value: Serialize + DeserializeOwned + Send + Clone + 'static,
{
    pub fn new(keyed_db: rocks::DBMap<Key, Value>) -> Self {
        let mut obligations = HashMap::<Key, VecDeque<oneshot::Sender<_>>>::new();
        let (tx, mut rx) = channel(100);
        tokio::spawn(async move {
            while let Some(command) = rx.recv().await {
                match command {
                    StoreCommand::Write(key, value) => {
                        let _ = keyed_db.insert(&key, &value);
                        if let Some(mut senders) = obligations.remove(&key) {
                            while let Some(s) = senders.pop_front() {
                                let _ = s.send(Ok(Some(value.clone())));
                            }
                        }
                    }
                    StoreCommand::WriteAll(key_values, sender) => {
                        let response = keyed_db.multi_insert(key_values.iter().map(|(k, v)| (k, v)));

                        if response.is_ok() {
                            for (key, _) in key_values {
                                if let Some(mut senders) = obligations.remove(&key) {
                                    while let Some(s) = senders.pop_front() {
                                        let _ = s.send(Ok(None));
                                    }
                                }
                            }
                        }
                        let _ = sender.send(response);
                    }
                    StoreCommand::Delete(key) => {
                        let _ = keyed_db.remove(&key);
                        if let Some(mut senders) = obligations.remove(&key) {
                            while let Some(s) = senders.pop_front() {
                                let _ = s.send(Ok(None));
                            }
                        }
                    }
                    StoreCommand::DeleteAll(keys, sender) => {
                        let response = keyed_db.multi_remove(keys.iter());
                        // notify the obligations only when the delete was successful
                        if response.is_ok() {
                            for key in keys {
                                if let Some(mut senders) = obligations.remove(&key) {
                                    while let Some(s) = senders.pop_front() {
                                        let _ = s.send(Ok(None));
                                    }
                                }
                            }
                        }
                        let _ = sender.send(response);
                    }
                    StoreCommand::Read(key, sender) => {
                        let response = keyed_db.get(&key);
                        let _ = sender.send(response);
                    }
                    StoreCommand::ReadAll(keys, sender) => {
                        let response = keyed_db.multi_get(keys.as_slice());
                        let _ = sender.send(response);
                    }
                    StoreCommand::NotifyRead(key, sender) => {
                        let response = keyed_db.get(&key);
                        if let Ok(Some(_)) = response {
                            let _ = sender.send(response);
                        } else {
                            obligations.entry(key).or_insert_with(VecDeque::new).push_back(sender)
                        }
                    }
                    StoreCommand::Iter(predicate, sender) => {
                        let response = if let Some(func) = predicate {
                            keyed_db.iter().filter(func).collect()
                        } else {
                            // Beware, we may overload the memory with a large table!
                            keyed_db.iter().collect()
                        };

                        let _ = sender.send(response);
                    }
                }
            }
        });
        Self { channel: tx }
    }
}

impl<Key, Value> Store<Key, Value>
where
    Key: Serialize + DeserializeOwned + Send,
    Value: Serialize + DeserializeOwned + Send,
{
    pub async fn write(&self, key: Key, value: Value) {
        if let Err(e) = self.channel.send(StoreCommand::Write(key, value)).await {
            panic!("Failed to send Write command to store: {e}");
        }
    }

    pub fn try_write(&self, key: Key, value: Value) {
        if let Err(e) = self.channel.try_send(StoreCommand::Write(key, value)) {
            panic!("Failed to send Write command to store: {e}");
        }
    }

    /// Atomically writes all the key-value pairs in storage.
    /// If the operation is successful, then the result will be a non
    /// error empty result. Otherwise the error is returned.
    pub async fn write_all(&self, key_value_pairs: impl IntoIterator<Item = (Key, Value)>) -> StoreResult<()> {
        let (sender, receiver) = oneshot::channel();
        if let Err(e) = self
            .channel
            .send(StoreCommand::WriteAll(key_value_pairs.into_iter().collect(), sender))
            .await
        {
            panic!("Failed to send WriteAll command to store: {e}");
        }
        receiver
            .await
            .expect("Failed to receive reply to WriteAll command from store")
    }

    pub async fn remove(&self, key: Key) {
        if let Err(e) = self.channel.send(StoreCommand::Delete(key)).await {
            panic!("Failed to send Delete command to store: {e}");
        }
    }

    /// Atomically removes all the data referenced by the provided keys.
    /// If the operation is successful, then the result will be a non
    /// error empty result. Otherwise the error is returned.
    pub async fn remove_all(&self, keys: impl IntoIterator<Item = Key>) -> StoreResult<()> {
        let (sender, receiver) = oneshot::channel();
        if let Err(e) = self
            .channel
            .send(StoreCommand::DeleteAll(keys.into_iter().collect(), sender))
            .await
        {
            panic!("Failed to send DeleteAll command to store: {e}");
        }
        receiver
            .await
            .expect("Failed to receive reply to RemoveAll command from store")
    }

    pub async fn read(&self, key: Key) -> StoreResult<Option<Value>> {
        let (sender, receiver) = oneshot::channel();
        if let Err(e) = self.channel.send(StoreCommand::Read(key, sender)).await {
            panic!("Failed to send Read command to store: {e}");
        }
        receiver
            .await
            .expect("Failed to receive reply to Read command from store")
    }

    /// Fetches all the values for the provided keys.
    pub async fn read_all(&self, keys: impl IntoIterator<Item = Key>) -> StoreResult<Vec<Option<Value>>> {
        let (sender, receiver) = oneshot::channel();
        if let Err(e) = self
            .channel
            .send(StoreCommand::ReadAll(keys.into_iter().collect(), sender))
            .await
        {
            panic!("Failed to send ReadAll command to store: {e}");
        }
        receiver
            .await
            .expect("Failed to receive reply to ReadAll command from store")
    }

    pub async fn notify_read(&self, key: Key) -> StoreResult<Option<Value>> {
        let (sender, receiver) = oneshot::channel();
        if let Err(e) = self.channel.send(StoreCommand::NotifyRead(key, sender)).await {
            panic!("Failed to send NotifyRead command to store: {e}");
        }
        receiver
            .await
            .expect("Failed to receive reply to NotifyRead command from store")
    }

    #[allow(clippy::type_complexity)]
    pub async fn iter(&self, predicate: Option<Box<dyn Fn(&(Key, Value)) -> bool + Send>>) -> HashMap<Key, Value> {
        let (sender, receiver) = oneshot::channel();
        if let Err(e) = self.channel.send(StoreCommand::Iter(predicate, sender)).await {
            panic!("Failed to send Iter command to store: {e}");
        }
        receiver
            .await
            .expect("Failed to receive reply to Iter command from store")
    }
}

pub struct CriticalPathStore {
    // last block info
    pub last_block_info: Result<Option<BlockInfo>, TypedStoreError>,
    // stores
    pub last_block_info_store: Store<BlockNumber, BlockInfo>,
    pub block_store: Store<BlockNumber, Block>,
    pub block_info_store: Store<BlockNumber, BlockInfo>,
}

impl CriticalPathStore {
    const BLOCKS_CF: &'static str = "blocks";
    const BLOCK_INFO_CF: &'static str = "block_info";
    const LAST_BLOCK_CF: &'static str = "last_block";
    pub fn reopen<Path: AsRef<std::path::Path>>(store_path: Path) -> Self {
        let rocksdb = open_cf(
            store_path,
            None,
            &[Self::BLOCKS_CF, Self::BLOCK_INFO_CF, Self::LAST_BLOCK_CF],
        )
        .expect("Cannot open database");
        let (block_map, block_info_map, last_block_map) = reopen!(&rocksdb,
            Self::BLOCKS_CF;<BlockNumber, Block>,
            Self::BLOCK_INFO_CF;<BlockNumber, BlockInfo>,
            Self::LAST_BLOCK_CF;<u64, BlockInfo>
        );

        let last_block_info = last_block_map.get(&0_u64);

        let last_block_info_store = Store::new(last_block_map);
        let block_store = Store::new(block_map);
        let block_info_store = Store::new(block_info_map);

        Self {
            last_block_info,
            last_block_info_store,
            block_store,
            block_info_store,
        }
    }
}

// TODO - Find intelligent way to make the JSON RPC Store modular across controllers
// For instance, we could mirror the transaction ingress for data queries to
// Create a protobuf style workflow where each controller implements their respective
// Data fetching logic inside of a protobuf sending and receiving workstream
pub struct RPCStoreHandle {
    pub rpc_store: RPCStore,
}
// catchup state
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CatchupState {
    pub state: Vec<Vec<u8>>,
}

pub struct RPCStore {
    pub latest_orderbook_depth_store: Store<String, OrderbookDepth>,
    // catchup store
    pub catchup_state_store: Store<BlockNumber, CatchupState>,
}

impl RPCStore {
    const LAST_ORDERBOOK_DEPTH_CF: &'static str = "last_orderbook_depth";
    const CATCHUP_STATE_CF: &'static str = "catchup_state";

    pub fn reopen<Path: AsRef<std::path::Path>>(store_path: Path) -> Self {
        let rocksdb = open_cf(
            store_path,
            None,
            &[Self::LAST_ORDERBOOK_DEPTH_CF, Self::CATCHUP_STATE_CF],
        )
        .expect("Cannot open database");
        let (orderbook_depth_map, catchup_state_map) = reopen!(&rocksdb,
            Self::LAST_ORDERBOOK_DEPTH_CF;<String, OrderbookDepth>,
            Self::CATCHUP_STATE_CF;<BlockNumber, CatchupState>
        );
        let latest_orderbook_depth_store = Store::new(orderbook_depth_map);
        let catchup_state_store = Store::new(catchup_state_map);

        Self {
            latest_orderbook_depth_store,
            catchup_state_store,
        }
    }
}
