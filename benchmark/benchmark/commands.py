# Copyright(C) Facebook, Inc. and its affiliates.
from os.path import join

from benchmark.utils import PathMaker


class CommandMaker:

    @staticmethod
    def cleanup():
        return (
            f'rm -r .db-* ;  rm -r *_db ; rm .*.json ; mkdir -p {PathMaker.results_path()}'
        )

    @staticmethod
    def clean_logs():
        return f'rm -r {PathMaker.logs_path()} ; mkdir -p {PathMaker.logs_path()}'

    @staticmethod
    def compile(mem_profiling, benchmark=True):
        if mem_profiling:
            params = ["--profile", "bench-profiling", "--features", "benchmark dhat-heap"]
        elif benchmark:
            params = ["--release", "--features", "benchmark"]
        else:
            params = ["--release"]
        return ["cargo", "build"] + params

    @staticmethod
    def generate_key(filename):
        assert isinstance(filename, str)
        return f'./benchmark-narwhal generate_keys --filename {filename}'

    @staticmethod
    def run_narwhal_primary(keys, committee, store, parameters, execution, debug=False):
        assert isinstance(keys, str)
        assert isinstance(committee, str)
        assert isinstance(parameters, str)
        assert isinstance(execution, str)
        assert isinstance(debug, bool)
        v = '-vvv' if debug else '-vv'
        command = (f'./benchmark-narwhal {v} run --keys {keys} --committee {committee} '
                f'--store {store} --parameters {parameters} --execution {execution} primary')
        print("Returning execution command = ", command)
        return command

    @staticmethod
    def run_gdex_node(db_dir, genesis_dir, key_dir, validator_name, validator_address, relayer_address, debug=False):
        assert isinstance(db_dir, str)
        assert isinstance(genesis_dir, str)
        assert isinstance(key_dir, str)
        assert isinstance(validator_name, str)
        assert isinstance(validator_address, str)
        assert isinstance(relayer_address, str)
        assert isinstance(debug, bool)
        v = '-vvv' if debug else '-vv'
        command = (f'./gdex-node {v} run --db-dir {db_dir} --genesis-dir  {genesis_dir} '
                f'--key-dir {key_dir} --validator-name {validator_name} --validator-address {validator_address} --relayer-address {relayer_address}')
        print("Returning execution command = ", command)
        return command

    @staticmethod
    def run_no_consensus_primary(keys, committee, store, parameters, debug=False):
        assert isinstance(keys, str)
        assert isinstance(committee, str)
        assert isinstance(parameters, str)
        assert isinstance(debug, bool)
        v = '-vvv' if debug else '-vv'
        return (f'./benchmark-narwhal {v} run --keys {keys} --committee {committee} '
                f'--store {store} --parameters {parameters} primary --consensus-disabled')

    @staticmethod
    def run_narwhal_worker(keys, committee, store, parameters, execution, id, debug=False):
        assert isinstance(keys, str)
        assert isinstance(committee, str)
        assert isinstance(parameters, str)
        assert isinstance(execution, str)
        assert isinstance(debug, bool)
        v = '-vvv' if debug else '-vv'
        command = (f'./benchmark-narwhal {v} run --keys {keys} --committee {committee} '
                f'--store {store} --parameters {parameters} --execution {execution} worker --id {id}')

        print("Returning execution command = ", command)
        return command

    @staticmethod
    def run_narwhal_client(address, size, rate, execution, nodes):
        assert isinstance(address, str)
        assert isinstance(size, int) and size > 0
        assert isinstance(rate, int) and rate >= 0
        assert isinstance(execution, str)
        assert isinstance(nodes, list)
        assert all(isinstance(x, str) for x in nodes)
        nodes = f'--nodes {" ".join(nodes)}' if nodes else ''
        command = f'./benchmark_narwhal_client {address} --size {size} --rate {rate} --execution {execution} {nodes}'
        print("Returning execution command = ", command)
        return command

    @staticmethod
    def run_gdex_client(address, relayer_address, rate, nodes):
        assert isinstance(address, str)
        assert isinstance(rate, int) and rate >= 0
        assert isinstance(nodes, list)
        assert all(isinstance(x, str) for x in nodes)
        nodes = f'--nodes {" ".join(nodes)}' if nodes else ''
        command = f'./benchmark_gdex_client {address} --relayer {relayer_address} --validator_key_fpath ../.proto/validator-0.key --rate {rate}  --nodes {nodes}'
        print("Returning execution command = ", command)
        return command

    @staticmethod
    def alias_demo_binaries(origin):
        assert isinstance(origin, str)
        client = join(origin, 'demo_client')
        return f'rm demo_client ; ln -s {client} .'

    @staticmethod
    def run_demo_client(keys, ports):
        assert all(isinstance(x, str) for x in keys)
        assert all(isinstance(x, int) and x > 1024 for x in ports)
        keys_string = ",".join(keys)
        ports_string = ",".join([str(x) for x in ports])
        return f'./demo_client run --keys "{keys_string}" --ports "{ports_string}"'

    @staticmethod
    def kill():
        return 'tmux kill-server'

    @staticmethod
    def alias_binaries(origin):
        print("origin=", origin)
        assert isinstance(origin, str)
        gdex_node, narwhal_node, narwhhal_client, gdex_client = join(origin, 'gdex-node'), join(origin, 'benchmark-narwhal'), join(origin, 'benchmark_narwhal_client'), join(origin, 'benchmark_gdex_client')
        return f'rm gdex-node ; rm benchmark-narwhal ; rm benchmark_narwhal_client ; rm benchmark_gdex_client ; ln -s {gdex_node} . ; ln -s {narwhal_node} . ; ln -s {narwhhal_client}; ln -s {gdex_client} .'
