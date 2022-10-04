# Copyright(C) Facebook, Inc. and its affiliates.
import os
import subprocess
from collections import OrderedDict
from math import ceil
from os.path import basename, splitext
from time import sleep

from benchmark.commands import CommandMaker
from benchmark.config import Key, LocalCommittee, NodeParameters, BenchParameters, GDEXBenchParameters, ConfigError
from benchmark.gdex_logs import LogParser, ParseError
from benchmark.utils import Print, BenchError, PathMaker

from benchmark.config import Committee
from benchmark.utils import multiaddr_to_url_data, url_to_multiaddr


class GDEXBench:
    BASE_PORT = 3000

    def __init__(self):
        pass

    def _background_run(self, command, log_file):
        name = splitext(basename(log_file))[0]
        cmd = f'{command} 2> {log_file}'
        subprocess.run(['tmux', 'new', '-d', '-s', name, cmd], check=True)

    def _kill_nodes(self):
        try:
            cmd = CommandMaker.kill().split()
            subprocess.run(cmd, stderr=subprocess.DEVNULL)
        except subprocess.SubprocessError as e:
            raise BenchError('Failed to kill testbed', e)

    def setup_genesis(self, bench_parameters_dict, benchmark=True, release=True):
        Print.info('Setting up testbed...')
        try:
            bench_parameters = GDEXBenchParameters(bench_parameters_dict)
        except ConfigError as e:
            raise BenchError('Invalid nodes or bench parameters', e)

        # Cleanup all files.
        cmd = f'{CommandMaker.clean_logs()} ;'
        Print.info(f"Running {cmd}")
        subprocess.run([cmd], shell=True)
        cmd = CommandMaker.cleanup()
        Print.info(f"Running {cmd}")
        subprocess.run([cmd], shell=True)
        sleep(0.5)  # Removing the store may take time.
        # Recompile the latest code.
        cmd = CommandMaker.compile(mem_profiling=bench_parameters.mem_profile, flamegraph=bench_parameters.flamegraph, benchmark=benchmark, release=release)
        Print.info(f"Running {cmd}")
        subprocess.run(cmd, check=True, cwd=PathMaker.gdex_build_path())
        sleep(0.5)  # Removing the store may take time.

        # Create alias for the client and nodes binary.
        cmd = CommandMaker.alias_binaries(PathMaker.binary_path(release))
        Print.info(f"Running {cmd}")
        subprocess.run([cmd], shell=True)

        cmd = CommandMaker.init_gdex_genesis(os.path.abspath(bench_parameters.key_dir))
        Print.info(f"Running {cmd}...")
        subprocess.run([cmd], shell=True)
        # Generate configuration files.
        keys = []
        key_files = [PathMaker.key_file(i) for i in range(bench_parameters.nodes[0])]

        for filename in key_files:
            cmd = CommandMaker.generate_gdex_key(filename, os.path.abspath(bench_parameters.key_dir)).split()
            Print.info(f"Running {cmd}...")
            subprocess.run(cmd, check=True)
            keys += [Key.from_file(os.path.abspath(bench_parameters.key_dir + filename))]

        sleep(5)
        names = [x.name for x in keys]

        workers = bench_parameters.workers
        committee = LocalCommittee(names, 3000, workers)
        committee.print(PathMaker.committee_file())
        for i, name in enumerate(names):
            validator_dict = committee.json["authorities"][name]
            balance = bench_parameters.starting_balance
            stake = validator_dict["stake"]
            key_file = os.path.abspath(bench_parameters.key_dir + key_files[i])

            primary_to_primary = validator_dict["primary"]["primary_to_primary"]
            worker_to_primary = validator_dict["primary"]["worker_to_primary"]
            primary_to_worker = []
            worker_to_worker = []
            consensus_address = []
            for i in range(bench_parameters.workers):
                primary_to_worker.append(validator_dict["workers"][i]["primary_to_worker"])
                worker_to_worker.append(validator_dict["workers"][i]["worker_to_worker"])
                consensus_address.append(validator_dict["workers"][i]["transactions"])

            cmd = CommandMaker.add_gdex_validator_genesis(
                bench_parameters.key_dir,
                name,
                balance,
                stake,
                key_file,
                primary_to_primary,
                worker_to_primary,
                ','.join(primary_to_worker),
                ','.join(worker_to_worker),
                ','.join(consensus_address)
            )
            Print.info(f"Running {cmd}")
            subprocess.run([cmd], shell=True)

        cmd = CommandMaker.add_controllers_gdex_genesis(os.path.abspath(bench_parameters.key_dir))
        Print.info(f"Running {cmd}")
        subprocess.run([cmd], shell=True)
        cmd = CommandMaker.build_gdex_genesis(os.path.abspath(bench_parameters.key_dir))
        Print.info(f"Running {cmd}")
        subprocess.run([cmd], shell=True)

        for i, name in enumerate(committee.json['authorities'].keys()):
            cmd = CommandMaker.verify_and_sign_gdex_genesis(os.path.abspath(bench_parameters.key_dir), os.path.abspath(bench_parameters.key_dir + key_files[i]))
            Print.info(f"Running {cmd}")
            subprocess.run([cmd], shell=True)
        Print.info(f"Running {cmd}")
        cmd = CommandMaker.finalize_genesis(os.path.abspath(bench_parameters.key_dir))
        Print.info(f"Running {cmd}")
        subprocess.run([cmd], shell=True)
        return committee, bench_parameters

    def run(self, bench_parameters_dict, debug=False):
        assert isinstance(debug, bool)
        Print.heading('Starting local benchmark')
        # Kill any previous testbed.
        self._kill_nodes()

        try:
            committee, bench_parameters = self.setup_genesis(bench_parameters_dict)
            # Run the primaries
            Print.info('Booting nodes...')
            rate_share = ceil(bench_parameters.rate[0] / (bench_parameters.workers * bench_parameters.nodes[0]))
            for i, name in enumerate(committee.json['authorities'].keys()):
                validator_dict = committee.json['authorities'][name]
                validator_grpc_address = validator_dict['grpc_address']
                validator_jsonrpc_address = validator_dict['jsonrpc_address']
                metrics_address = validator_dict['metrics_address']
                cmd = CommandMaker.run_gdex_node(
                    os.path.abspath(bench_parameters.db_dir),
                    os.path.abspath(bench_parameters.key_dir),
                    os.path.abspath(bench_parameters.key_dir + PathMaker.key_file(i)),
                    name,
                    validator_grpc_address,
                    validator_jsonrpc_address,
                    metrics_address,
                    debug,
                    bench_parameters.flamegraph
                )
                log_file = PathMaker.primary_log_file(i)
                print(cmd, ">>", log_file)
                self._background_run(cmd, log_file)
                # sleep to avoid weird port collisions in local bench
                sleep(0.5)

            Print.info('Booting clients...')
            # spawn one client per worker
            for i, name in enumerate(committee.json['authorities'].keys()):
              for worker_idx in range(bench_parameters.workers):
                validator_dict = committee.json['authorities'][name]
                validator_grpc_address = validator_dict['grpc_address']
                validator_jsonrpc_address = validator_dict['jsonrpc_address']
                if bench_parameters.order_bench:
                    cmd = CommandMaker.run_gdex_orderbook_client(
                        multiaddr_to_url_data(validator_grpc_address),
                        os.path.abspath(bench_parameters.key_dir + PathMaker.key_file(0)),
                        rate_share,
                        [multiaddr_to_url_data(node['grpc_address']) for node in committee.json['authorities'].values() if node['grpc_address'] != validator_grpc_address]
                    )
                else:
                    cmd = CommandMaker.run_gdex_client(
                        multiaddr_to_url_data(validator_grpc_address),
                        os.path.abspath(bench_parameters.key_dir + PathMaker.key_file(i)),
                        rate_share,
                        [multiaddr_to_url_data(node['grpc_address']) for node in committee.json['authorities'].values() if node['grpc_address'] != validator_grpc_address]
                    )
                log_file = PathMaker.client_log_file(i, worker_idx)
                print(cmd, ">>", log_file)
                self._background_run(cmd, log_file)

            # Wait for all transactions to be processed.
            Print.info(f'Running benchmark ({bench_parameters.duration} sec)...')
            sleep(bench_parameters.duration)
            self._kill_nodes()

            # Parse logs and return the parser.
            Print.info('Parsing logs...')
            return LogParser.process(PathMaker.logs_path(), faults=bench_parameters.faults)

        except (subprocess.SubprocessError, ParseError) as e:
            self._kill_nodes()
            raise BenchError('Failed to run benchmark', e)