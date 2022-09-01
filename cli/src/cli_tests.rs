//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
//! Note, the code in this file is inspired by https://github.com/MystenLabs/sui/blob/main/crates/sui/src/unit_tests/cli_tests.rs, commit #e91604e0863c86c77ea1def8d9bd116127bee0bc
#[cfg(test)]
mod cli_test_suite {
    use gdex_cli::command::GDEXCommand;
    use gdex_core::config::{
        network::NetworkConfig, PersistedConfig, GDEX_FULLNODE_CONFIG, GDEX_GATEWAY_CONFIG, GDEX_GENESIS_FILENAME,
        GDEX_NETWORK_CONFIG,
    };
    use std::{fs::read_dir, path::PathBuf};

    #[tokio::test]
    async fn genesis() -> Result<(), anyhow::Error> {
        let temp_dir = tempfile::tempdir()?;
        let working_dir = temp_dir.path();
        let config = working_dir.join(GDEX_NETWORK_CONFIG);

        // Start network without authorities
        let start = GDEXCommand::Start {
            config: Some(config.clone()),
            debug_max_ticks: None,
        }
        .execute()
        .await;
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

        // Start the network again, this time with authorities
        let start = GDEXCommand::Start {
            config: Some(config),
            debug_max_ticks: Some(5),
        }
        .execute()
        .await;
        start.unwrap();

        temp_dir.close()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_build_keystore() -> Result<(), anyhow::Error> {
        let temp_dir = tempfile::tempdir()?;
        let working_dir = temp_dir.path();

        // Genesis
        GDEXCommand::GenerateKeystore {
            keystore_path: Some(working_dir.to_path_buf()),
            keystore_name: Some(String::from("test.conf")),
        }
        .execute()
        .await?;

        Ok(())
    }

    #[should_panic]
    #[tokio::test]
    async fn start_bad_config() {
        let config = PathBuf::from("a_bad_config/test");

        // Start network without authorities
        GDEXCommand::Start {
            config: Some(config),
            debug_max_ticks: None,
        }
        .execute()
        .await
        .unwrap();
    }

    #[should_panic]
    #[tokio::test]
    async fn genesis_bad_config() {
        let working_dir = PathBuf::from("a_bad_config/test");

        GDEXCommand::Genesis {
            working_dir: Some(working_dir.to_path_buf()),
            write_config: None,
            force: false,
            from_config: None,
        }
        .execute()
        .await
        .unwrap();
    }

    #[should_panic]
    #[tokio::test]
    async fn repeat_genesis_no_force() {
        GDEXCommand::Genesis {
            working_dir: None,
            write_config: None,
            force: false,
            from_config: None,
        }
        .execute()
        .await
        .unwrap();
        GDEXCommand::Genesis {
            working_dir: None,
            write_config: None,
            force: false,
            from_config: None,
        }
        .execute()
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn genesis_with_specified_config() -> Result<(), anyhow::Error> {
        let temp_dir = tempfile::tempdir()?;
        let working_dir = temp_dir.path();

        // Genesis
        GDEXCommand::Genesis {
            working_dir: Some(working_dir.to_path_buf()),
            write_config: Some(working_dir.join(GDEX_GENESIS_FILENAME).to_path_buf()),
            force: true,
            from_config: None,
        }
        .execute()
        .await?;
        Ok(())
    }
}
