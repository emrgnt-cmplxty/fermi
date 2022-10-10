// gdex
use gdex_controller::bank::rpc_server;
use gdex_core::json_rpc;
// external
use std::fs::File;
use std::io::Write;
use sui_json_rpc::sui_rpc_doc;
use sui_json_rpc::SuiRpcModule;

const FILE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/json_rpc/spec/openrpc.json",);

// This simple script generates an OpenRPC document for the RPC server.
#[tokio::main]
async fn main() {
    let mut open_rpc = sui_rpc_doc();
    open_rpc.add_module(<json_rpc::server::JSONRPCService as SuiRpcModule>::rpc_doc_module());
    open_rpc.add_module(<rpc_server::JSONRPCService as SuiRpcModule>::rpc_doc_module());

    let content = serde_json::to_string_pretty(&open_rpc).unwrap();
    let mut f = File::create(FILE_PATH).unwrap();
    writeln!(f, "{content}").unwrap();
}
