# Copyright(C) Facebook, Inc. and its affiliates.
import subprocess
from math import ceil
from os.path import basename, splitext
from time import sleep

from benchmark.commands import CommandMaker
from benchmark.config import Key, LocalCommittee, NodeParameters, BenchParameters, GDEXBenchParameters, ConfigError
from benchmark.gdex_logs import LogParser, ParseError
from benchmark.utils import Print, BenchError, PathMaker

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
            print('self.nodes=', self.nodes)
            nodes, rate = self.nodes, self.rate

            # Cleanup all files.
            cmd = f'{CommandMaker.clean_logs()} ; {CommandMaker.cleanup()}'
            subprocess.run([cmd], shell=True, stderr=subprocess.DEVNULL)
            sleep(0.5)  # Removing the store may take time.

            # Recompile the latest code.
            cmd = CommandMaker.compile(mem_profiling=self.mem_profile)
            Print.info(f"About to run {cmd}...")
            subprocess.run(cmd, check=True, cwd=PathMaker.narwhal_node_crate_path())
            
            # Recompile the latest code.
            cmd = CommandMaker.compile(mem_profiling=self.mem_profile, benchmark=False)
            Print.info(f"About to run {cmd}...")
            subprocess.run(cmd, check=True, cwd=PathMaker.narwhal_node_crate_path())

            # Create alias for the client and nodes binary.
            cmd = CommandMaker.alias_binaries(PathMaker.binary_path())
            print(cmd)
            subprocess.run([cmd], shell=True)

            # Run the clients (they will wait for the nodes to be ready).
            
            # currently hard-coded for a single worker, for n-workers denom = n*len(nodes.keys())
            rate_share = ceil(rate / len(nodes.keys()))
            for id, address in enumerate(nodes.values()):
                    cmd = CommandMaker.run_gdex_client(
                        url_to_multiaddr(address),
                        512,
                        rate_share,
                        "advanced",
                        [x for x in nodes.values() if x != address]
                    )
                    print(cmd)
                    # currently hard-coded for a single worker, for n-workers 0 -> i
                    log_file = PathMaker.client_log_file(id, 0)
                    self._background_run(cmd, log_file)

            # Run the primaries
            for id, node_name in enumerate(nodes.keys()):
                cmd = CommandMaker.run_gdex_primary(
                    self.db_dir,
                    self.genesis_dir,
                    self.key_dir,
                    node_name,
                    url_to_multiaddr(nodes[node_name]),
                    debug
                )
                print(cmd)
                log_file = PathMaker.primary_log_file(id)
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
