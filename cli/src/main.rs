//! Copyright (c) 2022, Mysten Labs, Inc.
//! Copyright (c) 2022, BTI
//! SPDX-License-Identifier: Apache-2.0
use clap::*;
use colored::Colorize;
use gdex_cli::command::GDEXCommand;
use gdex_types::exit_main;
#[cfg(test)]
#[path = "unit_tests/cli_tests.rs"]
mod cli_tests;

#[tokio::main]
async fn main() {
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let bin_name = env!("CARGO_BIN_NAME");
    let cmd: GDEXCommand = GDEXCommand::parse();
    let _guard = telemetry_subscribers::TelemetryConfig::new(bin_name).with_env().init();

    exit_main!(cmd.execute().await);
}
