//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use gdex_cli::command::GDEXCommand;
use gdex_core::config::{
    network::NetworkConfig, PersistedConfig, GDEX_FULLNODE_CONFIG, GDEX_GATEWAY_CONFIG, GDEX_GENESIS_FILENAME,
    GDEX_NETWORK_CONFIG,
};
use std::fs::read_dir;

#[tokio::test]
async fn test_genesis() -> Result<(), anyhow::Error> {
    let temp_dir = tempfile::tempdir()?;
    let working_dir = temp_dir.path();
    let config = working_dir.join(GDEX_NETWORK_CONFIG);

    // Start network without authorities
    let start = GDEXCommand::Start { config: Some(config) }.execute().await;
    assert!(matches!(start, Err(..)));
    // Genesis
    GDEXCommand::Genesis {
        working_dir: Some(working_dir.to_path_buf()),
        write_config: None,
        force: false,
        from_config: None,
    }
    .execute()
    .await?;

    // Get all the new file names
    let files = read_dir(working_dir)?
        .flat_map(|r| r.map(|file| file.file_name().to_str().unwrap().to_owned()))
        .collect::<Vec<_>>();

    assert_eq!(8, files.len());
    assert!(files.contains(&GDEX_GATEWAY_CONFIG.to_string()));
    assert!(files.contains(&GDEX_NETWORK_CONFIG.to_string()));
    assert!(files.contains(&GDEX_FULLNODE_CONFIG.to_string()));
    assert!(files.contains(&GDEX_GENESIS_FILENAME.to_string()));
    // Commented components in GDEXCommand::Genesis
    // assert!(files.contains(&GDEX_KEYSTORE_FILENAME.to_string()));
    // assert!(files.contains(&GDEX_CLIENT_CONFIG.to_string()));

    // Check network config
    let network_conf = PersistedConfig::<NetworkConfig>::read(&working_dir.join(GDEX_NETWORK_CONFIG))?;
    assert_eq!(4, network_conf.validator_configs().len());

    // Commented components in GDEXCommand::Genesis
    // Check wallet config
    // let wallet_conf = PersistedConfig::<SuiClientConfig>::read(&working_dir.join(GDEX_CLIENT_CONFIG))?;

    // if let ClientType::Embedded(config) = &wallet_conf.gateway {
    //     assert_eq!(4, config.validator_set.len());
    //     assert_eq!(working_dir.join("client_db"), config.db_folder_path);
    // } else {
    //     panic!()
    // }

    // assert_eq!(5, wallet_conf.accounts.len());

    // Genesis 2nd time should fail
    let result = GDEXCommand::Genesis {
        working_dir: Some(working_dir.to_path_buf()),
        write_config: None,
        force: false,
        from_config: None,
    }
    .execute()
    .await;
    assert!(matches!(result, Err(..)));

    temp_dir.close()?;
    Ok(())
}
