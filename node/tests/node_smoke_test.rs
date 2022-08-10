// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use narwhal_config::Export;
use std::time::{Duration, Instant};
use test_utils::{committee, keys, temp_dir};

const TEST_DURATION: Duration = Duration::from_secs(3);

#[test]
fn test_primary_no_consensus() {
    let db_path = temp_dir().into_os_string().into_string().unwrap();
    let config_path = temp_dir().into_os_string().into_string().unwrap();
    let now = Instant::now();
    let duration = TEST_DURATION;

    let keys = keys(None);
    let keys_file_path = format!("{config_path}/smoke_test_keys.json");
    keys[0].export(&keys_file_path).unwrap();

    let committee = committee(None);
    let committee_file_path = format!("{config_path}/smoke_test_committee.json");
    committee.export(&committee_file_path).unwrap();

    let mut child = std::process::Command::new("cargo")
        .current_dir("..")
        .args(&["run", "--bin", "node", "--"])
        .args(&[
            "run",
            "--committee",
            &committee_file_path,
            "--keys",
            &keys_file_path,
            "--store",
            &db_path,
            "--execution",
            "no-advanced", 
            "primary",
            "--consensus-disabled",
        ])
        .spawn()
        .expect("failed to launch primary process w/o consensus");

    while now.elapsed() < duration {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    panic!("node panicked with: {:?}", child.stderr.take().unwrap());
                }
                assert!(status.success());
                break;
            }
            Ok(None) => continue,
            Err(e) => {
                panic!("error waiting for child process: {}", e);
            }
        }
    }
    let _ = child.kill();
}

#[test]
fn test_primary_no_consensus_advanced_execution() {
    let db_path = temp_dir().into_os_string().into_string().unwrap();
    let config_path = temp_dir().into_os_string().into_string().unwrap();
    let now = Instant::now();
    let duration = TEST_DURATION;

    let keys = keys(None);
    let keys_file_path = format!("{config_path}/smoke_test_keys.json");
    keys[0].export(&keys_file_path).unwrap();

    let committee = committee(None);
    let committee_file_path = format!("{config_path}/smoke_test_committee.json");
    committee.export(&committee_file_path).unwrap();

    let mut child = std::process::Command::new("cargo")
        .current_dir("..")
        .args(&["run", "--bin", "node", "--"])
        .args(&[
            "run",
            "--committee",
            &committee_file_path,
            "--keys",
            &keys_file_path,
            "--store",
            &db_path,
            "--execution",
            "advanced", 
            "primary",
            "--consensus-disabled",
        ])
        .spawn()
        .expect("failed to launch primary process w/o consensus");

    while now.elapsed() < duration {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    panic!("node panicked with: {:?}", child.stderr.take().unwrap());
                }
                assert!(status.success());
                break;
            }
            Ok(None) => continue,
            Err(e) => {
                panic!("error waiting for child process: {}", e);
            }
        }
    }
    let _ = child.kill();
}

#[test]
fn test_primary_with_consensus() {
    let db_path = temp_dir().into_os_string().into_string().unwrap();
    let config_path = temp_dir().into_os_string().into_string().unwrap();
    let now = Instant::now();
    let duration = TEST_DURATION;

    let keys = keys(None);
    let keys_file_path = format!("{config_path}/smoke_test_keys.json");
    keys[0].export(&keys_file_path).unwrap();

    let committee = committee(None);
    let committee_file_path = format!("{config_path}/smoke_test_committee.json");
    committee.export(&committee_file_path).unwrap();

    let mut child = std::process::Command::new("cargo")
        .current_dir("..")
        .args(&["run", "--bin", "node", "--"])
        .args(&[
            "run",
            "--committee",
            &committee_file_path,
            "--keys",
            &keys_file_path,
            "--store",
            &db_path,
            "--execution",
            "no-advanced",
            "primary",
            //no arg : default of with_consensus
        ])
        .spawn()
        .expect("failed to launch primary process w/o consensus");

    while now.elapsed() < duration {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    panic!("node panicked with: {:?}", child.stderr.take().unwrap());
                }
                assert!(status.success());
                break;
            }
            // This is expected to run indefinitely => will hit the timeout
            Ok(None) => continue,
            Err(e) => {
                panic!("error waiting for child process: {}", e);
            }
        }
    }
    let _ = child.kill();
}

#[test]
fn test_primary_with_consensus_advanced_execution() {
    let db_path = temp_dir().into_os_string().into_string().unwrap();
    let config_path = temp_dir().into_os_string().into_string().unwrap();
    let now = Instant::now();
    let duration = TEST_DURATION;

    let keys = keys(None);
    let keys_file_path = format!("{config_path}/smoke_test_keys.json");
    keys[0].export(&keys_file_path).unwrap();

    let committee = committee(None);
    let committee_file_path = format!("{config_path}/smoke_test_committee.json");
    committee.export(&committee_file_path).unwrap();

    let mut child = std::process::Command::new("cargo")
        .current_dir("..")
        .args(&["run", "--bin", "node", "--"])
        .args(&[
            "run",
            "--committee",
            &committee_file_path,
            "--keys",
            &keys_file_path,
            "--store",
            &db_path,
            "--execution",
            "advanced",
            "primary",
            //no arg : default of with_consensus
        ])
        .spawn()
        .expect("failed to launch primary process w/o consensus");

    while now.elapsed() < duration {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    panic!("node panicked with: {:?}", child.stderr.take().unwrap());
                }
                assert!(status.success());
                break;
            }
            // This is expected to run indefinitely => will hit the timeout
            Ok(None) => continue,
            Err(e) => {
                panic!("error waiting for child process: {}", e);
            }
        }
    }
    let _ = child.kill();
}

#[test]
fn test_benchmark_client() {

    let now = Instant::now();
    let duration = TEST_DURATION;

    const TXN_SIZE: u32 = 512;
    const TXN_RATE: u32 = 12500;

    let mut child = std::process::Command::new("cargo")
        .current_dir("..")
        .args(&["run", "--bin", "benchmark_client", "--features", "benchmark", "--"])
        .args(&[
            "http://127.0.0.1:3003/",
            "--size",
            &TXN_SIZE.to_string(),
            "--rate",
            &TXN_RATE.to_string(),
            "--execution",
            "no-advanced", 
            "--nodes",
            "http://127.0.0.1:3003/ http://127.0.0.1:3008/ http://127.0.0.1:3013/ http://127.0.0.1:3018/",
        ])
        .spawn()
        .expect("failed to launch benchmark client");

    while now.elapsed() < duration {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    panic!("benchmark client panicked with: {:?}", child.stderr.take().unwrap());
                }
                assert!(status.success());
                break;
            }
            Ok(None) => continue,
            Err(e) => {
                panic!("error waiting for child process: {}", e);
            }
        }
    }
    let _ = child.kill();
}