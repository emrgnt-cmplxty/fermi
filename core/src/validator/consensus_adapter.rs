//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use multiaddr::Multiaddr;
use narwhal_config::Committee as ConsensusCommittee;
use narwhal_crypto::KeyPair as ConsensusKeyPair;
use tokio::sync::mpsc::Sender;

/// Submit transactions to the Narwhal consensus.
pub struct ConsensusAdapter {
    /// A network client connecting to the consensus node of this authority.
    pub consensus_client: narwhal_types::TransactionsClient<tonic::transport::Channel>,
    /// The address of consensus
    pub consensus_addresses: Vec<Multiaddr>,
    /// A channel to tell consensus to reconfigure.
    pub tx_reconfigure_consensus: Sender<(ConsensusKeyPair, ConsensusCommittee)>,
}
