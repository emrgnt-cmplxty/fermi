// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use fastcrypto::encoding::{Encoding, Hex};
use std::fmt::{Debug, Display, Formatter};
use std::slice::Iter;

use crate::base_types::ExecutionDigests;
use crate::committee::{EpochId, StakeUnit};
use crate::crypto::{AuthoritySignInfo, AuthoritySignInfoTrait, AuthorityWeakQuorumSignInfo};
use crate::error::SuiResult;
use crate::gas::GasCostSummary;
use crate::waypoint::Waypoint;
use crate::{
    base_types::AuthorityName,
    committee::Committee,
    crypto::{sha3_hash, AuthoritySignature, VerificationObligation},
    error::SuiError,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub type CheckpointSequenceNumber = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointRequest {
    // Type of checkpoint request
    pub request_type: CheckpointRequestType,
    // A flag, if true also return the contents of the
    // checkpoint besides the meta-data.
    pub detail: bool,
}

impl CheckpointRequest {
    pub fn authenticated(seq: Option<CheckpointSequenceNumber>, detail: bool) -> CheckpointRequest {
        CheckpointRequest {
            request_type: CheckpointRequestType::AuthenticatedCheckpoint(seq),
            detail,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CheckpointRequestType {
    /// Request a stored authenticated checkpoint.
    /// if a sequence number is specified, return the checkpoint with that sequence number;
    /// otherwise if None returns the latest authenticated checkpoint stored.
    AuthenticatedCheckpoint(Option<CheckpointSequenceNumber>),
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CheckpointResponse {
    AuthenticatedCheckpoint {
        checkpoint: Option<AuthenticatedCheckpoint>,
        contents: Option<CheckpointContents>,
    },
}

// TODO: Rename to AuthenticatedCheckpointSummary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthenticatedCheckpoint {
    // The checkpoint with just a single authority
    // signature.
    Signed(SignedCheckpointSummary),
    // The checkpoint with a quorum of signatures.
    Certified(CertifiedCheckpointSummary),
}

impl AuthenticatedCheckpoint {
    pub fn summary(&self) -> &CheckpointSummary {
        match self {
            Self::Signed(s) => &s.summary,
            Self::Certified(c) => &c.summary,
        }
    }

    pub fn verify(&self, committee: &Committee, detail: Option<&CheckpointContents>) -> SuiResult {
        match self {
            Self::Signed(s) => s.verify(committee, detail),
            Self::Certified(c) => c.verify(committee, detail),
        }
    }

    pub fn sequence_number(&self) -> CheckpointSequenceNumber {
        match self {
            Self::Signed(s) => s.summary.sequence_number,
            Self::Certified(c) => c.summary.sequence_number,
        }
    }

    pub fn epoch(&self) -> EpochId {
        match self {
            Self::Signed(s) => s.summary.epoch,
            Self::Certified(c) => c.summary.epoch,
        }
    }
}

pub type CheckpointDigest = [u8; 32];
pub type CheckpointContentsDigest = [u8; 32];

// The constituent parts of checkpoints, signed and certified

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointSummary {
    pub epoch: EpochId,
    pub sequence_number: CheckpointSequenceNumber,
    pub content_digest: CheckpointContentsDigest,
    pub previous_digest: Option<CheckpointDigest>,
    /// The total gas costs of all transactions included in this checkpoint.
    pub gas_cost_summary: GasCostSummary,
    /// If this checkpoint is the last checkpoint of the epoch, we also include the committee
    /// of the next epoch. This allows anyone receiving this checkpoint know that the epoch
    /// will change after this checkpoint, as well as what the new committee is.
    /// The committee is stored as a vector of validator pub key and stake pairs. The vector
    /// should be sorted based on the Committee data structure.
    /// TODO: If desired, we could also commit to the previous last checkpoint cert so that
    /// they form a hash chain.
    pub next_epoch_committee: Option<Vec<(AuthorityName, StakeUnit)>>,
}

impl CheckpointSummary {
    pub fn new(
        epoch: EpochId,
        sequence_number: CheckpointSequenceNumber,
        transactions: &CheckpointContents,
        previous_digest: Option<CheckpointDigest>,
        gas_cost_summary: GasCostSummary,
        next_epoch_committee: Option<Committee>,
    ) -> CheckpointSummary {
        let mut waypoint = Box::new(Waypoint::default());
        transactions.iter().for_each(|tx| {
            waypoint.insert(tx);
        });

        let content_digest = transactions.digest();

        Self {
            epoch,
            sequence_number,
            content_digest,
            previous_digest,
            gas_cost_summary,
            next_epoch_committee: next_epoch_committee.map(|c| c.voting_rights),
        }
    }

    pub fn sequence_number(&self) -> &CheckpointSequenceNumber {
        &self.sequence_number
    }

    pub fn digest(&self) -> CheckpointDigest {
        sha3_hash(self)
    }
}

impl Display for CheckpointSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CheckpointSummary {{ epoch: {:?}, seq: {:?}, content_digest: {},
            gas_cost_summary: {:?}}}",
            self.epoch,
            self.sequence_number,
            Hex::encode(self.content_digest),
            self.gas_cost_summary,
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointSummaryEnvelope<S> {
    pub summary: CheckpointSummary,
    pub auth_signature: S,
}

impl<S> CheckpointSummaryEnvelope<S> {
    pub fn summary(&self) -> &CheckpointSummary {
        &self.summary
    }

    pub fn digest(&self) -> CheckpointDigest {
        self.summary.digest()
    }

    pub fn epoch(&self) -> EpochId {
        self.summary.epoch
    }

    pub fn sequence_number(&self) -> CheckpointSequenceNumber {
        self.summary.sequence_number
    }

    pub fn content_digest(&self) -> CheckpointContentsDigest {
        self.summary.content_digest
    }

    pub fn previous_digest(&self) -> Option<CheckpointDigest> {
        self.summary.previous_digest
    }

    pub fn next_epoch_committee(&self) -> Option<&[(AuthorityName, StakeUnit)]> {
        self.summary.next_epoch_committee.as_deref()
    }
}

impl<S: Debug> Display for CheckpointSummaryEnvelope<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.summary)?;
        writeln!(f, "Signature: {:?}", self.auth_signature)?;
        Ok(())
    }
}

pub type SignedCheckpointSummary = CheckpointSummaryEnvelope<AuthoritySignInfo>;

impl SignedCheckpointSummary {
    /// Create a new signed checkpoint proposal for this authority
    pub fn new(
        epoch: EpochId,
        sequence_number: CheckpointSequenceNumber,
        authority: AuthorityName,
        signer: &dyn signature::Signer<AuthoritySignature>,
        transactions: &CheckpointContents,
        previous_digest: Option<CheckpointDigest>,
        gas_cost_summary: GasCostSummary,
        next_epoch_committee: Option<Committee>,
    ) -> SignedCheckpointSummary {
        let checkpoint = CheckpointSummary::new(
            epoch,
            sequence_number,
            transactions,
            previous_digest,
            gas_cost_summary,
            next_epoch_committee,
        );
        SignedCheckpointSummary::new_from_summary(checkpoint, authority, signer)
    }

    pub fn new_from_summary(
        checkpoint: CheckpointSummary,
        authority: AuthorityName,
        signer: &dyn signature::Signer<AuthoritySignature>,
    ) -> SignedCheckpointSummary {
        let epoch = checkpoint.epoch;
        let auth_signature = AuthoritySignInfo::new(epoch, &checkpoint, authority, signer);
        SignedCheckpointSummary {
            summary: checkpoint,
            auth_signature,
        }
    }

    pub fn authority(&self) -> &AuthorityName {
        &self.auth_signature.authority
    }

    /// Checks that the signature on the digest is correct, and verify the contents as well if
    /// provided.
    pub fn verify(
        &self,
        committee: &Committee,
        contents: Option<&CheckpointContents>,
    ) -> Result<(), SuiError> {
        fp_ensure!(
            self.summary.epoch == committee.epoch,
            SuiError::from("Epoch in the summary doesn't match with the signature")
        );

        self.auth_signature.verify(&self.summary, committee)?;

        if let Some(contents) = contents {
            let content_digest = contents.digest();
            fp_ensure!(
                content_digest == self.summary.content_digest,
                SuiError::GenericAuthorityError{error:format!("Checkpoint contents digest mismatch: summary={:?}, received content digest {:?}, received {} transactions", self.summary, content_digest, contents.size())}
            );
        }

        Ok(())
    }
}

// Checkpoints are signed by an authority and 2f+1 form a
// certificate that others can use to catch up. The actual
// content of the digest must at the very least commit to
// the set of transactions contained in the certificate but
// we might extend this to contain roots of merkle trees,
// or other authenticated data structures to support light
// clients and more efficient sync protocols.

pub type CertifiedCheckpointSummary = CheckpointSummaryEnvelope<AuthorityWeakQuorumSignInfo>;

impl CertifiedCheckpointSummary {
    /// Aggregate many checkpoint signatures to form a checkpoint certificate.
    pub fn aggregate(
        signed_checkpoints: Vec<SignedCheckpointSummary>,
        committee: &Committee,
    ) -> Result<CertifiedCheckpointSummary, SuiError> {
        fp_ensure!(
            !signed_checkpoints.is_empty(),
            SuiError::from("Need at least one signed checkpoint to aggregate")
        );
        fp_ensure!(
            signed_checkpoints
                .iter()
                .all(|c| c.summary.epoch == committee.epoch),
            SuiError::from("SignedCheckpoint is from different epoch as committee")
        );

        let certified_checkpoint = CertifiedCheckpointSummary {
            summary: signed_checkpoints[0].summary.clone(),
            auth_signature: AuthorityWeakQuorumSignInfo::new_from_auth_sign_infos(
                signed_checkpoints
                    .into_iter()
                    .map(|v| v.auth_signature)
                    .collect(),
                committee,
            )?,
        };

        certified_checkpoint.verify(committee, None)?;
        Ok(certified_checkpoint)
    }

    pub fn signatory_authorities<'a>(
        &'a self,
        committee: &'a Committee,
    ) -> impl Iterator<Item = SuiResult<&AuthorityName>> {
        self.auth_signature.authorities(committee)
    }

    /// Check that a certificate is valid, and signed by a quorum of authorities
    pub fn verify(
        &self,
        committee: &Committee,
        contents: Option<&CheckpointContents>,
    ) -> Result<(), SuiError> {
        fp_ensure!(
            self.summary.epoch == committee.epoch,
            SuiError::from("Epoch in the summary doesn't match with the committee")
        );
        let mut obligation = VerificationObligation::default();
        let idx = obligation.add_message(&self.summary, self.auth_signature.epoch);
        self.auth_signature
            .add_to_verification_obligation(committee, &mut obligation, idx)?;

        obligation.verify_all()?;

        if let Some(contents) = contents {
            let content_digest = contents.digest();
            fp_ensure!(
                content_digest == self.summary.content_digest,
                SuiError::GenericAuthorityError{error:format!("Checkpoint contents digest mismatch: summary={:?}, content digest = {:?}, transactions {}", self.summary, content_digest, contents.size())}
            );
        }

        Ok(())
    }
}

/// A type-safe way to ensure that a checkpoint has been verified
#[derive(Clone, Debug)]
pub struct VerifiedCheckpoint(CertifiedCheckpointSummary);

// The only acceptible way to construct this type is via explicitly verifying it
static_assertions::assert_not_impl_any!(VerifiedCheckpoint: Serialize, DeserializeOwned);

impl VerifiedCheckpoint {
    pub fn new(
        checkpoint: CertifiedCheckpointSummary,
        committee: &Committee,
    ) -> Result<Self, (CertifiedCheckpointSummary, SuiError)> {
        match checkpoint.verify(committee, None) {
            Ok(()) => Ok(Self(checkpoint)),
            Err(err) => Err((checkpoint, err)),
        }
    }

    pub fn inner(&self) -> &CertifiedCheckpointSummary {
        &self.0
    }

    pub fn into_inner(self) -> CertifiedCheckpointSummary {
        self.0
    }
}

impl std::ops::Deref for VerifiedCheckpoint {
    type Target = CertifiedCheckpointSummary;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// This is a message validators publish to consensus in order to sign checkpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointSignatureMessage {
    pub summary: SignedCheckpointSummary,
}

/// CheckpointContents are the transactions included in an upcoming checkpoint.
/// They must have already been causally ordered. Since the causal order algorithm
/// is the same among validators, we expect all honest validators to come up with
/// the same order for each checkpoint content.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointContents {
    transactions: Vec<ExecutionDigests>,
}

impl CheckpointSignatureMessage {
    pub fn verify(&self, committee: &Committee) -> SuiResult {
        self.summary.verify(committee, None)
    }
}

impl CheckpointContents {
    pub fn new_with_causally_ordered_transactions<T>(contents: T) -> Self
    where
        T: Iterator<Item = ExecutionDigests>,
    {
        Self {
            transactions: contents.collect(),
        }
    }

    pub fn iter(&self) -> Iter<'_, ExecutionDigests> {
        self.transactions.iter()
    }

    pub fn into_inner(self) -> Vec<ExecutionDigests> {
        self.transactions
    }

    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    pub fn digest(&self) -> CheckpointContentsDigest {
        sha3_hash(self)
    }
}

#[cfg(test)]
mod tests {
    use fastcrypto::traits::KeyPair;
    use rand::prelude::StdRng;
    use rand::SeedableRng;

    use super::*;
    use crate::utils::make_committee_key;

    // TODO use the file name as a seed
    const RNG_SEED: [u8; 32] = [
        21, 23, 199, 200, 234, 250, 252, 178, 94, 15, 202, 178, 62, 186, 88, 137, 233, 192, 130,
        157, 179, 179, 65, 9, 31, 249, 221, 123, 225, 112, 199, 247,
    ];

    #[test]
    fn test_signed_checkpoint() {
        let mut rng = StdRng::from_seed(RNG_SEED);
        let (keys, committee) = make_committee_key(&mut rng);
        let (_, committee2) = make_committee_key(&mut rng);

        let set = CheckpointContents::new_with_causally_ordered_transactions(
            [ExecutionDigests::random()].into_iter(),
        );

        // TODO: duplicated in a test below.
        let signed_checkpoints: Vec<_> = keys
            .iter()
            .map(|k| {
                let name = k.public().into();

                SignedCheckpointSummary::new(
                    committee.epoch,
                    1,
                    name,
                    k,
                    &set,
                    None,
                    GasCostSummary::default(),
                    None,
                )
            })
            .collect();

        signed_checkpoints
            .iter()
            .for_each(|c| c.verify(&committee, None).expect("signature ok"));

        // fails when not signed by member of committee
        signed_checkpoints
            .iter()
            .for_each(|c| assert!(c.verify(&committee2, None).is_err()));
    }

    #[test]
    fn test_certified_checkpoint() {
        let mut rng = StdRng::from_seed(RNG_SEED);
        let (keys, committee) = make_committee_key(&mut rng);

        let set = CheckpointContents::new_with_causally_ordered_transactions(
            [ExecutionDigests::random()].into_iter(),
        );

        let signed_checkpoints: Vec<_> = keys
            .iter()
            .map(|k| {
                let name = k.public().into();

                SignedCheckpointSummary::new(
                    committee.epoch,
                    1,
                    name,
                    k,
                    &set,
                    None,
                    GasCostSummary::default(),
                    None,
                )
            })
            .collect();

        let checkpoint_cert = CertifiedCheckpointSummary::aggregate(signed_checkpoints, &committee)
            .expect("Cert is OK");

        // Signature is correct on proposal, and with same transactions
        assert!(checkpoint_cert.verify(&committee, Some(&set)).is_ok());

        // Make a bad proposal
        let signed_checkpoints: Vec<_> = keys
            .iter()
            .map(|k| {
                let name = k.public().into();
                let set = CheckpointContents::new_with_causally_ordered_transactions(
                    [ExecutionDigests::random()].into_iter(),
                );

                SignedCheckpointSummary::new(
                    committee.epoch,
                    1,
                    name,
                    k,
                    &set,
                    None,
                    GasCostSummary::default(),
                    None,
                )
            })
            .collect();

        assert!(CertifiedCheckpointSummary::aggregate(signed_checkpoints, &committee).is_err());
    }
}
