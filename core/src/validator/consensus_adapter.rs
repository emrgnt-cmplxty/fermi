//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use multiaddr::Multiaddr;
use narwhal_types::TransactionsClient;

/// Submit transactions to the Narwhal consensus.
pub struct ConsensusAdapter {
    /// A network client connecting to the consensus node of this authority.
    pub consensus_client: TransactionsClient<tonic::transport::Channel>,
    /// The address of consensus
    pub consensus_address: Multiaddr,
}
