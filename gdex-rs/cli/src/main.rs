// gdex
use gdex_cli::command::GDEXCommand;
use gdex_types::exit_main;
// external
use clap::*;
use colored::Colorize;

#[cfg(test)]
#[path = "cli_tests.rs"]
mod cli_tests;

#[cfg(not(tarpaulin))]
#[tokio::main]
async fn main() {
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let bin_name = env!("CARGO_BIN_NAME");
    let cmd: GDEXCommand = GDEXCommand::parse();
    let _guard = telemetry_subscribers::TelemetryConfig::new(bin_name).with_env().init();

    exit_main!(cmd.execute().await);
}
