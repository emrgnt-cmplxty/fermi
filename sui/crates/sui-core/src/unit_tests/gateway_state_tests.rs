// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde_json::json;
use std::{collections::HashSet, path::Path};

use signature::Signer;
use typed_store::Map;

use sui_framework_build::compiled_package::BuildConfig;
use sui_types::crypto::{AccountKeyPair, Signature};
use sui_types::gas_coin::GasCoin;
use sui_types::messages::Transaction;
use sui_types::object::{Object, GAS_VALUE_FOR_TESTING};
use sui_types::{crypto::get_key_pair, object::Owner};

use crate::authority_aggregator::authority_aggregator_tests::{
    crate_object_move_transaction, get_local_client, init_local_authorities,
};
use crate::authority_client::LocalAuthorityClient;
use crate::gateway_state::{GatewayAPI, GatewayState};

use super::*;

async fn create_gateway_state_with_object_basics_ref(
    genesis_objects: Vec<Object>,
) -> (GatewayState<LocalAuthorityClient>, ObjectRef) {
    let all_owners: HashSet<_> = genesis_objects
        .iter()
        .map(|o| o.get_single_owner().unwrap())
        .collect();
    let (authorities, _, pkg_ref) = init_local_authorities(4, genesis_objects).await;
    let path = tempfile::tempdir().unwrap().into_path();
    let gateway_store = Arc::new(GatewayStore::open(&path, None).unwrap());
    let gateway = GatewayState::new_with_authorities(
        gateway_store,
        authorities,
        GatewayMetrics::new_for_tests(),
    )
    .unwrap();
    for owner in all_owners {
        gateway.sync_account_state(owner).await.unwrap();
    }
    (gateway, pkg_ref)
}

async fn create_gateway_state(genesis_objects: Vec<Object>) -> GatewayState<LocalAuthorityClient> {
    create_gateway_state_with_object_basics_ref(genesis_objects)
        .await
        .0
}

async fn public_transfer_object(
    gateway: &GatewayState<LocalAuthorityClient>,
    signer: SuiAddress,
    key: &AccountKeyPair,
    coin_object_id: ObjectID,
    gas_object_id: ObjectID,
    recipient: SuiAddress,
) -> Result<SuiTransactionResponse, anyhow::Error> {
    let data = gateway
        .public_transfer_object(
            signer,
            coin_object_id,
            Some(gas_object_id),
            GAS_VALUE_FOR_TESTING / 10,
            recipient,
        )
        .await?;

    let signature = key.sign(&data.to_bytes());
    let result = gateway
        .execute_transaction(Transaction::from_data(data, signature))
        .await?;
    Ok(result)
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_public_transfer_object() {
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();
    let (addr2, _key2): (_, AccountKeyPair) = get_key_pair();

    let coin_object = Object::with_owner_for_testing(addr1);
    let gas_object = Object::with_owner_for_testing(addr1);

    let genesis_objects = vec![coin_object.clone(), gas_object.clone()];
    let gateway = create_gateway_state(genesis_objects).await;

    let effects = public_transfer_object(
        &gateway,
        addr1,
        &key1,
        coin_object.id(),
        gas_object.id(),
        addr2,
    )
    .await
    .unwrap()
    .effects;
    assert_eq!(effects.mutated.len(), 2);
    assert_eq!(
        effects.mutated_excluding_gas().next().unwrap().owner,
        Owner::AddressOwner(addr2)
    );
    assert_eq!(gateway.get_total_transaction_number().unwrap(), 1);
}

#[tokio::test]
async fn test_move_call() {
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();
    let gas_object = Object::with_owner_for_testing(addr1);
    let genesis_objects = vec![gas_object.clone()];
    let (gateway, pkg_ref) = create_gateway_state_with_object_basics_ref(genesis_objects).await;

    let tx = crate_object_move_transaction(
        addr1,
        &key1,
        addr1,
        100,
        pkg_ref,
        gas_object.compute_object_reference(),
    );

    let effects = gateway
        .execute_transaction(tx.into_inner())
        .await
        .unwrap()
        .effects;
    assert!(effects.status.is_ok());
    assert_eq!(effects.mutated.len(), 1);
    assert_eq!(effects.created.len(), 1);
    assert_eq!(effects.created[0].owner, Owner::AddressOwner(addr1));
}

#[tokio::test]
async fn test_publish() {
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();
    let gas_object = Object::with_owner_for_testing(addr1);
    let genesis_objects = vec![gas_object.clone()];
    let gateway = create_gateway_state(genesis_objects).await;

    // Provide path to well formed package sources
    let mut path = env!("CARGO_MANIFEST_DIR").to_owned();
    path.push_str("/src/unit_tests/data/object_owner/");

    let compiled_modules = BuildConfig::default()
        .build(Path::new(&path).to_path_buf())
        .unwrap()
        .get_package_bytes();
    let data = gateway
        .publish(
            addr1,
            compiled_modules,
            Some(gas_object.id()),
            GAS_VALUE_FOR_TESTING,
        )
        .await
        .unwrap();

    let signature = key1.sign(&data.to_bytes());
    gateway
        .execute_transaction(Transaction::from_data(data, signature))
        .await
        .unwrap();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_coin_split() {
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();

    let coin_object = Object::with_owner_for_testing(addr1);
    let gas_object = Object::with_owner_for_testing(addr1);

    let genesis_objects = vec![coin_object.clone(), gas_object.clone()];
    let gateway = create_gateway_state(genesis_objects).await;

    let split_amounts = vec![100, 200, 300, 400, 500];
    let total_amount: u64 = split_amounts.iter().sum();

    let data = gateway
        .split_coin(
            addr1,
            coin_object.id(),
            split_amounts.clone(),
            Some(gas_object.id()),
            GAS_VALUE_FOR_TESTING,
        )
        .await
        .unwrap();

    let signature = key1.sign(&data.to_bytes());
    let response = gateway
        .execute_transaction(Transaction::from_data(data, signature))
        .await
        .unwrap()
        .parsed_data
        .unwrap()
        .to_split_coin_response()
        .unwrap();

    assert_eq!(
        (coin_object.id(), coin_object.version().increment()),
        (response.updated_coin.id(), response.updated_coin.version())
    );
    assert_eq!(
        (gas_object.id(), gas_object.version().increment()),
        (response.updated_gas.id(), response.updated_gas.version())
    );
    let update_coin = GasCoin::try_from(&response.updated_coin).unwrap();
    assert_eq!(update_coin.value(), GAS_VALUE_FOR_TESTING - total_amount);
    let split_coin_values = response
        .new_coins
        .iter()
        .map(|o| GasCoin::try_from(o).unwrap().value())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        split_amounts,
        split_coin_values.into_iter().collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_coin_split_insufficient_gas() {
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();

    let coin_object = Object::with_owner_for_testing(addr1);
    let gas_object = Object::with_owner_for_testing(addr1);

    let genesis_objects = vec![coin_object.clone(), gas_object.clone()];
    let gateway = create_gateway_state(genesis_objects).await;

    let split_amounts = vec![100, 200, 300, 400, 500];

    let data = gateway
        .split_coin(
            addr1,
            coin_object.id(),
            split_amounts.clone(),
            Some(gas_object.id()),
            9, /* Insufficient gas */
        )
        .await
        .unwrap();

    let signature = key1.sign(&data.to_bytes());
    let response = gateway
        .execute_transaction(Transaction::from_data(data, signature))
        .await;
    // Tx should fail due to out of gas, and no transactions should remain pending.
    // Objects are not locked either.
    assert!(response.is_err());
    assert_eq!(
        gateway.store().epoch_tables().transactions.iter().count(),
        0
    );
    assert_eq!(
        gateway
            .store()
            .get_object_locking_transaction(&gas_object.compute_object_reference())
            .await
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn test_coin_merge() {
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();

    let coin_object1 = Object::with_owner_for_testing(addr1);
    let coin_object2 = Object::with_owner_for_testing(addr1);
    let gas_object = Object::with_owner_for_testing(addr1);
    let genesis_objects = vec![
        coin_object1.clone(),
        coin_object2.clone(),
        gas_object.clone(),
    ];
    let gateway = create_gateway_state(genesis_objects).await;

    let data = gateway
        .merge_coins(
            addr1,
            coin_object1.id(),
            coin_object2.id(),
            Some(gas_object.id()),
            GAS_VALUE_FOR_TESTING,
        )
        .await
        .unwrap();

    let signature = key1.sign(&data.to_bytes());
    let response = gateway
        .execute_transaction(Transaction::from_data(data, signature))
        .await
        .unwrap()
        .parsed_data
        .unwrap()
        .to_merge_coin_response()
        .unwrap();

    assert_eq!(
        (coin_object1.id(), coin_object1.version().increment()),
        (response.updated_coin.id(), response.updated_coin.version())
    );
    assert_eq!(
        (gas_object.id(), gas_object.version().increment()),
        (response.updated_gas.id(), response.updated_gas.version())
    );
    let update_coin = GasCoin::try_from(&response.updated_coin).unwrap();
    assert_eq!(update_coin.value(), GAS_VALUE_FOR_TESTING * 2);
}

#[tokio::test]
async fn test_pay_sui_empty_input_coins() -> Result<(), anyhow::Error> {
    let (addr1, _): (_, AccountKeyPair) = get_key_pair();
    let (recipient, _): (_, AccountKeyPair) = get_key_pair();
    let coin_object = Object::with_owner_for_testing(addr1);
    let genesis_objects = vec![coin_object.clone()];
    let gateway = create_gateway_state(genesis_objects).await;

    let res = gateway
        .pay_sui(addr1, vec![], vec![recipient], vec![100], 1000)
        .await;
    assert!(res.is_err());
    Ok(())
}

#[tokio::test]
async fn test_pay_all_sui_empty_input_coins() -> Result<(), anyhow::Error> {
    let (addr1, _): (_, AccountKeyPair) = get_key_pair();
    let (recipient, _): (_, AccountKeyPair) = get_key_pair();
    let coin_object = Object::with_owner_for_testing(addr1);
    let genesis_objects = vec![coin_object.clone()];
    let gateway = create_gateway_state(genesis_objects).await;

    let res = gateway.pay_all_sui(addr1, vec![], recipient, 1000).await;
    assert!(res.is_err());
    Ok(())
}

#[tokio::test]
async fn test_equivocation_resilient() {
    telemetry_subscribers::init_for_testing();
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();
    let coin_object = Object::with_owner_for_testing(addr1);
    let genesis_objects = vec![coin_object.clone()];
    let gateway = Arc::new(Box::new(create_gateway_state(genesis_objects).await));

    let mut handles = vec![];
    // We create 20 requests that try to touch the same object to the gateway.
    // Make sure that one of them succeeds and there are no pending tx in the end.
    for _ in 0..20 {
        let (recipient, _): (_, AccountKeyPair) = get_key_pair();
        let data = TransactionData::new_transfer_sui(
            recipient,
            addr1,
            None,
            coin_object.compute_object_reference(),
            1000,
        );
        let signature: Signature = key1.sign(&data.to_bytes());
        let handle = tokio::task::spawn({
            let gateway_copy = gateway.clone();
            async move {
                gateway_copy
                    .execute_transaction(Transaction::from_data(data, signature))
                    .await
            }
        });
        handles.push(handle);
    }
    let results = futures::future::join_all(handles).await;
    assert_eq!(
        results
            .into_iter()
            .filter(|r| r.as_ref().unwrap().is_ok())
            .count(),
        1
    );
    println!(
        "{:?}",
        gateway.store().epoch_tables().transactions.iter().next()
    );
    assert_eq!(
        gateway.store().epoch_tables().transactions.iter().count(),
        0
    );
}

#[tokio::test]
async fn test_public_transfer_object_with_retry() {
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();
    let (addr2, _key2): (_, AccountKeyPair) = get_key_pair();

    let coin_object = Object::with_owner_for_testing(addr1);
    let gas_object = Object::with_owner_for_testing(addr1);

    let genesis_objects = vec![coin_object.clone(), gas_object.clone()];
    let mut gateway = create_gateway_state(genesis_objects).await;
    // Make two authorities fail at the end of certificate processing.
    get_local_client(&mut gateway.authorities, 0)
        .fault_config
        .fail_after_handle_confirmation = true;
    get_local_client(&mut gateway.authorities, 1)
        .fault_config
        .fail_after_handle_confirmation = true;

    // Transfer will fail because we would not be able to reach quorum on cert processing.
    assert!(public_transfer_object(
        &gateway,
        addr1,
        &key1,
        coin_object.id(),
        gas_object.id(),
        addr2,
    )
    .await
    .is_err());

    // Since we never finished executing the transaction, the transaction is still in the
    // transactions table.
    // However objects in the transaction should no longer be locked since we reset
    // them at the last failed retry.
    assert_eq!(
        gateway.store().epoch_tables().transactions.iter().count(),
        1
    );
    let (tx_digest, tx) = gateway
        .store()
        .epoch_tables()
        .transactions
        .iter()
        .next()
        .unwrap();
    assert_eq!(
        gateway
            .store()
            .get_object_locking_transaction(&coin_object.compute_object_reference())
            .await
            .unwrap(),
        None,
    );

    // Recover one of the authorities.
    get_local_client(&mut gateway.authorities, 1)
        .fault_config
        .fail_after_handle_confirmation = false;

    // Retry transaction, and this time it should succeed.
    let effects = gateway
        .execute_transaction(tx.into_inner())
        .await
        .unwrap()
        .effects;
    let oref = effects.mutated_excluding_gas().next().unwrap();
    let updated_obj_ref = &oref.reference;
    let new_owner = &oref.owner;
    assert_eq!(new_owner, &Owner::AddressOwner(addr2));

    assert_eq!(
        gateway.store().epoch_tables().transactions.iter().count(),
        0
    );
    assert!(gateway
        .store()
        .get_object_locking_transaction(&coin_object.compute_object_reference())
        .await
        .is_err());
    assert!(gateway.store().effects_exists(&tx_digest).unwrap());
    // The transaction is deleted after this is done.
    assert!(gateway
        .store()
        .get_transaction(&tx_digest)
        .unwrap()
        .is_none());
    assert_eq!(gateway.store().next_sequence_number().unwrap(), 1);
    assert_eq!(
        gateway
            .store()
            .get_owner_objects(Owner::AddressOwner(addr1))
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        gateway
            .store()
            .get_owner_objects(Owner::AddressOwner(addr2))
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        gateway
            .store()
            .read_certificate(&tx_digest)
            .unwrap()
            .unwrap()
            .digest(),
        &tx_digest
    );
    assert_eq!(
        gateway
            .store()
            .parent(&updated_obj_ref.to_object_ref())
            .unwrap()
            .unwrap(),
        tx_digest
    );
}

#[tokio::test]
async fn test_get_owner_object() {
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();
    let gas_object = Object::with_owner_for_testing(addr1);
    let genesis_objects = vec![gas_object.clone()];
    let gateway = create_gateway_state(genesis_objects).await;

    // Provide path to well formed package sources
    let mut path = env!("CARGO_MANIFEST_DIR").to_owned();
    path.push_str("/src/unit_tests/data/object_owner/");

    // Publish object_owner package
    let compiled_modules = BuildConfig::default()
        .build(Path::new(&path).to_path_buf())
        .unwrap()
        .get_package_bytes();
    let data = gateway
        .publish(
            addr1,
            compiled_modules,
            Some(gas_object.id()),
            GAS_VALUE_FOR_TESTING,
        )
        .await
        .unwrap();

    let signature = key1.sign(&data.to_bytes());
    let response = gateway
        .execute_transaction(Transaction::from_data(data, signature))
        .await
        .unwrap()
        .parsed_data
        .unwrap()
        .to_publish_response()
        .unwrap();

    // create parent and child object
    let package = response.package.object_id;
    let data = gateway
        .move_call(
            addr1,
            package,
            "object_owner".to_string(),
            "create_parent".to_string(),
            vec![],
            vec![],
            None,
            10000,
        )
        .await
        .unwrap();
    let signature = key1.sign(&data.to_bytes());
    let response = gateway
        .execute_transaction(Transaction::from_data(data, signature))
        .await
        .unwrap();
    let parent = &response.effects.created.first().unwrap().reference;
    let data = gateway
        .move_call(
            addr1,
            package,
            "object_owner".to_string(),
            "create_child".to_string(),
            vec![],
            vec![],
            None,
            10000,
        )
        .await
        .unwrap();
    let signature = key1.sign(&data.to_bytes());
    let response = gateway
        .execute_transaction(Transaction::from_data(data, signature))
        .await
        .unwrap();
    let child = &response.effects.created.first().unwrap().reference;

    // Make parent owns child
    let data = gateway
        .move_call(
            addr1,
            package,
            "object_owner".to_string(),
            "add_child".to_string(),
            vec![],
            vec![
                SuiJsonValue::new(json!(parent.object_id.to_hex_literal())).unwrap(),
                SuiJsonValue::new(json!(child.object_id.to_hex_literal())).unwrap(),
            ],
            None,
            10000,
        )
        .await
        .unwrap();
    let signature = key1.sign(&data.to_bytes());
    let response = gateway
        .execute_transaction(Transaction::from_data(data, signature))
        .await
        .unwrap();
    let field_object = &response.effects.created.first().unwrap().reference;

    // Query get_objects_owned_by_object
    let objects = gateway
        .get_objects_owned_by_object(parent.object_id)
        .await
        .unwrap();
    assert_eq!(1, objects.len());
    assert_eq!(field_object.object_id, objects.first().unwrap().object_id);
    let objects = gateway
        .get_objects_owned_by_object(field_object.object_id)
        .await
        .unwrap();
    assert_eq!(1, objects.len());
    assert_eq!(child.object_id, objects.first().unwrap().object_id);

    // Query get_objects_owned_by_address should return nothing
    let objects = gateway
        .get_objects_owned_by_address(parent.object_id.into())
        .await
        .unwrap();
    assert!(objects.is_empty())
}

#[tokio::test]
async fn test_multiple_gateways() {
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();
    let (addr2, _key2): (_, AccountKeyPair) = get_key_pair();

    let coin_object1 = Object::with_owner_for_testing(addr1);
    let coin_object2 = Object::with_owner_for_testing(addr1);
    let coin_object3 = Object::with_owner_for_testing(addr1);
    let gas_object = Object::with_owner_for_testing(addr1);

    let genesis_objects = vec![
        coin_object1.clone(),
        coin_object2.clone(),
        coin_object3.clone(),
        gas_object.clone(),
    ];
    let gateway1 = create_gateway_state(genesis_objects).await;
    let path = tempfile::tempdir().unwrap().into_path();
    // gateway2 shares the same set of authorities as gateway1.
    let gateway2 = GatewayState::new_with_authorities(
        Arc::new(GatewayStore::open(&path, None).unwrap()),
        gateway1.authorities.clone(),
        GatewayMetrics::new_for_tests(),
    )
    .unwrap();
    let response = public_transfer_object(
        &gateway1,
        addr1,
        &key1,
        coin_object1.id(),
        gas_object.id(),
        addr2,
    )
    .await
    .unwrap();
    assert!(response.effects.status.is_ok());

    // gas_object on gateway2 should be out-of-dated.
    // Show that we can still handle the transaction successfully if we use it on gateway2.
    let response = public_transfer_object(
        &gateway2,
        addr1,
        &key1,
        coin_object2.id(),
        gas_object.id(),
        addr2,
    )
    .await
    .unwrap();
    assert!(response.effects.status.is_ok());

    // Now we try to use the same gas object on gateway1, and it will still work.
    let response = public_transfer_object(
        &gateway1,
        addr1,
        &key1,
        coin_object3.id(),
        gas_object.id(),
        addr2,
    )
    .await
    .unwrap();
    assert!(response.effects.status.is_ok());
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_batch_transaction() {
    let (addr1, key1): (_, AccountKeyPair) = get_key_pair();
    let (addr2, _key2): (_, AccountKeyPair) = get_key_pair();

    let coin_object1 = Object::with_owner_for_testing(addr1);
    let coin_object2 = Object::with_owner_for_testing(addr1);
    let gas_object = Object::with_owner_for_testing(addr1);

    let genesis_objects = vec![
        coin_object1.clone(),
        coin_object2.clone(),
        gas_object.clone(),
    ];
    let gateway = create_gateway_state(genesis_objects).await;
    let params = vec![
        RPCTransactionRequestParams::TransferObjectRequestParams(TransferObjectParams {
            object_id: coin_object1.id(),
            recipient: addr2,
        }),
        RPCTransactionRequestParams::TransferObjectRequestParams(TransferObjectParams {
            object_id: coin_object2.id(),
            recipient: addr2,
        }),
    ];
    // Gateway should be able to figure out the only usable gas object.
    let data = gateway
        .batch_transaction(addr1, params, None, 5000)
        .await
        .unwrap();
    let signature = key1.sign(&data.to_bytes());
    let effects = gateway
        .execute_transaction(Transaction::from_data(data, signature))
        .await
        .unwrap()
        .effects;
    assert!(effects.created.is_empty());
    assert_eq!(effects.mutated.len(), 3);
}
