// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::authority::SuiDataStore;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;
use sui_types::base_types::ObjectRef;
use sui_types::messages::TransactionKind;
use sui_types::{
    base_types::{SequenceNumber, SuiAddress},
    error::{SuiError, SuiResult},
    fp_ensure,
    gas::{self, SuiGasStatus},
    messages::{
        InputObjectKind, InputObjects, SingleTransactionKind, TransactionData, VerifiedCertificate,
    },
    object::{Object, Owner},
};
use tracing::instrument;

async fn get_gas_status<S>(
    store: &SuiDataStore<S>,
    transaction: &TransactionData,
) -> SuiResult<SuiGasStatus<'static>>
where
    S: Eq + Debug + Serialize + for<'de> Deserialize<'de>,
{
    let tx_kind = &transaction.kind;
    let gas_object_ref = transaction.gas_payment_object_ref();
    let gas_object_refs = match tx_kind {
        TransactionKind::Single(SingleTransactionKind::PaySui(p)) => p.coins.clone(),
        TransactionKind::Single(SingleTransactionKind::PayAllSui(p)) => p.coins.clone(),
        _ => vec![],
    };
    let extra_gas_object_refs = gas_object_refs.into_iter().skip(1).collect();

    let mut gas_status = check_gas(
        store,
        gas_object_ref,
        transaction.gas_budget,
        transaction.gas_price,
        &transaction.kind,
        extra_gas_object_refs,
    )
    .await?;

    if transaction.contains_shared_object() {
        // It's important that we do this here to make sure there is enough
        // gas to cover shared objects, before we lock all objects.
        gas_status.charge_consensus()?;
    }

    Ok(gas_status)
}

#[instrument(level = "trace", skip_all)]
pub async fn check_transaction_input<S>(
    store: &SuiDataStore<S>,
    transaction: &TransactionData,
) -> SuiResult<(SuiGasStatus<'static>, InputObjects)>
where
    S: Eq + Debug + Serialize + for<'de> Deserialize<'de>,
{
    transaction.validity_check()?;
    transaction.kind.validity_check()?;
    let gas_status = get_gas_status(store, transaction).await?;
    let input_objects = transaction.input_objects()?;
    let objects = store.check_input_objects(&input_objects)?;
    let input_objects = check_objects(transaction, input_objects, objects).await?;
    Ok((gas_status, input_objects))
}

pub async fn check_certificate_input<S>(
    store: &SuiDataStore<S>,
    cert: &VerifiedCertificate,
) -> SuiResult<(SuiGasStatus<'static>, InputObjects)>
where
    S: Eq + Debug + Serialize + for<'de> Deserialize<'de>,
{
    let gas_status = get_gas_status(store, &cert.data().data).await?;
    let input_object_kinds = cert.data().data.input_objects()?;
    let tx_data = &cert.data().data;
    let input_object_data = if tx_data.kind.is_change_epoch_tx() {
        // When changing the epoch, we update a the system object, which is shared, without going
        // through sequencing, so we must bypass the sequence checks here.
        store.check_input_objects(&input_object_kinds)?
    } else {
        store.check_sequenced_input_objects(cert.digest(), &input_object_kinds)?
    };
    let input_objects =
        check_objects(&cert.data().data, input_object_kinds, input_object_data).await?;
    Ok((gas_status, input_objects))
}

/// Checking gas budget by fetching the gas object only from the store,
/// and check whether the balance and budget satisfies the miminum requirement.
/// Returns the gas object (to be able to reuse it latter) and a gas status
/// that will be used in the entire lifecycle of the transaction execution.
#[instrument(level = "trace", skip_all)]
async fn check_gas<S>(
    store: &SuiDataStore<S>,
    gas_payment: &ObjectRef,
    gas_budget: u64,
    computation_gas_price: u64,
    tx_kind: &TransactionKind,
    additional_objects_for_gas_payment: Vec<ObjectRef>,
) -> SuiResult<SuiGasStatus<'static>>
where
    S: Eq + Debug + Serialize + for<'de> Deserialize<'de>,
{
    if tx_kind.is_system_tx() {
        Ok(SuiGasStatus::new_unmetered())
    } else {
        let gas_object = store.get_object_by_key(&gas_payment.0, gas_payment.1)?;
        let gas_object = gas_object.ok_or(SuiError::TransactionInputObjectsErrors {
            errors: vec![SuiError::ObjectNotFound {
                object_id: gas_payment.0,
                version: Some(gas_payment.1),
            }],
        })?;

        // TODO: cache this storage_gas_price in memory
        let storage_gas_price = store
            .get_sui_system_state_object()?
            .parameters
            .storage_gas_price;

        // If the transaction is TransferSui, we ensure that the gas balance is enough to cover
        // both gas budget and the transfer amount.
        let extra_amount = match tx_kind {
            TransactionKind::Single(SingleTransactionKind::TransferSui(t)) => {
                t.amount.unwrap_or_default()
            }
            TransactionKind::Single(SingleTransactionKind::PaySui(t)) => t.amounts.iter().sum(),
            _ => 0,
        };
        // TODO: We should revisit how we compute gas price and compare to gas budget.
        let gas_price = std::cmp::max(computation_gas_price, storage_gas_price);

        if tx_kind.is_pay_sui_tx() {
            let mut additional_objs = vec![];
            for obj_ref in additional_objects_for_gas_payment.iter() {
                let obj = store.get_object_by_key(&obj_ref.0, obj_ref.1)?;
                let obj = obj.ok_or(SuiError::TransactionInputObjectsErrors {
                    errors: vec![SuiError::ObjectNotFound {
                        object_id: gas_payment.0,
                        version: None,
                    }],
                })?;
                additional_objs.push(obj);
            }
            gas::check_gas_balance(
                &gas_object,
                gas_budget,
                gas_price,
                extra_amount,
                additional_objs,
            )?;
        } else {
            gas::check_gas_balance(&gas_object, gas_budget, gas_price, extra_amount, vec![])?;
        }

        let gas_status =
            gas::start_gas_metering(gas_budget, computation_gas_price, storage_gas_price)?;
        Ok(gas_status)
    }
}

/// Check all the objects used in the transaction against the database, and ensure
/// that they are all the correct version and number.
#[instrument(level = "trace", skip_all)]
async fn check_objects(
    transaction: &TransactionData,
    input_objects: Vec<InputObjectKind>,
    objects: Vec<Object>,
) -> Result<InputObjects, SuiError> {
    // We require that mutable objects cannot show up more than once.
    // In [`SingleTransactionKind::input_objects`] we checked that there is no
    // duplicate objects in the same SingleTransactionKind. However for a Batch
    // Transaction, we still need to make sure that the same mutable object don't show
    // up in more than one SingleTransactionKind.
    // TODO: We should be able to allow the same shared object to show up
    // in more than one SingleTransactionKind. We need to ensure that their
    // version number only increases once at the end of the Batch execution.
    let mut used_objects: HashSet<SuiAddress> = HashSet::new();
    for object in objects.iter() {
        if !object.is_immutable() {
            fp_ensure!(
                used_objects.insert(object.id().into()),
                SuiError::InvalidBatchTransaction {
                    error: format!("Mutable object {} cannot appear in more than one single transactions in a batch", object.id()),
                }
            );
        }
    }

    // Gather all objects and errors.
    let mut all_objects = Vec::with_capacity(input_objects.len());
    let mut errors = Vec::new();
    let transfer_object_ids: HashSet<_> = transaction
        .kind
        .single_transactions()
        .filter_map(|s| {
            if let SingleTransactionKind::TransferObject(t) = s {
                Some(t.object_ref.0)
            } else {
                None
            }
        })
        .collect();

    for (object_kind, object) in input_objects.into_iter().zip(objects) {
        if transfer_object_ids.contains(&object.id()) {
            object.ensure_public_transfer_eligible()?;
        }
        // Check if the object contents match the type of lock we need for
        // this object.
        match check_one_object(&transaction.signer(), object_kind, &object) {
            Ok(()) => all_objects.push((object_kind, object)),
            Err(e) => {
                errors.push(e);
            }
        }
    }
    // If any errors with the locks were detected, we return all errors to give the client
    // a chance to update the authority if possible.
    if !errors.is_empty() {
        return Err(SuiError::TransactionInputObjectsErrors { errors });
    }
    fp_ensure!(!all_objects.is_empty(), SuiError::ObjectInputArityViolation);

    Ok(InputObjects::new(all_objects))
}

/// The logic to check one object against a reference, and return the object if all is well
/// or an error if not.
fn check_one_object(
    sender: &SuiAddress,
    object_kind: InputObjectKind,
    object: &Object,
) -> SuiResult {
    match object_kind {
        InputObjectKind::MovePackage(package_id) => {
            fp_ensure!(
                object.data.try_as_package().is_some(),
                SuiError::MoveObjectAsPackage {
                    object_id: package_id
                }
            );
        }
        InputObjectKind::ImmOrOwnedMoveObject((object_id, sequence_number, object_digest)) => {
            fp_ensure!(
                !object.is_package(),
                SuiError::MovePackageAsObject { object_id }
            );
            // wrapped objects that are then deleted will be set to MAX,
            // so we need to cap the sequence number at MAX - 1
            fp_ensure!(
                sequence_number < SequenceNumber::MAX.decrement().unwrap(),
                SuiError::InvalidSequenceNumber
            );

            // Check that the seq number is the same
            // Note that this generally can't fail, because we fetch objects at the version
            // specified by the input objects. This makes check_transaction_input idempotent.
            // A tx that tries to operate on older versions will fail later when checking the
            // object locks.
            fp_ensure!(
                object.version() == sequence_number,
                SuiError::UnexpectedSequenceNumber {
                    object_id,
                    expected_sequence: object.version(),
                    given_sequence: sequence_number,
                }
            );

            // Check the digest matches

            let expected_digest = object.digest();
            fp_ensure!(
                expected_digest == object_digest,
                SuiError::InvalidObjectDigest {
                    object_id,
                    expected_digest
                }
            );

            match object.owner {
                Owner::Immutable => {
                    // Nothing else to check for Immutable.
                }
                Owner::AddressOwner(owner) => {
                    // Check the owner is the transaction sender.
                    fp_ensure!(
                        sender == &owner,
                        SuiError::IncorrectSigner {
                            error: format!("Object {:?} is owned by account address {:?}, but signer address is {:?}", object_id, owner, sender),
                        }
                    );
                }
                Owner::ObjectOwner(owner) => {
                    return Err(SuiError::InvalidChildObjectArgument {
                        child_id: object.id(),
                        parent_id: owner.into(),
                    });
                }
                Owner::Shared { .. } => {
                    // This object is a mutable shared object. However the transaction
                    // specifies it as an owned object. This is inconsistent.
                    return Err(SuiError::NotSharedObjectError);
                }
            };
        }
        InputObjectKind::SharedMoveObject {
            initial_shared_version: input_initial_shared_version,
            ..
        } => {
            fp_ensure!(
                object.version() < SequenceNumber::MAX,
                SuiError::InvalidSequenceNumber
            );

            match object.owner {
                Owner::AddressOwner(_) | Owner::ObjectOwner(_) | Owner::Immutable => {
                    // When someone locks an object as shared it must be shared already.
                    return Err(SuiError::NotSharedObjectError);
                }
                Owner::Shared {
                    initial_shared_version: actual_initial_shared_version,
                } => {
                    fp_ensure!(
                        input_initial_shared_version == actual_initial_shared_version,
                        SuiError::SharedObjectStartingVersionMismatch
                    )
                }
            }
        }
    };
    Ok(())
}
