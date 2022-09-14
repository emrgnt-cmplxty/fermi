# Copyright(C) Facebook, Inc. and its affiliates.
from os.path import join

from benchmark.utils import PathMaker


class CommandMaker:
    @staticmethod
    def cleanup():
        return (
            f"rm -r .db-* ; "
            f"rm -r *_db ; "
            f"rm .proto/committee/* ; "
            f"rm .proto/signatures/* ; "
            f"rm -r .proto/db/* ; "
            f"rm .proto/*.key ; "
            f"rm .proto/*.blob ; "
            f"rm .proto/*controller ; "
            f"rm .proto/.*.json ; "
            f"mkdir -p {PathMaker.results_path()} ; "
        )

    @staticmethod
    def clean_logs():
        return f"rm -r {PathMaker.logs_path()} ; mkdir -p {PathMaker.logs_path()}"

    @staticmethod
    def compile(mem_profiling, flamegraph, benchmark=True, release=True):
        if mem_profiling:
            params = [
                "--profile",
                "bench-profiling",
                "--features",
                "benchmark dhat-heap",
            ]
        elif flamegraph:
            params = ["--profile", "flamegraph-profiling", "--features", "benchmark"]
        elif benchmark:
            params = ["--release", "--features", "benchmark"]
        elif release:
            params = ["--release"]
        else:
            params = []
        return ["cargo", "build"] + params

    @staticmethod
    def generate_key(filename):
        assert isinstance(filename, str)
        return f"./benchmark-narwhal generate_keys --filename {filename}"

    @staticmethod
    def init_gdex_genesis(path):
        assert isinstance(path, str)
        return f"./gdex init-genesis --path {path}"

    @staticmethod
    def add_controllers_gdex_genesis(path):
        return f"./gdex add-controllers-genesis --path {path}"

    @staticmethod
    def build_gdex_genesis(path):
        return f"./gdex build-genesis --path {path}"

    @staticmethod
    def add_gdex_validator_genesis(
        path,
        name,
        balance,
        stake,
        key_file,
        primary_to_primary_address,
        worker_to_primary_address,
        primary_to_worker_address,
        worker_to_worker_address,
        consensus_address,
    ):
        assert isinstance(path, str)
        return (
            f"./gdex add-validator-genesis --path {path} --name {name} --balance {balance} --stake {stake}"
            f" --key-file {key_file} --narwhal-primary-to-primary {primary_to_primary_address}"
            f" --narwhal-worker-to-primary {worker_to_primary_address} --narwhal-primary-to-worker {primary_to_worker_address}"
            f" --narwhal-worker-to-worker {worker_to_worker_address} --narwhal-consensus-addresses {consensus_address}"
        )

    @staticmethod
    def verify_and_sign_gdex_genesis(path, filename):
        assert isinstance(path, str)
        assert isinstance(filename, str)
        return f"./gdex verify-and-sign-genesis --path {path} --key-file {filename}"

    @staticmethod
    def finalize_genesis(path):
        return f"./gdex finalize-genesis --path {path}"

    @staticmethod
    def generate_gdex_key(filename, path=".proto"):
        assert isinstance(path, str)
        assert isinstance(filename, str)
        return f"./gdex generate-keystore {path} {filename}"

    @staticmethod
    def run_narwhal_primary(
        keys, committee, store, parameters, execution, debug=False, flamegraph=None
    ):
        assert isinstance(keys, str)
        assert isinstance(committee, str)
        assert isinstance(parameters, str)
        assert isinstance(execution, str)
        assert isinstance(debug, bool)
        v = "-vvv" if debug else "-vv"
        flamegraph = "flamegraph -- " if flamegraph else ""
        command = (
            f"{flamegraph}./benchmark-narwhal {v} run --keys {keys} --committee {committee} "
            f"--store {store} --parameters {parameters} --execution {execution} primary"
        )

        return command

    @staticmethod
    def run_gdex_node(
        db_dir,
        genesis_dir,
        key_path,
        validator_name,
        validator_address,
        relayer_address,
        metrics_address,
        debug=False,
        flamegraph=None,
    ):
        assert isinstance(db_dir, str)
        assert isinstance(genesis_dir, str)
        assert isinstance(key_path, str)
        assert isinstance(validator_name, str)
        assert isinstance(validator_address, str)
        assert isinstance(relayer_address, str)
        assert isinstance(debug, bool)
        v = "-vvv" if debug else "-vv"
        flamegraph = "flamegraph -- " if flamegraph else ""

        command = (
            f"{flamegraph}./gdex-node {v} run --db-dir {db_dir} --genesis-dir  {genesis_dir} "
            f"--key-path {key_path} --validator-name {validator_name} --validator-address {validator_address} --relayer-address {relayer_address} --metrics-address {metrics_address}"
        )
        print("Returning execution command = ", command)
        return command

    @staticmethod
    def run_no_consensus_primary(keys, committee, store, parameters, debug=False):
        assert isinstance(keys, str)
        assert isinstance(committee, str)
        assert isinstance(parameters, str)
        assert isinstance(debug, bool)
        v = "-vvv" if debug else "-vv"
        return (
            f"./benchmark-narwhal {v} run --keys {keys} --committee {committee} "
            f"--store {store} --parameters {parameters} primary --consensus-disabled"
        )

    @staticmethod
    def run_narwhal_worker(
        keys, committee, store, parameters, execution, id, debug=False, flamegraph=None
    ):
        assert isinstance(keys, str)
        assert isinstance(committee, str)
        assert isinstance(parameters, str)
        assert isinstance(execution, str)
        assert isinstance(debug, bool)
        v = "-vvv" if debug else "-vv"
        flamegraph = "flamegraph -- " if flamegraph else ""
        command = (
            f"{flamegraph}./benchmark-narwhal {v} run --keys {keys} --committee {committee} "
            f"--store {store} --parameters {parameters} --execution {execution} worker --id {id}"
        )

        return command

    @staticmethod
    def run_narwhal_client(address, size, rate, execution, nodes):
        assert isinstance(address, str)
        assert isinstance(size, int) and size > 0
        assert isinstance(rate, int) and rate >= 0
        assert isinstance(execution, str)
        assert isinstance(nodes, list)
        assert all(isinstance(x, str) for x in nodes)
        nodes = f'--nodes {" ".join(nodes)}' if nodes else ""
        command = f"./benchmark_narwhal_client {address} --size {size} --rate {rate} --execution {execution} {nodes}"

        return command

    @staticmethod
    def run_gdex_client(address, relayer_address, validator_key_path, rate, nodes):
        assert isinstance(address, str)
        assert isinstance(rate, int) and rate >= 0
        assert isinstance(nodes, list)
        assert all(isinstance(x, str) for x in nodes)
        nodes = f'--nodes {" ".join(nodes)}' if nodes else ""
        command = f"./benchmark_gdex_client {address} --relayer {relayer_address} --validator_key_fpath {validator_key_path} --rate {rate} {nodes}"
        print("Returning execution command = ", command)
        return command

    @staticmethod
    def run_gdex_orderbook_client(address, relayer_address, validator_key_path, rate, nodes):
        assert isinstance(address, str)
        assert isinstance(rate, int) and rate >= 0
        assert isinstance(nodes, list)
        assert all(isinstance(x, str) for x in nodes)
        nodes = f'{" ".join(nodes)}' if nodes else ""
        command = f"./benchmark_orderbook_client {address} --relayer {relayer_address} --validator_key_fpath {validator_key_path} --rate {rate}  --nodes {nodes}"
        print("Returning execution command = ", command)
        return command

    @staticmethod
    def alias_demo_binaries(origin):
        assert isinstance(origin, str)
        client = join(origin, "demo_client")
        return f"rm demo_client ; ln -s {client} ."

    @staticmethod
    def run_demo_client(keys, ports):
        assert all(isinstance(x, str) for x in keys)
        assert all(isinstance(x, int) and x > 1024 for x in ports)
        keys_string = ",".join(keys)
        ports_string = ",".join([str(x) for x in ports])
        return f'./demo_client run --keys "{keys_string}" --ports "{ports_string}"'

    @staticmethod
    def kill():
        return "tmux kill-server"

    @staticmethod
    def alias_binaries(origin):
        assert isinstance(origin, str)
        (
            gdex_node,
            narwhal_node,
            narwhhal_client,
            gdex_client,
            gdex,
            orderbook_client,
        ) = (
            join(origin, "gdex-node"),
            join(origin, "benchmark-narwhal"),
            join(origin, "benchmark_narwhal_client"),
            join(origin, "benchmark_gdex_client"),
            join(origin, "gdex"),
            join(origin, "benchmark_orderbook_client"),
        )
        return (
            f"rm gdex-node ; rm benchmark-narwhal ; rm benchmark_narwhal_client ; rm benchmark_gdex_client ; rm gdex ; rm benchmark_orderbook_client ; "
            f"ln -s {gdex_node} . ; ln -s {narwhal_node} . ; ln -s {narwhhal_client} . ; ln -s {gdex_client} . ; ln -s {gdex} . ; ln -s {orderbook_client} ."
        )
