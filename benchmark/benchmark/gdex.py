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
from benchmark.utils import multiaddr_to_url_data


def url_to_multiaddr(url):
    assert isinstance(url, str)
    return '/dns/localhost/tcp/%s/http' % (url.split(':')[-1])

class GDEXBench:
    BASE_PORT = 3000

    def __init__(self, bench_parameters_dict):
        try:
            self.bench_parameters = GDEXBenchParameters(bench_parameters_dict)
        except ConfigError as e:
            raise BenchError('Invalid nodes or bench parameters', e)

    def __getattr__(self, attr):
        return getattr(self.bench_parameters, attr)

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

    def run(self, debug=False):
        assert isinstance(debug, bool)
        Print.heading('Starting local benchmark')

        # Kill any previous testbed.
        self._kill_nodes()

        try:
            Print.info('Setting up testbed...')
            nodes, rate = self.nodes, self.rate

            # Cleanup all files.
            cmd = f'{CommandMaker.clean_logs()} ;'
            subprocess.run([cmd], shell=True)
            cmd = CommandMaker.cleanup()
            subprocess.run([cmd], shell=True, cwd=PathMaker.gdex_build_path())
            sleep(0.5)  # Removing the store may take time.
            # Recompile the latest code.
            cmd = CommandMaker.compile(mem_profiling=self.mem_profile, flamegraph=self.flamegraph)
            Print.info(f"About to run {cmd}...")
            subprocess.run(cmd, check=True, cwd=PathMaker.gdex_build_path())
            sleep(0.5)  # Removing the store may take time.

            # Recompile the latest code.
            cmd = CommandMaker.compile(mem_profiling=self.mem_profile, flamegraph=self.flamegraph, benchmark=False)
            Print.info(f"About to run {cmd}...")
            subprocess.run(cmd, check=True, cwd=PathMaker.gdex_build_path())
            sleep(5)  # Removing the store may take time.

            # Create alias for the client and nodes binary.
            cmd = CommandMaker.alias_binaries(PathMaker.binary_path())
            print(cmd)
            subprocess.run([cmd], shell=True)

            cmd = CommandMaker.init_gdex_genesis(os.path.abspath(self.bench_parameters.key_dir))
            subprocess.run([cmd], shell=True)
            # Generate configuration files.
            keys = []
            key_files = [PathMaker.key_file(i) for i in range(self.bench_parameters.nodes)]

            for filename in key_files:
                sleep(2)
                cmd = CommandMaker.generate_gdex_key(filename, os.path.abspath(self.bench_parameters.key_dir)).split()
                subprocess.run(cmd, check=True)
                keys += [Key.from_file(os.path.abspath(self.bench_parameters.key_dir + filename))]

            names = [x.name for x in keys]

            workers = self.bench_parameters.workers
            committee = LocalCommittee(names, 3000, workers)
            committee.print(PathMaker.committee_file())
            for i, name in enumerate(names):
                validator_dict = committee.json["authorities"][name]
                balance = 5000000000000
                stake = validator_dict["stake"]
                key_file = os.path.abspath(self.bench_parameters.key_dir + key_files[i])

                primary_to_primary = validator_dict["primary"]["primary_to_primary"]
                worker_to_primary = validator_dict["primary"]["worker_to_primary"]
                primary_to_worker = []
                worker_to_worker = []
                consensus_address = []
                for i in range(self.bench_parameters.workers):
                    primary_to_worker.append(validator_dict["workers"][i]["primary_to_worker"])
                    worker_to_worker.append(validator_dict["workers"][i]["worker_to_worker"])
                    consensus_address.append(validator_dict["workers"][i]["transactions"])

                cmd = CommandMaker.add_gdex_validator_genesis(
                    self.bench_parameters.key_dir,
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
                print(cmd)
                subprocess.run([cmd], shell=True)

            cmd = CommandMaker.add_controllers_gdex_genesis(os.path.abspath(self.bench_parameters.key_dir))
            subprocess.run([cmd], shell=True)
            cmd = CommandMaker.build_gdex_genesis(os.path.abspath(self.bench_parameters.key_dir))
            subprocess.run([cmd], shell=True)

            for i, name in enumerate(committee.json['authorities'].keys())  :
                cmd = CommandMaker.verify_and_sign_gdex_genesis(os.path.abspath(self.bench_parameters.key_dir), os.path.abspath(self.bench_parameters.key_dir + key_files[i]))
                subprocess.run([cmd], shell=True)
            cmd = CommandMaker.finalize_genesis(os.path.abspath(self.bench_parameters.key_dir))
            subprocess.run([cmd], shell=True)

            # Run the primaries
            # currently hard-coded for a single worker, for n-workers denom = n*len(nodes.keys())
            Print.info('Booting nodes...')
            rate_share = ceil(rate / committee.workers())
            for i, name in enumerate(committee.json['authorities'].keys()):
                validator_dict = committee.json['authorities'][name]
                validator_address = validator_dict['network_address']
                relayer_address = validator_dict['relayer_address']
                cmd = CommandMaker.run_gdex_node(
                    os.path.abspath(self.bench_parameters.key_dir),
                    os.path.abspath(self.bench_parameters.key_dir),
                    os.path.abspath(self.bench_parameters.key_dir + PathMaker.key_file(i)),
                    name,
                    validator_address,
                    relayer_address,
                    debug,
                    self.flamegraph
                )
                log_file = PathMaker.primary_log_file(i)
                print(cmd, ">>", log_file)
                self._background_run(cmd, log_file)

            Print.info('Booting clients...')
            for i, name in enumerate(committee.json['authorities'].keys()):
                validator_dict = committee.json['authorities'][name]
                validator_address = validator_dict['network_address']
                relayer_address = validator_dict['relayer_address']
                if self.order_bench:
                    cmd = CommandMaker.run_gdex_orderbook_client(
                        i,
                        multiaddr_to_url_data(validator_address),
                        multiaddr_to_url_data(relayer_address),
                        rate_share,
                        [multiaddr_to_url_data(node['network_address']) for node in committee.json['authorities'].values() if node['network_address'] != validator_address]
                    )
                else:
                    cmd = CommandMaker.run_gdex_client(
                        multiaddr_to_url_data(validator_address),
                        multiaddr_to_url_data(relayer_address),
                        os.path.abspath(self.bench_parameters.key_dir + PathMaker.key_file(i)),
                        rate_share,
                        [multiaddr_to_url_data(node['network_address']) for node in committee.json['authorities'].values() if node['network_address'] != validator_address]
                    )
                log_file = PathMaker.client_log_file(i, 0)
                print(cmd, ">>", log_file)
                self._background_run(cmd, log_file)

            # Wait for all transactions to be processed.
            Print.info(f'Running benchmark ({self.duration} sec)...')
            sleep(self.duration)
            self._kill_nodes()

            # Parse logs and return the parser.
            Print.info('Parsing logs...')
            return LogParser.process(PathMaker.logs_path(), faults=self.faults)

        except (subprocess.SubprocessError, ParseError) as e:
            self._kill_nodes()
            raise BenchError('Failed to run benchmark', e)
