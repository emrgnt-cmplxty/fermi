// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    base_types::*,
    committee::{EpochId, StakeUnit},
    messages::ExecutionFailureStatus,
    object::Owner,
};
use move_binary_format::errors::{Location, PartialVMError, VMError};
use move_core_types::vm_status::{StatusCode, StatusType};
use narwhal_executor::SubscriberError;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};
use thiserror::Error;
use tonic::Status;
use typed_store::rocks::TypedStoreError;

pub const TRANSACTION_NOT_FOUND_MSG_PREFIX: &str = "Could not find the referenced transaction";

#[macro_export]
macro_rules! fp_bail {
    ($e:expr) => {
        return Err($e)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! fp_ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            fp_bail!($e);
        }
    };
}
pub(crate) use fp_ensure;

#[macro_export]
macro_rules! exit_main {
    ($result:expr) => {
        match $result {
            Ok(_) => (),
            Err(err) => {
                println!("{}", err.to_string().bold().red());
                std::process::exit(1);
            }
        }
    };
}

/// Custom error type for Sui.
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error, Hash)]
#[allow(clippy::large_enum_variant)]
pub enum SuiError {
    // Object misuse issues
    #[error("Error checking transaction input objects: {:?}", errors)]
    TransactionInputObjectsErrors { errors: Vec<SuiError> },
    #[error("Attempt to transfer an object that's not owned.")]
    TransferUnownedError,
    #[error("Attempt to transfer an object that does not have public transfer. Object transfer must be done instead using a distinct Move function call.")]
    TransferObjectWithoutPublicTransferError,
    #[error("A move package is expected, instead a move object is passed: {object_id}")]
    MoveObjectAsPackage { object_id: ObjectID },
    #[error("The SUI coin to be transferred has balance {balance}, which is not enough to cover the transfer amount {required}")]
    TransferInsufficientBalance { balance: u64, required: u64 },
    #[error("A move object is expected, instead a move package is passed: {object_id}")]
    MovePackageAsObject { object_id: ObjectID },
    #[error("Expecting a singler owner, shared ownership found")]
    UnexpectedOwnerType,
    #[error("Shared object not yet supported")]
    UnsupportedSharedObjectError,
    #[error("Object used as shared is not shared.")]
    NotSharedObjectError,
    #[error("An object that's owned by another object cannot be deleted or wrapped. It must be transferred to an account address first before deletion")]
    DeleteObjectOwnedObject,
    #[error("The shared locks for this transaction have not yet been set.")]
    SharedObjectLockNotSetError,
    #[error("Invalid Batch Transaction: {}", error)]
    InvalidBatchTransaction { error: String },
    #[error(
        "Object {child_id:?} is owned by object {parent_id:?}. \
        Objects owned by other objects cannot be used as input arguments."
    )]
    InvalidChildObjectArgument {
        child_id: ObjectID,
        parent_id: ObjectID,
    },

    // Signature verification
    #[error("Signature is not valid: {}", error)]
    InvalidSignature { error: String },
    #[error("Sender Signature must be verified separately from Authority Signature")]
    SenderSigUnbatchable,
    #[error("Value was not signed by the correct sender: {}", error)]
    IncorrectSigner { error: String },
    #[error("Value was not signed by a known authority")]
    UnknownSigner,
    // Certificate verification
    #[error(
        "Signature or certificate from wrong epoch, expected {expected_epoch}, got {actual_epoch}"
    )]
    WrongEpoch {
        expected_epoch: EpochId,
        actual_epoch: EpochId,
    },
    #[error("Signatures in a certificate must form a quorum")]
    CertificateRequiresQuorum,
    #[error("Authority {authority_name:?} could not sync certificate: {err:?}")]
    CertificateSyncError { authority_name: String, err: String },
    #[error(
        "The given sequence number ({given_sequence:?}) must match the next expected sequence ({expected_sequence:?}) number of the object ({object_id:?})"
    )]
    UnexpectedSequenceNumber {
        object_id: ObjectID,
        expected_sequence: SequenceNumber,
        given_sequence: SequenceNumber,
    },
    #[error("Invalid Authority Bitmap: {}", error)]
    InvalidAuthorityBitmap { error: String },
    #[error("Unexpected validator response from handle_transaction: {err}")]
    UnexpectedResultFromValidatorHandleTransaction { err: String },
    #[error("Transaction certificate processing failed: {err}")]
    ErrorWhileProcessingCertificate { err: String },
    #[error(
        "Failed to process transaction on a quorum of validators to form a transaction certificate. Locked objects: {:#?}. Validator errors: {:#?}",
        conflicting_tx_digests,
        errors.iter().map(| e | ToString::to_string(&e)).collect::<Vec<String>>()
    )]
    QuorumFailedToProcessTransaction {
        good_stake: StakeUnit,
        errors: Vec<SuiError>,
        conflicting_tx_digests:
            BTreeMap<TransactionDigest, (Vec<(AuthorityName, ObjectRef)>, StakeUnit)>,
    },
    #[error(
        "Failed to process transaction on a quorum of validators to form a transaction certificate because of locked objects, but retried a conflicting transaction {:?}, success: {}",
        conflicting_tx_digest,
        conflicting_tx_retry_success
    )]
    QuorumFailedToProcessTransactionWithConflictingTransactionRetried {
        conflicting_tx_digest: TransactionDigest,
        conflicting_tx_retry_success: bool,
    },
    #[error(
    "Failed to execute certificate on a quorum of validators. Validator errors: {:#?}",
    errors.iter().map(| e | ToString::to_string(&e)).collect::<Vec<String>>()
    )]
    QuorumFailedToExecuteCertificate { errors: Vec<SuiError> },
    #[error("Module publish failed: {err}")]
    ErrorWhileProcessingPublish { err: String },
    #[error("Move call failed: {err}")]
    ErrorWhileProcessingMoveCall { err: String },
    #[error("An invalid answer was returned by the authority while requesting information")]
    ErrorWhileRequestingInformation,
    #[error("Object fetch failed for {object_id:?}, err {err:?}.")]
    ObjectFetchFailed { object_id: ObjectID, err: String },
    #[error("Object {object_id:?} at old version: {current_sequence_number:?}")]
    MissingEarlierConfirmations {
        object_id: ObjectID,
        current_sequence_number: VersionNumber,
    },
    #[error("System Transaction not accepted")]
    InvalidSystemTransaction,
    // Synchronization validation
    #[error("Transaction index must increase by one")]
    UnexpectedTransactionIndex,
    #[error("Once one iterator is allowed on a stream at once.")]
    ConcurrentIteratorError,
    #[error("The notifier subsystem is closed.")]
    ClosedNotifierError,

    // Account access
    #[error("No certificate with digest: {certificate_digest:?}")]
    CertificateNotfound {
        certificate_digest: TransactionDigest,
    },
    #[error("No parent for object {object_id:?} at this sequence number {sequence:?}")]
    ParentNotfound {
        object_id: ObjectID,
        sequence: SequenceNumber,
    },
    #[error("Unknown sender's account")]
    UnknownSenderAccount,
    #[error("Signatures in a certificate must be from different authorities.")]
    CertificateAuthorityReuse,
    #[error("Sequence numbers above the maximal value are not usable for transfers.")]
    InvalidSequenceNumber,
    #[error("Sequence number overflow.")]
    SequenceOverflow,
    #[error("Sequence number underflow.")]
    SequenceUnderflow,
    #[error("Wrong shard used.")]
    WrongShard,
    #[error("Invalid cross shard update.")]
    InvalidCrossShardUpdate,
    #[error("Invalid authenticator")]
    InvalidAuthenticator,
    #[error("Invalid address")]
    InvalidAddress,
    #[error("Invalid transaction digest.")]
    InvalidTransactionDigest,
    #[error(
        "Invalid Object digest for object {object_id:?}. Expected digest : {expected_digest:?}."
    )]
    InvalidObjectDigest {
        object_id: ObjectID,
        expected_digest: ObjectDigest,
    },
    #[error("Cannot deserialize.")]
    InvalidDecoding,
    #[error("Unexpected message.")]
    UnexpectedMessage,
    #[error("The transaction inputs contain duplicates ObjectRef's")]
    DuplicateObjectRefInput,
    #[error("Network error while querying service: {:?}.", error)]
    ClientIoError { error: String },
    #[error("Cannot transfer immutable object.")]
    TransferImmutableError,
    #[error("Wrong initial version given for shared object")]
    SharedObjectStartingVersionMismatch,

    // Errors related to batches
    #[error("The range specified is invalid.")]
    InvalidSequenceRangeError,
    #[error("No batches matched the range requested.")]
    NoBatchesFoundError,
    #[error("The channel to repond to the client returned an error.")]
    CannotSendClientMessageError,
    #[error("Subscription service had to drop {0} items")]
    SubscriptionItemsDroppedError(u64),
    #[error("Subscription service closed.")]
    SubscriptionServiceClosed,
    #[error("Checkpointing error: {}", error)]
    CheckpointingError { error: String },
    #[error(
        "ExecutionDriver error for {:?}: {} - Caused by : {}",
        digest,
        msg,
        format!("[ {} ]", errors.iter().map(|e| ToString::to_string(&e)).collect::<Vec<String>>().join("; ")),
    )]
    ExecutionDriverError {
        digest: TransactionDigest,
        msg: String,
        errors: Vec<SuiError>,
    },

    // Move module publishing related errors
    #[error("Failed to load the Move module, reason: {error:?}.")]
    ModuleLoadFailure { error: String },
    #[error("Failed to verify the Move module, reason: {error:?}.")]
    ModuleVerificationFailure { error: String },
    #[error("Failed to verify the Move module, reason: {error:?}.")]
    ModuleDeserializationFailure { error: String },
    #[error("Failed to publish the Move module(s), reason: {error:?}.")]
    ModulePublishFailure { error: String },
    #[error("Failed to build Move modules: {error}.")]
    ModuleBuildFailure { error: String },
    #[error("Dependent package not found on-chain: {package_id:?}")]
    DependentPackageNotFound { package_id: ObjectID },
    #[error("Move unit tests failed: {error:?}")]
    MoveUnitTestFailure { error: String },

    // Move call related errors
    #[error("Function resolution failure: {error:?}.")]
    FunctionNotFound { error: String },
    #[error("Module not found in package: {module_name:?}.")]
    ModuleNotFound { module_name: String },
    #[error("Function signature is invalid: {error:?}.")]
    InvalidFunctionSignature { error: String },
    #[error("Non-`entry` function used for entry point to execution: {error:?}.")]
    InvalidNonEntryFunction { error: String },
    #[error("Type error while binding function arguments: {error:?}.")]
    TypeError { error: String },
    #[error("Execution aborted: {error:?}.")]
    AbortedExecution { error: String },
    #[error("Invalid move event: {error:?}.")]
    InvalidMoveEvent { error: String },
    #[error("Circular object ownership detected")]
    CircularObjectOwnership,
    #[error("When an (either direct or indirect) child object of a shared object is passed as a Move argument,\
        either the child object's type or the shared object's type must be defined in the same module \
        as the called function. This is violated by object {child} (defined in module '{child_module}'), \
        whose ancestor {ancestor} is a shared object (defined in module '{ancestor_module}'), \
        and neither are defined in this module '{current_module}'")]
    InvalidSharedChildUse {
        child: ObjectID,
        child_module: String,
        ancestor: ObjectID,
        ancestor_module: String,
        current_module: String,
    },

    // Gas related errors
    #[error("Gas budget set higher than max: {error:?}.")]
    GasBudgetTooHigh { error: String },
    #[error("Insufficient gas: {error:?}.")]
    InsufficientGas { error: String },

    // Internal state errors
    #[error("Attempt to update state of TxContext from a different instance than original.")]
    InvalidTxUpdate,
    #[error("Attempt to re-initialize a transaction lock for objects {:?}.", refs)]
    ObjectLockAlreadyInitialized { refs: Vec<ObjectRef> },
    #[error("Object {obj_ref:?} lock has not been initialized.")]
    ObjectLockUninitialized { obj_ref: ObjectRef },
    #[error(
        "Object {obj_ref:?} already locked by a different transaction: {pending_transaction:?}"
    )]
    ObjectLockConflict {
        obj_ref: ObjectRef,
        pending_transaction: TransactionDigest,
    },
    #[error("Objects {obj_refs:?} are already locked by a transaction from a future epoch {locked_epoch:?}), attempt to override with a transaction from epoch {new_epoch:?}")]
    ObjectLockedAtFutureEpoch {
        obj_refs: Vec<ObjectRef>,
        locked_epoch: EpochId,
        new_epoch: EpochId,
    },
    #[error("{TRANSACTION_NOT_FOUND_MSG_PREFIX} [{:?}].", digest)]
    TransactionNotFound { digest: TransactionDigest },
    #[error(
        "Could not find the referenced object {:?} at version {:?}.",
        object_id,
        version
    )]
    ObjectNotFound {
        object_id: ObjectID,
        version: Option<SequenceNumber>,
    },
    #[error(
        "Transaction involving Shared Object {:?} at version {:?} is not ready for execution because prior transactions have yet to execute.",
        object_id,
        version_not_ready
    )]
    SharedObjectPriorVersionsPendingExecution {
        object_id: ObjectID,
        version_not_ready: SequenceNumber,
    },
    #[error("Could not find the referenced object {:?} as the asked version {:?} is higher than the latest {:?}", object_id, asked_version, latest_version)]
    ObjectSequenceNumberTooHigh {
        object_id: ObjectID,
        asked_version: SequenceNumber,
        latest_version: SequenceNumber,
    },
    #[error("Object deleted at reference {:?}.", object_ref)]
    ObjectDeleted { object_ref: ObjectRef },
    #[error("Object ID did not have the expected type")]
    BadObjectType { error: String },
    #[error("Move Execution failed")]
    MoveExecutionFailure,
    #[error("Wrong number of parameters for the transaction.")]
    ObjectInputArityViolation,
    #[error("Execution invariant violated")]
    ExecutionInvariantViolation,
    #[error("Authority did not return the information it is expected to have.")]
    AuthorityInformationUnavailable,
    #[error("Failed to update authority.")]
    AuthorityUpdateFailure,
    #[error("Validator {authority:?} is faulty in a Byzantine manner: {reason:?}")]
    ByzantineAuthoritySuspicion {
        authority: AuthorityName,
        reason: String,
    },
    #[error(
        "Sync from authority failed. From {xsource:?} to {destination:?}, digest {tx_digest:?}: {error:?}",
    )]
    PairwiseSyncFailed {
        xsource: AuthorityName,
        destination: AuthorityName,
        tx_digest: TransactionDigest,
        error: Box<SuiError>,
    },
    #[error("Storage error")]
    StorageError(#[from] TypedStoreError),
    #[error("Non-RocksDB Storage error: {0}")]
    GenericStorageError(String),
    #[error(
        "Attempted to access {object} through parent {given_parent}, \
        but it's actual parent is {actual_owner}"
    )]
    InvalidChildObjectAccess {
        object: ObjectID,
        given_parent: ObjectID,
        actual_owner: Owner,
    },

    #[error("Missing fields/data in storage error: {0}")]
    StorageMissingFieldError(String),
    #[error("Corrupted fields/data in storage error: {0}")]
    StorageCorruptedFieldError(String),

    #[error("Batch error: cannot send transaction to batch.")]
    BatchErrorSender,
    #[error("Authority Error: {error:?}")]
    GenericAuthorityError { error: String },

    #[error("Failed to dispatch event: {error:?}")]
    EventFailedToDispatch { error: String },

    #[error("Failed to serialize Owner: {error:?}")]
    OwnerFailedToSerialize { error: String },

    #[error("Failed to deserialize fields into JSON: {error:?}")]
    ExtraFieldFailedToDeserialize { error: String },

    #[error("Failed to execute transaction locally by Orchestrator: {error:?}")]
    TransactionOrchestratorLocalExecutionError { error: String },

    // Errors returned by authority and client read API's
    #[error("Failure serializing object in the requested format: {:?}", error)]
    ObjectSerializationError { error: String },
    #[error("Event store component is not active on this node")]
    NoEventStore,

    // Client side error
    #[error("Client state has a different pending transaction.")]
    ConcurrentTransactionError,
    #[error("Transfer should be received by us.")]
    IncorrectRecipientError,
    #[error("Too many authority errors were detected for {}: {:?}", action, errors)]
    TooManyIncorrectAuthorities {
        errors: Vec<(AuthorityName, SuiError)>,
        action: String,
    },
    #[error("Inconsistent results observed in the Gateway. This should not happen and typically means there is a bug in the Sui implementation. Details: {error:?}")]
    InconsistentGatewayResult { error: String },
    #[error("Invalid transaction range query to the gateway: {:?}", error)]
    GatewayInvalidTxRangeQuery { error: String },
    #[error("Gateway checking transaction validity failed: {:?}", error)]
    GatewayTransactionPrepError { error: String },

    // Errors related to the authority-consensus interface.
    #[error("Authority state can be modified by a single consensus client at the time")]
    OnlyOneConsensusClientPermitted,
    #[error("Failed to connect with consensus node: {0}")]
    ConsensusConnectionBroken(String),
    #[error("Failed to hear back from consensus: {0}")]
    FailedToHearBackFromConsensus(String),
    #[error("Failed to execute handle_consensus_transaction on Sui: {0}")]
    HandleConsensusTransactionFailure(String),
    #[error("Consensus listener has too many pending transactions and is out of capacity: {0}")]
    ListenerCapacityExceeded(usize),
    #[error("Failed to serialize/deserialize Narwhal message: {0}")]
    ConsensusSuiSerializationError(String),
    #[error("Only shared object transactions need to be sequenced")]
    NotASharedObjectTransaction,

    // Cryptography errors.
    #[error("Signature seed invalid length, input byte size was: {0}")]
    SignatureSeedInvalidLength(usize),
    #[error("HKDF error: {0}")]
    HkdfError(String),
    #[error("Signature key generation error: {0}")]
    SignatureKeyGenError(String),
    #[error("Key Conversion Error: {0}")]
    KeyConversionError(String),
    #[error("Invalid Private Key provided")]
    InvalidPrivateKey,

    // Epoch related errors.
    #[error("Validator temporarily stopped processing transactions due to epoch change")]
    ValidatorHaltedAtEpochEnd,
    #[error("Inconsistent state detected during epoch change: {:?}", error)]
    InconsistentEpochState { error: String },
    #[error("Error when advancing epoch: {:?}", error)]
    AdvanceEpochError { error: String },

    // These are errors that occur when an RPC fails and is simply the utf8 message sent in a
    // Tonic::Status
    #[error("{1} - {0}")]
    RpcError(String, String),

    #[error("Error when calling executeTransaction rpc endpoint: {:?}", error)]
    RpcExecuteTransactionError { error: String },

    #[error("Use of disabled feature: {:?}", error)]
    UnsupportedFeatureError { error: String },

    #[error("Unable to communicate with the Quorum Driver channel: {:?}", error)]
    QuorumDriverCommunicationError { error: String },

    #[error("Operation timed out")]
    TimeoutError,

    #[error("Error executing {0}")]
    ExecutionError(String),

    #[error("Invalid committee composition")]
    InvalidCommittee(String),

    #[error("Missing committee information for epoch {0}")]
    MissingCommitteeAtEpoch(EpochId),

    #[error("Failed to get supermajority's consensus on committee information for minimal epoch: {minimal_epoch}")]
    FailedToGetAgreedCommitteeFromMajority { minimal_epoch: EpochId },

    #[error("Empty input coins for Pay related transaction")]
    EmptyInputCoins,

    #[error("SUI payment transactions use first input coin for gas payment, but found a different gas object.")]
    UnexpectedGasPaymentObject,
}

pub type SuiResult<T = ()> = Result<T, SuiError>;

// TODO these are both horribly wrong, categorization needs to be considered
impl From<PartialVMError> for SuiError {
    fn from(error: PartialVMError) -> Self {
        SuiError::ModuleVerificationFailure {
            error: error.to_string(),
        }
    }
}

impl From<ExecutionError> for SuiError {
    fn from(error: ExecutionError) -> Self {
        SuiError::ExecutionError(error.to_string())
    }
}

impl From<VMError> for SuiError {
    fn from(error: VMError) -> Self {
        SuiError::ModuleVerificationFailure {
            error: error.to_string(),
        }
    }
}

impl From<SubscriberError> for SuiError {
    fn from(error: SubscriberError) -> Self {
        SuiError::HandleConsensusTransactionFailure(error.to_string())
    }
}

impl From<Status> for SuiError {
    fn from(status: Status) -> Self {
        let result = bincode::deserialize::<SuiError>(status.details());
        if let Ok(sui_error) = result {
            sui_error
        } else {
            Self::RpcError(
                status.message().to_owned(),
                status.code().description().to_owned(),
            )
        }
    }
}

impl From<SuiError> for Status {
    fn from(error: SuiError) -> Self {
        let bytes = bincode::serialize(&error).unwrap();
        Status::with_details(tonic::Code::Internal, error.to_string(), bytes.into())
    }
}

impl From<ExecutionErrorKind> for SuiError {
    fn from(kind: ExecutionErrorKind) -> Self {
        ExecutionError::from_kind(kind).into()
    }
}

impl From<&str> for SuiError {
    fn from(error: &str) -> Self {
        SuiError::GenericAuthorityError {
            error: error.to_string(),
        }
    }
}

impl SuiError {
    pub fn indicates_epoch_change(&self) -> bool {
        match self {
            SuiError::QuorumFailedToProcessTransaction { errors, .. }
            | SuiError::QuorumFailedToExecuteCertificate { errors, .. } => {
                errors.iter().any(|err| {
                    matches!(err, SuiError::ValidatorHaltedAtEpochEnd)
                        || matches!(self, SuiError::MissingCommitteeAtEpoch(_))
                })
            }
            SuiError::ValidatorHaltedAtEpochEnd | SuiError::MissingCommitteeAtEpoch(_) => true,
            _ => false,
        }
    }
}

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type ExecutionErrorKind = ExecutionFailureStatus;

#[derive(Debug)]
pub struct ExecutionError {
    inner: Box<ExecutionErrorInner>,
}

#[derive(Debug)]
struct ExecutionErrorInner {
    kind: ExecutionErrorKind,
    source: Option<BoxError>,
}

impl ExecutionError {
    pub fn new(kind: ExecutionErrorKind, source: Option<BoxError>) -> Self {
        Self {
            inner: Box::new(ExecutionErrorInner { kind, source }),
        }
    }

    pub fn new_with_source<E: Into<BoxError>>(kind: ExecutionErrorKind, source: E) -> Self {
        Self::new(kind, Some(source.into()))
    }

    pub fn from_kind(kind: ExecutionErrorKind) -> Self {
        Self::new(kind, None)
    }

    pub fn kind(&self) -> &ExecutionErrorKind {
        &self.inner.kind
    }

    pub fn to_execution_status(&self) -> ExecutionFailureStatus {
        self.kind().clone()
    }
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExecutionError: {:?}", self)
    }
}

impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source.as_ref().map(|e| &**e as _)
    }
}

impl From<ExecutionErrorKind> for ExecutionError {
    fn from(kind: ExecutionErrorKind) -> Self {
        Self::from_kind(kind)
    }
}

impl From<VMError> for ExecutionError {
    fn from(error: VMError) -> Self {
        let kind = match (error.major_status(), error.sub_status(), error.location()) {
            (StatusCode::EXECUTED, _, _) => {
                // If we have an error the status probably shouldn't ever be Executed
                debug_assert!(false, "VmError shouldn't ever report successful execution");
                ExecutionFailureStatus::VMInvariantViolation
            }
            (StatusCode::ABORTED, None, _) => {
                debug_assert!(false, "No abort code");
                // this is a Move VM invariant violation, the code should always be there
                ExecutionFailureStatus::VMInvariantViolation
            }
            (StatusCode::ABORTED, _, Location::Script) => {
                debug_assert!(false, "Scripts are not used in Sui");
                // this is a Move VM invariant violation, in the sense that the location
                // is malformed
                ExecutionFailureStatus::VMInvariantViolation
            }
            (StatusCode::ABORTED, Some(code), Location::Module(id)) => {
                ExecutionFailureStatus::MoveAbort(id.to_owned(), code)
            }
            (StatusCode::OUT_OF_GAS, _, _) => ExecutionFailureStatus::InsufficientGas,
            _ => match error.major_status().status_type() {
                StatusType::Execution => ExecutionFailureStatus::MovePrimitiveRuntimeError,
                StatusType::Validation
                | StatusType::Verification
                | StatusType::Deserialization
                | StatusType::Unknown => {
                    ExecutionFailureStatus::VMVerificationOrDeserializationError
                }
                StatusType::InvariantViolation => ExecutionFailureStatus::VMInvariantViolation,
            },
        };
        Self::new_with_source(kind, error)
    }
}
