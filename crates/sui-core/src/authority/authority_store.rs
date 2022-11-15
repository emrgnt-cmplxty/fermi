// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use super::{
    authority_store_tables::{AuthorityEpochTables, AuthorityPerpetualTables},
    *,
};
use crate::authority::authority_store_tables::ExecutionIndicesWithHash;
use arc_swap::ArcSwap;
use narwhal_executor::ExecutionIndices;
use once_cell::sync::OnceCell;
use rocksdb::Options;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::BTreeMap;
use std::iter;
use std::path::Path;
use std::sync::Arc;
use std::{fmt::Debug, path::PathBuf};
use sui_storage::{
    mutex_table::{LockGuard, MutexTable},
    write_ahead_log::{DBWriteAheadLog, WriteAheadLog},
    LockService,
};
use sui_types::batch::TxSequenceNumber;
use sui_types::crypto::{AuthoritySignInfo, EmptySignInfo};
use sui_types::message_envelope::VerifiedEnvelope;
use sui_types::object::Owner;
use sui_types::storage::{ChildObjectResolver, SingleTxContext, WriteKind};
use sui_types::{base_types::SequenceNumber, storage::ParentSync};
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tracing::{debug, info, trace};
use typed_store::rocks::DBBatch;
use typed_store::traits::Map;

pub type AuthorityStore = SuiDataStore<AuthoritySignInfo>;
pub type GatewayStore = SuiDataStore<EmptySignInfo>;

pub struct CertLockGuard(LockGuard);

const NUM_SHARDS: usize = 4096;
const SHARD_SIZE: usize = 128;

/// The key where the latest consensus index is stored in the database.
// TODO: Make a single table (e.g., called `variables`) storing all our lonely variables in one place.
const LAST_CONSENSUS_INDEX_ADDR: u64 = 0;

/// ALL_OBJ_VER determines whether we want to store all past
/// versions of every object in the store. Authority doesn't store
/// them, but other entities such as replicas will.
/// S is a template on Authority signature state. This allows SuiDataStore to be used on either
/// authorities or non-authorities. Specifically, when storing transactions and effects,
/// S allows SuiDataStore to either store the authority signed version or unsigned version.
pub struct SuiDataStore<S> {
    /// A write-ahead/recovery log used to ensure we finish fully processing certs after errors or
    /// crashes.
    pub wal: Arc<DBWriteAheadLog<TrustedCertificate>>,

    /// The LockService this store depends on for locking functionality
    lock_service: LockService,

    /// Internal vector of locks to manage concurrent writes to the database
    mutex_table: MutexTable<ObjectDigest>,

    pub(crate) perpetual_tables: AuthorityPerpetualTables<S>,
    pub(crate) epoch_tables: ArcSwap<AuthorityEpochTables<S>>,

    // needed for re-opening epoch db.
    path: PathBuf,
    db_options: Option<Options>,

    pub(crate) effects_notify_read: NotifyRead<TransactionDigest, TransactionEffects>,
    pub(crate) consensus_notify_read: NotifyRead<TransactionDigest, ()>,
}

impl<S: Eq + Debug + Serialize + for<'de> Deserialize<'de>> SuiDataStore<S> {
    /// Open an authority store by directory path
    pub fn open(path: &Path, db_options: Option<Options>) -> SuiResult<Self> {
        let perpetual_tables = AuthorityPerpetualTables::open(path, db_options.clone());

        let epoch = if perpetual_tables.database_is_empty()? {
            0
        } else {
            perpetual_tables.get_epoch()?
        };

        let epoch_tables = Arc::new(AuthorityEpochTables::open(epoch, path, db_options.clone()));

        // For now, create one LockService for each SuiDataStore, and we use a specific
        // subdir of the data store directory
        let lockdb_path: PathBuf = path.join("lockdb");
        let lock_service =
            LockService::new(lockdb_path, None).expect("Could not initialize lockdb");

        let wal_path = path.join("recovery_log");
        let wal = Arc::new(DBWriteAheadLog::new(wal_path));

        Ok(Self {
            wal,
            lock_service,
            mutex_table: MutexTable::new(NUM_SHARDS, SHARD_SIZE),
            perpetual_tables,
            epoch_tables: epoch_tables.into(),
            path: path.into(),
            db_options,
            effects_notify_read: NotifyRead::new(),
            consensus_notify_read: NotifyRead::new(),
        })
    }

    #[allow(dead_code)]
    pub(crate) fn reopen_epoch_db(&self, new_epoch: EpochId) {
        info!(?new_epoch, "re-opening AuthorityEpochTables for new epoch");
        let epoch_tables = Arc::new(AuthorityEpochTables::open(
            new_epoch,
            &self.path,
            self.db_options.clone(),
        ));
        self.epoch_tables.store(epoch_tables);
    }

    pub fn epoch_tables(&self) -> arc_swap::Guard<Arc<AuthorityEpochTables<S>>> {
        self.epoch_tables.load()
    }

    pub async fn acquire_tx_guard(&self, cert: &VerifiedCertificate) -> SuiResult<CertTxGuard> {
        let digest = cert.digest();
        let guard = self.wal.begin_tx(digest, cert.serializable_ref()).await?;

        if guard.retry_num() > MAX_TX_RECOVERY_RETRY {
            // If the tx has been retried too many times, it could be a poison pill, and we should
            // prevent the client from continually retrying it.
            let err = "tx has exceeded the maximum retry limit for transient errors".to_owned();
            debug!(?digest, "{}", err);
            return Err(SuiError::ErrorWhileProcessingCertificate { err });
        }

        Ok(guard)
    }

    /// Acquire the lock for a tx without writing to the WAL.
    pub async fn acquire_tx_lock(&self, digest: &TransactionDigest) -> CertLockGuard {
        CertLockGuard(self.wal.acquire_lock(digest).await)
    }

    /// Returns the TransactionEffects if we have an effects structure for this transaction digest
    pub fn get_effects(
        &self,
        transaction_digest: &TransactionDigest,
    ) -> SuiResult<TransactionEffects> {
        self.perpetual_tables
            .effects
            .get(transaction_digest)?
            .map(|data| data.into_data())
            .ok_or(SuiError::TransactionNotFound {
                digest: *transaction_digest,
            })
    }

    /// Returns true if we have an effects structure for this transaction digest
    pub fn effects_exists(&self, transaction_digest: &TransactionDigest) -> SuiResult<bool> {
        self.perpetual_tables
            .effects
            .contains_key(transaction_digest)
            .map_err(|e| e.into())
    }

    /// Returns true if there are no objects in the database
    pub fn database_is_empty(&self) -> SuiResult<bool> {
        self.perpetual_tables.database_is_empty()
    }

    pub fn next_sequence_number(&self) -> Result<TxSequenceNumber, SuiError> {
        Ok(self
            .perpetual_tables
            .executed_sequence
            .iter()
            .skip_prior_to(&TxSequenceNumber::MAX)?
            .next()
            .map(|(v, _)| v + 1u64)
            .unwrap_or(0))
    }

    #[cfg(test)]
    pub fn side_sequence(&self, seq: TxSequenceNumber, digest: &ExecutionDigests) {
        self.perpetual_tables
            .executed_sequence
            .insert(&seq, digest)
            .unwrap();
    }

    #[cfg(test)]
    pub fn get_next_object_version(&self, obj: &ObjectID) -> Option<SequenceNumber> {
        self.epoch_tables().next_object_versions.get(obj).unwrap()
    }

    /// Gets all pending certificates. Used during recovery.
    pub fn all_pending_certificates(&self) -> SuiResult<Vec<VerifiedCertificate>> {
        Ok(self
            .epoch_tables()
            .pending_certificates
            .iter()
            .map(|(_, cert)| cert.into())
            .collect())
    }

    /// Stores a list of pending certificates to be executed.
    pub fn store_pending_certificates(&self, certs: &[VerifiedCertificate]) -> SuiResult<()> {
        let batch = self
            .epoch_tables()
            .pending_certificates
            .batch()
            .insert_batch(
                &self.epoch_tables().pending_certificates,
                certs
                    .iter()
                    .map(|cert| (*cert.digest(), cert.clone().serializable())),
            )?;
        batch.write()?;
        Ok(())
    }

    /// Gets one pending certificate.
    pub fn get_pending_certificate(
        &self,
        tx: &TransactionDigest,
    ) -> SuiResult<Option<VerifiedCertificate>> {
        Ok(self
            .epoch_tables()
            .pending_certificates
            .get(tx)?
            .map(|c| c.into()))
    }

    /// Checks if a certificate is in the pending queue.
    pub fn pending_certificate_exists(&self, tx: &TransactionDigest) -> Result<bool, SuiError> {
        Ok(self.epoch_tables().pending_certificates.contains_key(tx)?)
    }

    /// Deletes one pending certificate.
    pub fn remove_pending_certificate(&self, digest: &TransactionDigest) -> SuiResult<()> {
        self.epoch_tables().pending_certificates.remove(digest)?;
        Ok(())
    }

    /// Deletes all pending certificates in the epoch.
    pub fn cleanup_pending_certificates(&self) -> SuiResult<()> {
        self.epoch_tables().pending_certificates.clear()?;
        Ok(())
    }

    /// A function that acquires all locks associated with the objects (in order to avoid deadlocks).
    async fn acquire_locks(&self, input_objects: &[ObjectRef]) -> Vec<LockGuard> {
        self.mutex_table
            .acquire_locks(input_objects.iter().map(|(_, _, digest)| *digest))
            .await
    }

    // Methods to read the store
    pub fn get_owner_objects(&self, owner: Owner) -> Result<Vec<ObjectInfo>, SuiError> {
        debug!(?owner, "get_owner_objects");
        Ok(self
            .perpetual_tables
            .owner_index
            .iter()
            // The object id 0 is the smallest possible
            .skip_to(&(owner, ObjectID::ZERO))?
            .take_while(|((object_owner, _), _)| (object_owner == &owner))
            .map(|(_, object_info)| object_info)
            .collect())
    }

    pub fn get_object_by_key(
        &self,
        object_id: &ObjectID,
        version: VersionNumber,
    ) -> Result<Option<Object>, SuiError> {
        Ok(self
            .perpetual_tables
            .objects
            .get(&ObjectKey(*object_id, version))?)
    }

    pub fn object_exists(
        &self,
        object_id: &ObjectID,
        version: VersionNumber,
    ) -> Result<bool, SuiError> {
        Ok(self
            .perpetual_tables
            .objects
            .contains_key(&ObjectKey(*object_id, version))?)
    }

    /// Read an object and return it, or Err(ObjectNotFound) if the object was not found.
    pub fn get_object(&self, object_id: &ObjectID) -> Result<Option<Object>, SuiError> {
        self.perpetual_tables.get_object(object_id)
    }

    /// Get many objects
    pub fn get_objects(&self, objects: &[ObjectID]) -> Result<Vec<Option<Object>>, SuiError> {
        let mut result = Vec::new();
        for id in objects {
            result.push(self.get_object(id)?);
        }
        Ok(result)
    }

    pub fn check_input_objects(
        &self,
        objects: &[InputObjectKind],
    ) -> Result<Vec<Object>, SuiError> {
        let mut result = Vec::new();
        let mut errors = Vec::new();
        for kind in objects {
            let obj = match kind {
                InputObjectKind::MovePackage(id) | InputObjectKind::SharedMoveObject { id, .. } => {
                    self.get_object(id)?
                }
                InputObjectKind::ImmOrOwnedMoveObject(objref) => {
                    self.get_object_by_key(&objref.0, objref.1)?
                }
            };
            match obj {
                Some(obj) => result.push(obj),
                None => errors.push(kind.object_not_found_error()),
            }
        }
        if !errors.is_empty() {
            Err(SuiError::TransactionInputObjectsErrors { errors })
        } else {
            Ok(result)
        }
    }

    /// When making changes, please see if check_sequenced_input_objects() below needs
    /// similar changes as well.
    pub fn get_missing_input_objects(
        &self,
        digest: &TransactionDigest,
        objects: &[InputObjectKind],
    ) -> Result<Vec<ObjectKey>, SuiError> {
        let shared_locks_cell: OnceCell<HashMap<_, _>> = OnceCell::new();

        let mut missing = Vec::new();
        for kind in objects {
            match kind {
                InputObjectKind::SharedMoveObject { id, .. } => {
                    let shared_locks = shared_locks_cell.get_or_try_init(|| {
                        Ok::<HashMap<ObjectID, SequenceNumber>, SuiError>(
                            self.all_shared_locks(digest)?.into_iter().collect(),
                        )
                    })?;
                    match shared_locks.get(id) {
                        Some(version) => {
                            if !self.object_exists(id, *version)? {
                                // When this happens, other transactions that use smaller versions of
                                // this shared object haven't finished execution.
                                missing.push(ObjectKey(*id, *version));
                            }
                        }
                        None => {
                            // Abort the function because the lock should have been set.
                            return Err(SuiError::SharedObjectLockNotSetError);
                        }
                    };
                }
                InputObjectKind::MovePackage(id) => {
                    // Move package always uses version 1.
                    let version = VersionNumber::from_u64(1);
                    if !self.object_exists(id, version)? {
                        // The cert cannot have been formed if immutable inputs were missing.
                        missing.push(ObjectKey(*id, version));
                    }
                }
                InputObjectKind::ImmOrOwnedMoveObject(objref) => {
                    if !self.object_exists(&objref.0, objref.1)? {
                        missing.push(ObjectKey::from(objref));
                    }
                }
            };
        }

        Ok(missing)
    }

    /// When making changes, please see if get_missing_input_objects() above needs
    /// similar changes as well.
    pub fn check_sequenced_input_objects(
        &self,
        digest: &TransactionDigest,
        objects: &[InputObjectKind],
    ) -> Result<Vec<Object>, SuiError> {
        let shared_locks_cell: OnceCell<HashMap<_, _>> = OnceCell::new();

        let mut result = Vec::new();
        let mut errors = Vec::new();
        for kind in objects {
            let obj = match kind {
                InputObjectKind::SharedMoveObject { id, .. } => {
                    let shared_locks = shared_locks_cell.get_or_try_init(|| {
                        Ok::<HashMap<ObjectID, SequenceNumber>, SuiError>(
                            self.all_shared_locks(digest)?.into_iter().collect(),
                        )
                    })?;
                    match shared_locks.get(id) {
                        Some(version) => {
                            if let Some(obj) = self.get_object_by_key(id, *version)? {
                                result.push(obj);
                            } else {
                                // When this happens, other transactions that use smaller versions of
                                // this shared object haven't finished execution.
                                errors.push(SuiError::SharedObjectPriorVersionsPendingExecution {
                                    object_id: *id,
                                    version_not_ready: *version,
                                });
                            }
                            continue;
                        }
                        None => {
                            errors.push(SuiError::SharedObjectLockNotSetError);
                            continue;
                        }
                    }
                }
                InputObjectKind::MovePackage(id) => self.get_object(id)?,
                InputObjectKind::ImmOrOwnedMoveObject(objref) => {
                    self.get_object_by_key(&objref.0, objref.1)?
                }
            };
            // SharedMoveObject should not reach here
            match obj {
                Some(obj) => result.push(obj),
                None => errors.push(kind.object_not_found_error()),
            }
        }
        if !errors.is_empty() {
            Err(SuiError::TransactionInputObjectsErrors { errors })
        } else {
            Ok(result)
        }
    }

    pub async fn get_tx_sequence(
        &self,
        tx: TransactionDigest,
    ) -> SuiResult<Option<TxSequenceNumber>> {
        self.lock_service.get_tx_sequence(tx).await
    }

    /// Get the transaction envelope that currently locks the given object,
    /// or returns Err(TransactionLockDoesNotExist) if the lock does not exist.
    pub async fn get_object_locking_transaction(
        &self,
        object_ref: &ObjectRef,
    ) -> SuiResult<Option<VerifiedEnvelope<SenderSignedData, S>>> {
        let tx_lock = self.lock_service.get_lock(*object_ref).await?.ok_or(
            SuiError::ObjectLockUninitialized {
                obj_ref: *object_ref,
            },
        )?;

        // Returns None if either no TX with the lock, or TX present but no entry in transactions table.
        // However we retry a couple times because the TX is written after the lock is acquired, so it might
        // just be a race.
        match tx_lock {
            Some(lock_info) => {
                let tx_digest = &lock_info.tx_digest;
                let mut retry_strategy = ExponentialBackoff::from_millis(2)
                    .factor(10)
                    .map(jitter)
                    .take(3);
                let mut tx_option = self.epoch_tables().transactions.get(tx_digest)?;
                while tx_option.is_none() {
                    if let Some(duration) = retry_strategy.next() {
                        // Wait to retry
                        tokio::time::sleep(duration).await;
                        trace!(?tx_digest, "Retrying getting pending transaction");
                    } else {
                        // No more retries, just quit
                        break;
                    }
                    tx_option = self.epoch_tables().transactions.get(tx_digest)?;
                }
                Ok(tx_option.map(|t| t.into()))
            }
            None => Ok(None),
        }
    }

    /// Read a certificate and return an option with None if it does not exist.
    pub fn read_certificate(
        &self,
        digest: &TransactionDigest,
    ) -> Result<Option<VerifiedCertificate>, SuiError> {
        self.perpetual_tables
            .certificates
            .get(digest)
            .map(|r| r.map(|c| c.into()))
            .map_err(|e| e.into())
    }

    /// Read the transactionDigest that is the parent of an object reference
    /// (ie. the transaction that created an object at this version.)
    pub fn parent(&self, object_ref: &ObjectRef) -> Result<Option<TransactionDigest>, SuiError> {
        self.perpetual_tables
            .parent_sync
            .get(object_ref)
            .map_err(|e| e.into())
    }

    /// Batch version of `parent` function.
    pub fn multi_get_parents(
        &self,
        object_refs: &[ObjectRef],
    ) -> Result<Vec<Option<TransactionDigest>>, SuiError> {
        self.perpetual_tables
            .parent_sync
            .multi_get(object_refs)
            .map_err(|e| e.into())
    }

    /// Returns all parents (object_ref and transaction digests) that match an object_id, at
    /// any object version, or optionally at a specific version.
    pub fn get_parent_iterator(
        &self,
        object_id: ObjectID,
        seq: Option<SequenceNumber>,
    ) -> Result<impl Iterator<Item = (ObjectRef, TransactionDigest)> + '_, SuiError> {
        let seq_inner = seq.unwrap_or_else(|| SequenceNumber::from(0));
        let obj_dig_inner = ObjectDigest::new([0; 32]);

        Ok(self
            .perpetual_tables
            .parent_sync
            .iter()
            // The object id [0; 16] is the smallest possible
            .skip_to(&(object_id, seq_inner, obj_dig_inner))?
            .take_while(move |((id, iseq, _digest), _txd)| {
                let mut flag = id == &object_id;
                if let Some(seq_num) = seq {
                    flag &= seq_num == *iseq;
                }
                flag
            }))
    }

    /// Read a lock for a specific (transaction, shared object) pair.
    #[cfg(test)] // Nothing wrong with this function, but it is not currently used outside of tests
    pub fn get_assigned_object_versions<'a>(
        &self,
        transaction_digest: &TransactionDigest,
        object_ids: impl Iterator<Item = &'a ObjectID>,
    ) -> Result<Vec<Option<SequenceNumber>>, SuiError> {
        let keys = object_ids.map(|objid| (*transaction_digest, *objid));

        self.epoch_tables()
            .assigned_object_versions
            .multi_get(keys)
            .map_err(SuiError::from)
    }

    /// Read a lock for a specific (transaction, shared object) pair.
    pub fn all_shared_locks(
        &self,
        transaction_digest: &TransactionDigest,
    ) -> Result<Vec<(ObjectID, SequenceNumber)>, SuiError> {
        Ok(self
            .epoch_tables()
            .assigned_object_versions
            .iter()
            .skip_to(&(*transaction_digest, ObjectID::ZERO))?
            .take_while(|((tx, _objid), _ver)| tx == transaction_digest)
            .map(|((_tx, objid), ver)| (objid, ver))
            .collect())
    }

    // Methods to mutate the store

    /// Insert a genesis object.
    pub async fn insert_genesis_object(&self, object: Object) -> SuiResult {
        // We only side load objects with a genesis parent transaction.
        debug_assert!(object.previous_transaction == TransactionDigest::genesis());
        let object_ref = object.compute_object_reference();
        self.insert_object_direct(object_ref, &object).await
    }

    /// Insert an object directly into the store, and also update relevant tables
    /// NOTE: does not handle transaction lock.
    /// This is used by the gateway to insert object directly.
    /// TODO: We need this today because we don't have another way to sync an account.
    pub async fn insert_object_direct(&self, object_ref: ObjectRef, object: &Object) -> SuiResult {
        // Insert object
        self.perpetual_tables
            .objects
            .insert(&object_ref.into(), object)?;

        // Update the index
        if object.get_single_owner().is_some() {
            self.perpetual_tables.owner_index.insert(
                &(object.owner, object_ref.0),
                &ObjectInfo::new(&object_ref, object),
            )?;
            // Only initialize lock for address owned objects.
            if !object.is_child_object() {
                self.lock_service
                    .initialize_locks(&[object_ref], false /* is_force_reset */)
                    .await?;
            }
        }

        // Update the parent
        self.perpetual_tables
            .parent_sync
            .insert(&object_ref, &object.previous_transaction)?;

        Ok(())
    }

    /// This function is used by the bench.rs script, and should not be used in other contexts
    /// In particular it does not check the old locks before inserting new ones, so the objects
    /// must be new.
    pub async fn bulk_object_insert(&self, objects: &[&Object]) -> SuiResult<()> {
        let batch = self.perpetual_tables.objects.batch();
        let ref_and_objects: Vec<_> = objects
            .iter()
            .map(|o| (o.compute_object_reference(), o))
            .collect();

        batch
            .insert_batch(
                &self.perpetual_tables.objects,
                ref_and_objects
                    .iter()
                    .map(|(oref, o)| (ObjectKey::from(oref), **o)),
            )?
            .insert_batch(
                &self.perpetual_tables.owner_index,
                ref_and_objects
                    .iter()
                    .map(|(oref, o)| ((o.owner, oref.0), ObjectInfo::new(oref, o))),
            )?
            .insert_batch(
                &self.perpetual_tables.parent_sync,
                ref_and_objects
                    .iter()
                    .map(|(oref, o)| (oref, o.previous_transaction)),
            )?
            .write()?;

        let non_child_object_refs: Vec<_> = ref_and_objects
            .iter()
            .filter(|(_, object)| !object.is_child_object())
            .map(|(oref, _)| *oref)
            .collect();
        self.lock_service
            .initialize_locks(&non_child_object_refs, false /* is_force_reset */)
            .await?;

        Ok(())
    }

    /// Acquires the transaction lock for a specific transaction, writing the transaction
    /// to the transaction column family if acquiring the lock succeeds.
    /// The lock service is used to atomically acquire locks.
    pub async fn lock_and_write_transaction(
        &self,
        epoch: EpochId,
        owned_input_objects: &[ObjectRef],
        transaction: VerifiedEnvelope<SenderSignedData, S>,
    ) -> Result<(), SuiError> {
        let tx_digest = *transaction.digest();

        // Acquire the lock on input objects
        self.lock_service
            .acquire_locks(epoch, owned_input_objects.to_owned(), tx_digest)
            .await?;

        // TODO: we should have transaction insertion be atomic with lock acquisition, or retry.
        // For now write transactions after because if we write before, there is a chance the lock can fail
        // and this can cause invalid transactions to be inserted in the table.
        // https://github.com/MystenLabs/sui/issues/1990
        self.epoch_tables()
            .transactions
            .insert(&tx_digest, transaction.serializable_ref())?;

        Ok(())
    }

    /// This function should only be used by the gateway.
    /// It's called when we could not get a transaction to successfully execute,
    /// and have to roll back.
    pub async fn reset_transaction_lock(&self, owned_input_objects: &[ObjectRef]) -> SuiResult {
        // this object should not be a child object, since child objects can no longer be
        // inputs, but
        // TODO double check these are not child objects
        self.lock_service
            .initialize_locks(owned_input_objects, true /* is_force_reset */)
            .await?;
        Ok(())
    }

    /// Updates the state resulting from the execution of a certificate.
    ///
    /// Internally it checks that all locks for active inputs are at the correct
    /// version, and then writes locks, objects, certificates, parents atomically.
    pub async fn update_state(
        &self,
        inner_temporary_store: InnerTemporaryStore,
        certificate: &VerifiedCertificate,
        proposed_seq: TxSequenceNumber,
        effects: &TransactionEffectsEnvelope<S>,
        effects_digest: &TransactionEffectsDigest,
    ) -> SuiResult<TxSequenceNumber> {
        // Extract the new state from the execution
        // TODO: events are already stored in the TxDigest -> TransactionEffects store. Is that enough?
        let mut write_batch = self.perpetual_tables.certificates.batch();

        // Store the certificate indexed by transaction digest
        let transaction_digest: &TransactionDigest = certificate.digest();
        write_batch = write_batch.insert_batch(
            &self.perpetual_tables.certificates,
            iter::once((transaction_digest, certificate.serializable_ref())),
        )?;

        let seq = self
            .sequence_tx(
                write_batch,
                inner_temporary_store,
                transaction_digest,
                proposed_seq,
                effects,
                effects_digest,
            )
            .await?;

        self.effects_notify_read
            .notify(transaction_digest, effects.data());

        Ok(seq)
    }

    /// Persist temporary storage to DB for genesis modules
    pub async fn update_objects_state_for_genesis(
        &self,
        inner_temporary_store: InnerTemporaryStore,
        transaction_digest: TransactionDigest,
    ) -> Result<(), SuiError> {
        debug_assert_eq!(transaction_digest, TransactionDigest::genesis());
        let write_batch = self.perpetual_tables.certificates.batch();
        self.batch_update_objects(
            write_batch,
            inner_temporary_store,
            transaction_digest,
            UpdateType::Genesis,
        )
        .await?;
        Ok(())
    }

    /// This is used by the Gateway to update its local store after a transaction succeeded
    /// on the authorities. Since we don't yet have local execution on the gateway, we will
    /// need to recreate the temporary store based on the inputs and effects to update it properly.
    pub async fn update_gateway_state(
        &self,
        input_objects: InputObjects,
        mutated_objects: BTreeMap<ObjectRef, (Object, WriteKind)>,
        certificate: VerifiedCertificate,
        proposed_seq: TxSequenceNumber,
        effects: TransactionEffectsEnvelope<S>,
        effects_digest: &TransactionEffectsDigest,
    ) -> SuiResult {
        let ctx = SingleTxContext::gateway(certificate.sender_address());
        let transaction_digest = certificate.digest();
        let mut temporary_store =
            TemporaryStore::new(Arc::new(&self), input_objects, *transaction_digest);
        for (_, (object, kind)) in mutated_objects {
            temporary_store.write_object(&ctx, object, kind);
        }
        for obj_ref in &effects.data().deleted {
            temporary_store.delete_object(&ctx, &obj_ref.0, obj_ref.1, DeleteKind::Normal);
        }
        for obj_ref in &effects.data().wrapped {
            temporary_store.delete_object(&ctx, &obj_ref.0, obj_ref.1, DeleteKind::Wrap);
        }
        let (inner_temporary_store, _events) = temporary_store.into_inner();

        let mut write_batch = self.perpetual_tables.certificates.batch();
        // Store the certificate indexed by transaction digest
        write_batch = write_batch.insert_batch(
            &self.perpetual_tables.certificates,
            iter::once((transaction_digest, certificate.serializable_ref())),
        )?;
        self.sequence_tx(
            write_batch,
            inner_temporary_store,
            transaction_digest,
            proposed_seq,
            &effects,
            effects_digest,
        )
        .await?;
        Ok(())
    }

    async fn sequence_tx(
        &self,
        write_batch: DBBatch,
        inner_temporary_store: InnerTemporaryStore,
        transaction_digest: &TransactionDigest,
        proposed_seq: TxSequenceNumber,
        effects: &TransactionEffectsEnvelope<S>,
        effects_digest: &TransactionEffectsDigest,
    ) -> SuiResult<TxSequenceNumber> {
        // Safe to unwrap since UpdateType::Transaction ensures we get a sequence number back.
        let assigned_seq = self
            .batch_update_objects(
                write_batch,
                inner_temporary_store,
                *transaction_digest,
                UpdateType::Transaction(proposed_seq, *effects_digest),
            )
            .await?
            .unwrap();

        // Store the signed effects of the transaction
        // We can't write this until after sequencing succeeds (which happens in
        // batch_update_objects), as effects_exists is used as a check in many places
        // for "did the tx finish".
        let batch = self.perpetual_tables.effects.batch();
        let batch = batch.insert_batch(
            &self.perpetual_tables.effects,
            [(transaction_digest, effects)].into_iter(),
        )?;

        // Writing to executed_sequence must be done *after* writing to effects, so that we never
        // broadcast a sequenced transaction (via the batch system) for which no effects can be
        // retrieved.
        //
        // Currently we write both effects and executed_sequence in the same batch to avoid
        // consistency issues between the two (see #4395 for more details).
        //
        // Note that this write may be done repeatedly when retrying a tx. The
        // sequence_transaction call in batch_update_objects assigns a sequence number to
        // the transaction the first time it is called and will return that same sequence
        // on subsequent calls.
        trace!(
            ?assigned_seq,
            tx_digest = ?transaction_digest,
            ?effects_digest,
            "storing sequence number to executed_sequence"
        );
        let batch = batch.insert_batch(
            &self.perpetual_tables.executed_sequence,
            [(
                assigned_seq,
                ExecutionDigests::new(*transaction_digest, *effects_digest),
            )]
            .into_iter(),
        )?;

        batch.write()?;

        Ok(assigned_seq)
    }

    /// Helper function for updating the objects in the state
    async fn batch_update_objects(
        &self,
        mut write_batch: DBBatch,
        inner_temporary_store: InnerTemporaryStore,
        transaction_digest: TransactionDigest,
        update_type: UpdateType,
    ) -> SuiResult<Option<TxSequenceNumber>> {
        let InnerTemporaryStore {
            objects,
            mutable_inputs: active_inputs,
            written,
            deleted,
        } = inner_temporary_store;
        trace!(written =? written.values().map(|((obj_id, ver, _), _, _)| (obj_id, ver)).collect::<Vec<_>>(),
               "batch_update_objects: temp store written");

        let owned_inputs: Vec<_> = active_inputs
            .iter()
            .filter(|(id, _, _)| objects.get(id).unwrap().is_address_owned())
            .cloned()
            .collect();

        // Make an iterator over all objects that are either deleted or have changed owner,
        // along with their old owner.  This is used to update the owner index.
        // For wrapped objects, although their owners technically didn't change, we will lose track
        // of them and there is no guarantee on their owner in the future. Hence we treat them
        // the same as deleted.
        let old_object_owners =
            deleted
                .iter()
                // We need to call get() on objects because some object that were just deleted may not
                // be in the objects list. This can happen if these deleted objects were wrapped in the past,
                // and hence will not show up in the input objects.
                .filter_map(|(id, _)| objects.get(id).and_then(Object::get_owner_and_id))
                .chain(written.iter().filter_map(
                    |(id, (_, new_object, _))| match objects.get(id) {
                        Some(old_object) if old_object.owner != new_object.owner => {
                            old_object.get_owner_and_id()
                        }
                        _ => None,
                    },
                ));

        // Delete the old owner index entries
        write_batch =
            write_batch.delete_batch(&self.perpetual_tables.owner_index, old_object_owners)?;

        // Index the certificate by the objects mutated
        write_batch = write_batch.insert_batch(
            &self.perpetual_tables.parent_sync,
            written
                .iter()
                .map(|(_, (object_ref, _object, _kind))| (object_ref, transaction_digest)),
        )?;

        // Index the certificate by the objects deleted
        write_batch = write_batch.insert_batch(
            &self.perpetual_tables.parent_sync,
            deleted.iter().map(|(object_id, (version, kind))| {
                (
                    (
                        *object_id,
                        *version,
                        if kind == &DeleteKind::Wrap {
                            ObjectDigest::OBJECT_DIGEST_WRAPPED
                        } else {
                            ObjectDigest::OBJECT_DIGEST_DELETED
                        },
                    ),
                    transaction_digest,
                )
            }),
        )?;

        // Once a transaction is done processing and effects committed, we no longer
        // need it in the transactions table. This also allows us to track pending
        // transactions.
        self.epoch_tables()
            .transactions
            .remove(&transaction_digest)?;

        // Update the indexes of the objects written
        write_batch = write_batch.insert_batch(
            &self.perpetual_tables.owner_index,
            written
                .iter()
                .filter_map(|(_id, (object_ref, new_object, _kind))| {
                    trace!(?object_ref, owner =? new_object.owner, "Updating owner_index");
                    new_object
                        .get_owner_and_id()
                        .map(|owner_id| (owner_id, ObjectInfo::new(object_ref, new_object)))
                }),
        )?;

        // Insert each output object into the stores
        write_batch = write_batch.insert_batch(
            &self.perpetual_tables.objects,
            written
                .iter()
                .map(|(_, (obj_ref, new_object, _kind))| (ObjectKey::from(obj_ref), new_object)),
        )?;

        // Atomic write of all data other than locks
        write_batch.write()?;
        trace!("Finished writing batch");

        // Need to have a critical section for now because we need to prevent execution of older
        // certs which may overwrite newer objects with older ones.  This can be removed once we have
        // an object storage supporting multiple object versions at once, then there is idempotency and
        // old writes would be OK.
        let assigned_seq = {
            // Acquire the lock to ensure no one else writes when we are in here.
            let _mutexes = self.acquire_locks(&owned_inputs[..]).await;

            // NOTE: We just check here that locks exist, not that they are locked to a specific TX.  Why?
            // 1. Lock existence prevents re-execution of old certs when objects have been upgraded
            // 2. Not all validators lock, just 2f+1, so transaction should proceed regardless
            //    (But the lock should exist which means previous transactions finished)
            // 3. Equivocation possible (different TX) but as long as 2f+1 approves current TX its fine
            // 4. Locks may have existed when we started processing this tx, but could have since
            //    been deleted by a concurrent tx that finished first. In that case, check if the tx effects exist.
            let new_locks_to_init: Vec<_> = written
                .iter()
                .filter_map(|(_, (object_ref, new_object, _kind))| {
                    if new_object.is_address_owned() {
                        Some(*object_ref)
                    } else {
                        None
                    }
                })
                .collect();

            match update_type {
                UpdateType::Transaction(seq, _) => {
                    // sequence_transaction atomically assigns a sequence number to the tx and
                    // initializes locks for the output objects.
                    // It also (not atomically) deletes the locks for input objects.
                    // After this call completes, new txes can run on the output locks, so all
                    // output objects must be written already.
                    Some(
                        self.lock_service
                            .sequence_transaction(
                                transaction_digest,
                                seq,
                                owned_inputs,
                                new_locks_to_init,
                            )
                            .await?,
                    )
                }
                UpdateType::Genesis => {
                    info!("Creating locks for genesis objects");
                    self.lock_service
                        .create_locks_for_genesis_objects(new_locks_to_init)
                        .await?;
                    None
                }
            }

            // implicit: drop(_mutexes);
        };

        Ok(assigned_seq)
    }

    /// This function is called at the end of epoch for each transaction that's
    /// executed locally on the validator but didn't make to the last checkpoint.
    /// The effects of the execution is reverted here.
    /// The following things are reverted:
    /// 1. Certificate and effects are deleted.
    /// 2. Latest parent_sync entries for each mutated object are deleted.
    /// 3. All new object states are deleted.
    /// 4. owner_index table change is reverted.
    pub fn revert_state_update(&self, tx_digest: &TransactionDigest) -> SuiResult {
        let effects = self.get_effects(tx_digest)?;
        let mut write_batch = self.perpetual_tables.certificates.batch();
        write_batch =
            write_batch.delete_batch(&self.perpetual_tables.certificates, iter::once(tx_digest))?;
        write_batch =
            write_batch.delete_batch(&self.perpetual_tables.effects, iter::once(tx_digest))?;

        let all_new_refs = effects
            .mutated
            .iter()
            .chain(effects.created.iter())
            .chain(effects.unwrapped.iter())
            .map(|(r, _)| r)
            .chain(effects.deleted.iter())
            .chain(effects.wrapped.iter());
        write_batch = write_batch.delete_batch(&self.perpetual_tables.parent_sync, all_new_refs)?;

        let all_new_object_keys = effects
            .mutated
            .iter()
            .chain(effects.created.iter())
            .chain(effects.unwrapped.iter())
            .map(|((id, version, _), _)| ObjectKey(*id, *version));
        write_batch =
            write_batch.delete_batch(&self.perpetual_tables.objects, all_new_object_keys)?;

        // Reverting the change to the owner_index table is most complex.
        // For each newly created (i.e. created and unwrapped) object, the entry in owner_index
        // needs to be deleted; for each mutated object, we need to query the object state of
        // the older version, and then rewrite the entry with the old object info.
        // TODO: Validators should not need to maintain owner_index.
        // This is dependent on https://github.com/MystenLabs/sui/issues/2629.
        let owners_to_delete = effects
            .created
            .iter()
            .chain(effects.unwrapped.iter())
            .chain(effects.mutated.iter())
            .map(|((id, _, _), owner)| (*owner, *id));
        write_batch =
            write_batch.delete_batch(&self.perpetual_tables.owner_index, owners_to_delete)?;
        let mutated_objects = effects
            .mutated
            .iter()
            .map(|(r, _)| r)
            .chain(effects.deleted.iter())
            .chain(effects.wrapped.iter())
            .map(|(id, version, _)| {
                ObjectKey(
                    *id,
                    version
                        .decrement()
                        .expect("version revert should never fail"),
                )
            });
        let old_objects = self
            .perpetual_tables
            .objects
            .multi_get(mutated_objects)?
            .into_iter()
            .map(|obj_opt| {
                let obj = obj_opt.expect("Older object version not found");
                (
                    (obj.owner, obj.id()),
                    ObjectInfo::new(&obj.compute_object_reference(), &obj),
                )
            });
        write_batch = write_batch.insert_batch(&self.perpetual_tables.owner_index, old_objects)?;

        write_batch.write()?;
        Ok(())
    }

    /// Returns the last entry we have for this object in the parents_sync index used
    /// to facilitate client and authority sync. In turn the latest entry provides the
    /// latest object_reference, and also the latest transaction that has interacted with
    /// this object.
    ///
    /// This parent_sync index also contains entries for deleted objects (with a digest of
    /// ObjectDigest::deleted()), and provides the transaction digest of the certificate
    /// that deleted the object. Note that a deleted object may re-appear if the deletion
    /// was the result of the object being wrapped in another object.
    ///
    /// If no entry for the object_id is found, return None.
    pub fn get_latest_parent_entry(
        &self,
        object_id: ObjectID,
    ) -> Result<Option<(ObjectRef, TransactionDigest)>, SuiError> {
        self.perpetual_tables.get_latest_parent_entry(object_id)
    }

    /// Lock a sequence number for the shared objects of the input transaction based on the effects
    /// of that transaction. Used by the nodes, which don't listen to consensus.
    pub fn acquire_shared_locks_from_effects(
        &self,
        certificate: &VerifiedCertificate,
        effects: &TransactionEffects,
        // Do not remove unused arg - ensures that this function is not called without holding a
        // lock.
        _tx_guard: &CertTxGuard<'_>,
    ) -> SuiResult {
        let digest = *certificate.digest();

        let sequenced: Vec<_> = effects
            .shared_objects
            .iter()
            .map(|(id, version, _)| ((digest, *id), *version))
            .collect();
        debug!(?sequenced, "Shared object locks sequenced from effects");

        let mut write_batch = self.epoch_tables().assigned_object_versions.batch();
        write_batch =
            write_batch.insert_batch(&self.epoch_tables().assigned_object_versions, sequenced)?;
        write_batch.write()?;

        Ok(())
    }

    pub fn consensus_message_processed(
        &self,
        digest: &TransactionDigest,
    ) -> Result<bool, SuiError> {
        Ok(self
            .epoch_tables()
            .consensus_message_processed
            .contains_key(digest)?)
    }

    pub async fn consensus_message_processed_notify(
        &self,
        digest: &TransactionDigest,
    ) -> Result<(), SuiError> {
        let registration = self.consensus_notify_read.register_one(digest);
        if self
            .epoch_tables()
            .consensus_message_processed
            .contains_key(digest)?
        {
            return Ok(());
        }
        registration.await;
        Ok(())
    }

    /// Caller is responsible to call consensus_message_processed before this method
    pub async fn record_owned_object_cert_from_consensus(
        &self,
        certificate: &VerifiedCertificate,
        consensus_index: ExecutionIndicesWithHash,
    ) -> Result<(), SuiError> {
        let write_batch = self.epoch_tables().last_consensus_index.batch();
        self.finish_consensus_message_process(write_batch, certificate, consensus_index)
    }

    /// Locks a sequence number for the shared objects of the input transaction. Also updates the
    /// last consensus index, consensus_message_processed and pending_certificates tables.
    /// This function must only be called from the consensus task (i.e. from handle_consensus_transaction).
    ///
    /// Caller is responsible to call consensus_message_processed before this method
    pub async fn record_shared_object_cert_from_consensus(
        &self,
        certificate: &VerifiedCertificate,
        consensus_index: ExecutionIndicesWithHash,
    ) -> Result<(), SuiError> {
        // Make an iterator to save the certificate.
        let transaction_digest = *certificate.digest();

        // Make an iterator to update the locks of the transaction's shared objects.
        let ids = certificate.shared_input_objects().map(|(id, _)| id);
        let versions = self.epoch_tables().next_object_versions.multi_get(ids)?;

        let mut sequenced_to_write = Vec::new();
        let mut schedule_to_write = Vec::new();
        for ((id, initial_shared_version), v) in
            certificate.shared_input_objects().zip(versions.iter())
        {
            // On epoch changes, the `next_object_versions` table will be empty, and we rely on
            // parent sync to recover the current version of the object.  However, if an object was
            // previously aware of the object as owned, and it was upgraded to shared, the version
            // in parent sync may be out of date, causing a fork.  In that case, we know that the
            // `initial_shared_version` will be greater than the version in parent sync, and we can
            // use that.  It is the version that the object was shared at, and can be trusted
            // because it has been checked and signed by a quorum of other validators when creating
            // the certificate.
            let version = match v {
                Some(v) => *v,
                None => *initial_shared_version.max(
                    &self
                        // TODO: if we use an eventually consistent object store in the future,
                        // we must make this read strongly consistent somehow!
                        .get_latest_parent_entry(*id)?
                        .map(|(objref, _)| objref.1)
                        .unwrap_or_default(),
                ),
            };
            let next_version = version.increment();

            sequenced_to_write.push(((transaction_digest, *id), version));
            schedule_to_write.push((*id, next_version));
        }

        trace!(tx_digest = ?transaction_digest,
               ?sequenced_to_write, ?schedule_to_write,
               "locking shared objects");

        // Make an iterator to update the last consensus index.

        // Holding _tx_lock avoids the following race:
        // - we check effects_exist, returns false
        // - another task (starting from handle_node_sync_certificate) writes effects,
        //    and then deletes locks from assigned_object_versions
        // - we write to assigned_object versions, re-creating the locks that were just deleted
        // - now it's possible to run a new tx against old versions of the shared objects.
        let _tx_lock = self.acquire_tx_lock(&transaction_digest).await;

        // Note: if we crash here we are not in an inconsistent state since
        //       it is ok to just update the pending list without updating the sequence.

        // Atomically store all elements.
        // TODO: clear the shared object locks per transaction after ensuring consistency.
        let mut write_batch = self.epoch_tables().assigned_object_versions.batch();

        write_batch = write_batch.insert_batch(
            &self.epoch_tables().assigned_object_versions,
            sequenced_to_write,
        )?;

        write_batch = write_batch
            .insert_batch(&self.epoch_tables().next_object_versions, schedule_to_write)?;

        self.finish_consensus_message_process(write_batch, certificate, consensus_index)
    }

    /// When we finish processing certificate from consensus we record this information.
    /// Tables updated:
    ///  * consensus_message_processed - indicate that this certificate was processed by consensus
    ///  * last_consensus_index - records last processed position in consensus stream
    ///  * consensus_message_order - records at what position this transaction was first seen in consensus
    /// Self::consensus_message_processed returns true after this call for given certificate
    fn finish_consensus_message_process(
        &self,
        batch: DBBatch,
        certificate: &VerifiedCertificate,
        consensus_index: ExecutionIndicesWithHash,
    ) -> SuiResult {
        let transaction_digest = *certificate.digest();
        let batch = batch.insert_batch(
            &self.epoch_tables().consensus_message_order,
            [(consensus_index.index.clone(), transaction_digest)],
        )?;
        let batch = batch.insert_batch(
            &self.epoch_tables().last_consensus_index,
            [(LAST_CONSENSUS_INDEX_ADDR, consensus_index)],
        )?;
        let batch = batch.insert_batch(
            &self.epoch_tables().consensus_message_processed,
            [(transaction_digest, true)],
        )?;
        let batch = batch.insert_batch(
            &self.epoch_tables().pending_certificates,
            [(*certificate.digest(), certificate.clone().serializable())],
        )?;
        batch.write()?;
        self.consensus_notify_read.notify(certificate.digest(), &());
        Ok(())
    }

    /// Returns transaction digests from consensus_message_order table in the "checkpoint range".
    ///
    /// Checkpoint range is defined from the last seen checkpoint(excluded) to the provided
    /// to_height (included)
    pub fn last_checkpoint(
        &self,
        to_height_included: u64,
    ) -> SuiResult<Option<(u64, Vec<TransactionDigest>)>> {
        let epoch_tables = self.epoch_tables();

        let Some((index, from_height_excluded)) = epoch_tables
            .checkpoint_boundary
            .iter()
            .skip_to_last()
            .next() else {
            return Ok(None);
        };
        if from_height_excluded >= to_height_included {
            // Due to crash recovery we might enter this function twice for same boundary
            debug!("Not returning last checkpoint - already processed");
            return Ok(None);
        }

        let iter = epoch_tables.consensus_message_order.iter();
        let last_previous = ExecutionIndices::end_for_commit(from_height_excluded);
        let iter = iter.skip_to(&last_previous)?;
        // skip_to lands to key the last_key or key after it
        // technically here we need to check if first item in stream has a key equal to last_previous
        // however in practice this can not happen because number of batches in certificate is
        // limited and is less then u64::MAX
        let roots = iter
            .take_while(|(idx, _tx)| idx.next_certificate_index <= to_height_included)
            .map(|(_idx, tx)| tx)
            .collect();

        Ok(Some((index, roots)))
    }

    pub fn record_checkpoint_boundary(&self, commit_round: u64) -> SuiResult {
        if let Some((index, height)) = self
            .epoch_tables()
            .checkpoint_boundary
            .iter()
            .skip_to_last()
            .next()
        {
            if height >= commit_round {
                // Due to crash recovery we might see same boundary twice
                debug!("Not recording checkpoint boundary - already updated");
            } else {
                let index = index + 1;
                debug!(
                    "Recording checkpoint boundary {} at {}",
                    index, commit_round
                );
                self.epoch_tables()
                    .checkpoint_boundary
                    .insert(&index, &commit_round)?;
            }
        } else {
            // Table is empty
            debug!("Recording first checkpoint boundary at {}", commit_round);
            self.epoch_tables()
                .checkpoint_boundary
                .insert(&0, &commit_round)?;
        }
        Ok(())
    }

    pub fn transactions_in_seq_range(
        &self,
        start: u64,
        end: u64,
    ) -> SuiResult<Vec<(u64, ExecutionDigests)>> {
        Ok(self
            .perpetual_tables
            .executed_sequence
            .iter()
            .skip_to(&start)?
            .take_while(|(seq, _tx)| *seq < end)
            .collect())
    }

    /// Return the latest consensus index. It is used to bootstrap the consensus client.
    pub fn last_consensus_index(&self) -> SuiResult<ExecutionIndicesWithHash> {
        self.epoch_tables()
            .last_consensus_index
            .get(&LAST_CONSENSUS_INDEX_ADDR)
            .map(|x| x.unwrap_or_default())
            .map_err(SuiError::from)
    }

    pub fn get_transaction(
        &self,
        transaction_digest: &TransactionDigest,
    ) -> SuiResult<Option<VerifiedEnvelope<SenderSignedData, S>>> {
        let transaction = self.epoch_tables().transactions.get(transaction_digest)?;
        Ok(transaction.map(|t| t.into()))
    }

    pub fn get_certified_transaction(
        &self,
        transaction_digest: &TransactionDigest,
    ) -> SuiResult<Option<VerifiedCertificate>> {
        let transaction = self.perpetual_tables.certificates.get(transaction_digest)?;
        Ok(transaction.map(|t| t.into()))
    }

    pub fn multi_get_certified_transaction(
        &self,
        transaction_digests: &[TransactionDigest],
    ) -> SuiResult<Vec<Option<VerifiedCertificate>>> {
        Ok(self
            .perpetual_tables
            .certificates
            .multi_get(transaction_digests)?
            .into_iter()
            .map(|o| o.map(|c| c.into()))
            .collect())
    }

    pub fn get_sui_system_state_object(&self) -> SuiResult<SuiSystemState>
    where
        S: Eq + Serialize + for<'de> Deserialize<'de>,
    {
        self.perpetual_tables.get_sui_system_state_object()
    }
}

impl SuiDataStore<AuthoritySignInfo> {
    /// Returns true if we have a transaction structure for this transaction digest
    pub fn transaction_exists(
        &self,
        cur_epoch: EpochId,
        transaction_digest: &TransactionDigest,
    ) -> SuiResult<bool> {
        let tx: Option<VerifiedSignedTransaction> = self
            .epoch_tables()
            .transactions
            .get(transaction_digest)?
            .map(|t| t.into());
        Ok(if let Some(signed_tx) = tx {
            signed_tx.epoch() == cur_epoch
        } else {
            false
        })
    }

    pub fn get_signed_transaction_info(
        &self,
        transaction_digest: &TransactionDigest,
    ) -> Result<VerifiedTransactionInfoResponse, SuiError> {
        Ok(VerifiedTransactionInfoResponse {
            signed_transaction: self
                .epoch_tables()
                .transactions
                .get(transaction_digest)?
                .map(|t| t.into()),
            certified_transaction: self
                .perpetual_tables
                .certificates
                .get(transaction_digest)?
                .map(|c| c.into()),
            signed_effects: self.perpetual_tables.effects.get(transaction_digest)?,
        })
    }
}

impl<S: Eq + Debug + Serialize + for<'de> Deserialize<'de>> BackingPackageStore
    for SuiDataStore<S>
{
    fn get_package(&self, package_id: &ObjectID) -> SuiResult<Option<Object>> {
        let package = self.get_object(package_id)?;
        if let Some(obj) = &package {
            fp_ensure!(
                obj.is_package(),
                SuiError::BadObjectType {
                    error: format!("Package expected, Move object found: {package_id}"),
                }
            );
        }
        Ok(package)
    }
}

impl<S: Eq + Debug + Serialize + for<'de> Deserialize<'de>> ChildObjectResolver
    for SuiDataStore<S>
{
    fn read_child_object(&self, parent: &ObjectID, child: &ObjectID) -> SuiResult<Option<Object>> {
        let child_object = match self.get_object(child)? {
            None => return Ok(None),
            Some(o) => o,
        };
        let parent = *parent;
        if child_object.owner != Owner::ObjectOwner(parent.into()) {
            return Err(SuiError::InvalidChildObjectAccess {
                object: *child,
                given_parent: parent,
                actual_owner: child_object.owner,
            });
        }
        Ok(Some(child_object))
    }
}

impl<S: Eq + Debug + Serialize + for<'de> Deserialize<'de>> ParentSync for SuiDataStore<S> {
    fn get_latest_parent_entry_ref(&self, object_id: ObjectID) -> SuiResult<Option<ObjectRef>> {
        Ok(self
            .get_latest_parent_entry(object_id)?
            .map(|(obj_ref, _)| obj_ref))
    }
}

impl<S: Eq + Debug + Serialize + for<'de> Deserialize<'de>> ModuleResolver for SuiDataStore<S> {
    type Error = SuiError;

    // TODO: duplicated code with ModuleResolver for InMemoryStorage in memory_storage.rs.
    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        // TODO: We should cache the deserialized modules to avoid
        // fetching from the store / re-deserializing them everytime.
        // https://github.com/MystenLabs/sui/issues/809
        Ok(self
            .get_package(&ObjectID::from(*module_id.address()))?
            .and_then(|package| {
                // unwrap safe since get_package() ensures it's a package object.
                package
                    .data
                    .try_as_package()
                    .unwrap()
                    .serialized_module_map()
                    .get(module_id.name().as_str())
                    .cloned()
            }))
    }
}

/// A wrapper to make Orphan Rule happy
pub struct ResolverWrapper<T: ModuleResolver>(pub Arc<T>);

impl<T: ModuleResolver> ModuleResolver for ResolverWrapper<T> {
    type Error = T::Error;
    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        self.0.get_module(module_id)
    }
}

// The primary key type for object storage.
#[serde_as]
#[derive(Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Hash, Serialize, Deserialize, Debug)]
pub struct ObjectKey(pub ObjectID, pub VersionNumber);

impl ObjectKey {
    pub const ZERO: ObjectKey = ObjectKey(ObjectID::ZERO, VersionNumber::MIN);

    pub fn max_for_id(id: &ObjectID) -> Self {
        Self(*id, VersionNumber::MAX)
    }
}

impl From<ObjectRef> for ObjectKey {
    fn from(object_ref: ObjectRef) -> Self {
        ObjectKey::from(&object_ref)
    }
}

impl From<&ObjectRef> for ObjectKey {
    fn from(object_ref: &ObjectRef) -> Self {
        Self(object_ref.0, object_ref.1)
    }
}

pub enum UpdateType {
    Transaction(TxSequenceNumber, TransactionEffectsDigest),
    Genesis,
}

pub trait EffectsStore {
    fn get_effects<'a>(
        &self,
        transactions: impl Iterator<Item = &'a TransactionDigest> + Clone,
    ) -> SuiResult<Vec<Option<TransactionEffects>>>;
}

impl EffectsStore for Arc<AuthorityStore> {
    fn get_effects<'a>(
        &self,
        transactions: impl Iterator<Item = &'a TransactionDigest> + Clone,
    ) -> SuiResult<Vec<Option<TransactionEffects>>> {
        Ok(self
            .perpetual_tables
            .effects
            .multi_get(transactions)?
            .into_iter()
            .map(|item| item.map(|x| x.into_data()))
            .collect())
    }
}
