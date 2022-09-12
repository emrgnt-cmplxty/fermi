//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0

use crate::{
    block::{Block, BlockInfo, BlockNumber},
    order_book::OrderbookDepth,
};
use mysten_store::{
    reopen,
    rocks::{open_cf, DBMap, TypedStoreError},
    Map, Store,
};

pub struct ProcessBlockStore {
    // last block info
    pub last_block_info: Result<Option<BlockInfo>, TypedStoreError>,
    // stores
    pub block_store: Store<BlockNumber, Block>,
    pub block_info_store: Store<BlockNumber, BlockInfo>,
    pub last_block_info_store: Store<u64, BlockInfo>,
    pub latest_orderbook_depth_store: Store<String, OrderbookDepth>,
}

impl ProcessBlockStore {
    const BLOCKS_CF: &'static str = "blocks";
    const BLOCK_INFO_CF: &'static str = "block_info";
    const LAST_BLOCK_CF: &'static str = "last_block";
    const LAST_ORDERBOOK_DEPTH_CF: &'static str = "last_orderbook_depth";

    pub fn reopen<Path: AsRef<std::path::Path>>(store_path: Path) -> Self {
        let rocksdb = open_cf(
            store_path,
            None,
            &[
                Self::BLOCKS_CF,
                Self::BLOCK_INFO_CF,
                Self::LAST_BLOCK_CF,
                Self::LAST_ORDERBOOK_DEPTH_CF,
            ],
        )
        .expect("Cannot open database");
        let (block_map, block_info_map, last_block_map, orderbook_depth_map) = reopen!(&rocksdb,
            Self::BLOCKS_CF;<BlockNumber, Block>,
            Self::BLOCK_INFO_CF;<BlockNumber, BlockInfo>,
            Self::LAST_BLOCK_CF;<u64, BlockInfo>,
            Self::LAST_ORDERBOOK_DEPTH_CF;<String, OrderbookDepth>
        );

        let last_block_info = last_block_map.get(&0_u64);

        let block_store = Store::new(block_map);
        let block_info_store = Store::new(block_info_map);
        let last_block_info_store = Store::new(last_block_map);
        let latest_orderbook_depth_store = Store::new(orderbook_depth_map);

        Self {
            last_block_info,
            block_store,
            block_info_store,
            last_block_info_store,
            latest_orderbook_depth_store,
        }
    }
}