# Copyright(C) Facebook, Inc. and its affiliates.
import os
from collections import OrderedDict
from fabric import Connection, ThreadingGroup as Group
from fabric.exceptions import GroupException
from paramiko import RSAKey
from paramiko.ssh_exception import PasswordRequiredException, SSHException
from os.path import basename, splitext
from time import sleep
from math import ceil
from copy import deepcopy
import subprocess

from benchmark.config import Committee, Key, NodeParameters, BenchParameters, ConfigError, GDEXBenchParameters
from benchmark.utils import BenchError, Print, PathMaker, progress_bar, multiaddr_to_url_data
from benchmark.commands import CommandMaker
from benchmark.logs import LogParser, ParseError
from benchmark.instance import InstanceManager

class FabricError(Exception):
    ''' Wrapper for Fabric exception with a meaningfull error message. '''

    def __init__(self, error):
        assert isinstance(error, GroupException)
        message = list(error.result.values())[-1]
        super().__init__(message)


class ExecutionError(Exception):
    pass


class Bench:
    def __init__(self, ctx):
        self.manager = InstanceManager.make()
        self.settings = self.manager.settings
        self.local_proto_dir = os.getcwd() + '/.proto/'
        self.remote_proto_dir = self.settings.repo_name + "/.proto/"
        try:
            ctx.connect_kwargs.pkey = RSAKey.from_private_key_file(
                self.manager.settings.key_path
            )
            ctx.forward_agent = True
            self.connect = ctx.connect_kwargs
        except (IOError, PasswordRequiredException, SSHException) as e:
            raise BenchError('Failed to load SSH key', e)

    def _check_stderr(self, output):
        if isinstance(output, dict):
            for x in output.values():
                if x.stderr:
                    raise ExecutionError(x.stderr)
        else:
            if output.stderr:
                raise ExecutionError(output.stderr)

    def install(self):
        Print.info('Installing rust and cloning the repo...')
        cmd = [
            'sudo apt-get update',
            'sudo apt-get -y upgrade',
            'sudo apt-get -y autoremove',

            # The following dependencies prevent the error: [error: linker `cc` not found].
            'sudo apt-get -y install build-essential',
            'sudo apt-get -y install cmake',
            'sudo apt-get install libssl-dev',

            # Install rust (non-interactive).
            'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y',
            'source $HOME/.cargo/env',
            'rustup default stable',

            # This is missing from the Rocksdb installer (needed for Rocksdb).
            'sudo apt-get install -y clang',
            'sudo apt-get install openssl',
            'sudo apt-get install pkg-config',

            # Clone the repo.
            'ssh-keyscan -H github.com >> ~/.ssh/known_hosts',
            f'(git clone {self.settings.repo_url} || (cd {self.settings.repo_name} ; git pull))'
        ]
        hosts = self.manager.hosts(flat=True)
        try:
            g = Group(*hosts, user='ubuntu', connect_kwargs=self.connect, forward_agent=True)
            # TODO fix this hack
            for c in g:
                c._config.forward_agent = True
            g.run(' && '.join(cmd), hide=False)
            Print.heading(f'Initialized testbed of {len(hosts)} nodes')
        except (GroupException, ExecutionError) as e:
            e = FabricError(e) if isinstance(e, GroupException) else e
            raise BenchError('Failed to install repo on testbed', e)

    def kill(self, hosts=[], delete_logs=False):
        assert isinstance(hosts, list)
        assert isinstance(delete_logs, bool)
        hosts = hosts if hosts else self.manager.hosts(flat=True)
        delete_logs = CommandMaker.clean_logs() if delete_logs else 'true'
        cmd = [delete_logs, f'({CommandMaker.kill()} || true)']
        try:
            g = Group(*hosts, user='ubuntu', connect_kwargs=self.connect)
            g.run(' && '.join(cmd), hide=True)
        except GroupException as e:
            raise BenchError('Failed to kill nodes', FabricError(e))

    def _select_hosts(self, bench_parameters):
        # Collocate the primary and its workers on the same machine.
        if bench_parameters.collocate:
            nodes = max(bench_parameters.nodes)

            # Ensure there are enough hosts.
            hosts = self.manager.hosts()
            if sum(len(x) for x in hosts.values()) < nodes:
                return []

            # Select the hosts in different data centers.
            ordered = zip(*hosts.values())
            ordered = [x for y in ordered for x in y]
            return ordered[:nodes]

        # Spawn the primary and each worker on a different machine. Each
        # authority runs in a single data center.
        else:
            primaries = max(bench_parameters.nodes)

            # Ensure there are enough hosts.
            hosts = self.manager.hosts()
            if len(hosts.keys()) < primaries:
                return []
            for ips in hosts.values():
                if len(ips) < bench_parameters.workers + 1:
                    return []

            # Ensure the primary and its workers are in the same region.
            selected = []
            for region in list(hosts.keys())[:primaries]:
                ips = list(hosts[region])[:bench_parameters.workers + 1]
                selected.append(ips)
            return selected

    def _background_run(self, host, command, log_file, cwd='~/gdex-core/benchmark'):
        name = splitext(basename(log_file))[0]
        cmd = f'(cd {cwd}) && tmux new -d -s "{name}" "{command} |& tee {log_file}"'
        c = Connection(host, user='ubuntu', connect_kwargs=self.connect)
        output = c.run(cmd, hide=True)
        self._check_stderr(output)

    def _update(self, hosts, collocate):
        if collocate:
            ips = list(set(hosts))
        else:
            ips = list(set([x for y in hosts for x in y]))

        Print.info(
            f'Updating {len(ips)} machines (branch "{self.settings.branch}")...'
        )
        compile_cmd = ' '.join(CommandMaker.compile(False))
        cmd = [
            f'(cd {self.settings.repo_name} && git fetch -f)',
            f'(cd {self.settings.repo_name} && git checkout -f {self.settings.branch})',
            f'(cd {self.settings.repo_name} && git pull -f)',
            'source $HOME/.cargo/env',
            f'(cd {self.settings.repo_name} && {compile_cmd})',
            CommandMaker.alias_binaries(
                f'./{self.settings.repo_name}/target/release/'
            )
        ]
        g = Group(*ips, user='ubuntu', connect_kwargs=self.connect, forward_agent=True)
        g.run(' && '.join(cmd), hide=True)

    def _config(self, hosts, node_parameters, bench_parameters):
        Print.info('Generating configuration files...')
        # Cleanup all local configuration files.
        cmd = CommandMaker.cleanup()
        subprocess.run([cmd], shell=True, stderr=subprocess.DEVNULL, cwd='.')

        # Recompile the latest code.
        cmd = CommandMaker.compile(mem_profiling=False)
        Print.info(f"About to run {cmd}...")
        subprocess.run(cmd, check=True, cwd='../')

        # Create alias for the client and nodes binary.
        cmd = CommandMaker.alias_binaries(PathMaker.binary_path())
        subprocess.run([cmd], shell=True)

        cmd = CommandMaker.init_gdex_genesis(self.local_proto_dir)
        subprocess.run([cmd], shell=True)

        # Generate configuration files.
        keys = []
        key_files = [PathMaker.key_file(i) for i in range(len(hosts))]

        for filename in key_files:
            cmd = CommandMaker.generate_gdex_key(filename).split()
            subprocess.run(cmd, check=True)
            keys += [Key.from_file(self.local_proto_dir + filename)]
        sleep(2)
        names = [x.name for x in keys]

        if bench_parameters.collocate:
            workers = bench_parameters.workers
            addresses = OrderedDict(
                (x, [y] * (workers + 1)) for x, y in zip(names, hosts)
            )
        else:
            addresses = OrderedDict(
                (x, y) for x, y in zip(names, hosts)
            )
        committee = Committee(addresses, self.settings.base_port)
        committee.print(PathMaker.committee_file())
        for i, name in enumerate(names):
            sleep(5)
            validator_dict = committee.json["authorities"][name]
            balance = 5000000000000
            stake = validator_dict["stake"]
            key_file = self.local_proto_dir + key_files[i]

            primary_to_primary = validator_dict["primary"]["primary_to_primary"]
            worker_to_primary = validator_dict["primary"]["worker_to_primary"]

            primary_to_worker = validator_dict["workers"][0]["primary_to_worker"]
            worker_to_worker = validator_dict["workers"][0]["worker_to_worker"]
            consensus_address = validator_dict["workers"][0]["transactions"]
            cmd = CommandMaker.add_gdex_validator_genesis(
                self.local_proto_dir,
                name,
                balance,
                stake,
                key_file,
                primary_to_primary,
                worker_to_primary,
                primary_to_worker,
                worker_to_worker,
                consensus_address
            )
            subprocess.run([cmd], shell=True)
        sleep(2)
        cmd = CommandMaker.add_controllers_gdex_genesis(self.local_proto_dir)
        subprocess.run([cmd], shell=True)
        sleep(2)
        cmd = CommandMaker.build_gdex_genesis(self.local_proto_dir)
        subprocess.run([cmd], shell=True)
        sleep(5)
        for i, name in enumerate(names):
            cmd = CommandMaker.verify_and_sign_gdex_genesis(self.local_proto_dir, self.local_proto_dir + key_files[i])
            subprocess.run([cmd], shell=True)

        cmd = CommandMaker.finalize_genesis(self.local_proto_dir)
        subprocess.run([cmd], shell=True)

        node_parameters.print(PathMaker.parameters_file())

        # Cleanup all nodes and upload configuration files.
        local_committee_dir = self.local_proto_dir + 'committee/'
        remote_committee_dir = self.remote_proto_dir + 'committee/'
        local_signatures_dir = self.local_proto_dir + 'signatures/'
        remote_signatures_dir = self.remote_proto_dir + 'signatures/'

        names = names[:len(names)-bench_parameters.faults]
        progress = progress_bar(names, prefix='Uploading config files:')
        for i, name in enumerate(progress):
            for ip in committee.ips(name):
                print(i, name, ip)
                c = Connection(ip, user='ubuntu', connect_kwargs=self.connect)
                c.run(f'(cd gdex-core && {CommandMaker.cleanup()}) || true', hide=False)
                c.put(self.local_proto_dir + "genesis.blob", self.remote_proto_dir)
                c.put(self.local_proto_dir + PathMaker.key_file(i), self.remote_proto_dir)
                c.put(self.local_proto_dir + "master_controller", self.remote_proto_dir)

                for fname in os.listdir(local_committee_dir):
                    c.put(local_committee_dir + fname, remote_committee_dir)

                for fname in os.listdir(local_signatures_dir):
                    c.put(local_signatures_dir + fname, remote_signatures_dir)



        return committee

    def _run_single(self, rate, committee, bench_parameters, node_parameters, debug=False):
        faults = bench_parameters.faults

        # Kill any potentially unfinished run and delete logs.
        hosts = committee.ips()
        self.kill(hosts=hosts, delete_logs=True)

        # Run the clients (they will wait for the nodes to be ready).
        # Filter all faulty nodes from the client addresses (or they will wait
        # for the faulty nodes to be online).
        Print.info('Booting nodes...')
        rate_share = ceil(rate / committee.workers())
        for i, name in enumerate(committee.json['authorities'].keys()):
            validator_dict = committee.json['authorities'][name]
            validator_address = validator_dict['network_address'].split('/')
            validator_address[2] = '0.0.0.0'
            validator_address = '/'.join(validator_address)
            relayer_address = validator_dict['relayer_address'].split('/')
            relayer_address[2] = '0.0.0.0'
            relayer_address = '/'.join(relayer_address)
            host = Committee.ip_from_multi_address(validator_dict['network_address'])
            cmd = CommandMaker.run_gdex_node(
                self.remote_proto_dir,
                self.remote_proto_dir,
                self.remote_proto_dir + PathMaker.key_file(i),
                name,
                validator_address,
                relayer_address
            )

            log_file = PathMaker.primary_log_file(i)
            self._background_run(host, cmd, log_file)

        # Run the primaries (except the faulty ones).
        Print.info('Booting clients...')
        for i, name in enumerate(committee.json['authorities'].keys()):
            validator_dict = committee.json['authorities'][name]
            validator_address = validator_dict['network_address']
            relayer_address = validator_dict['relayer_address']
            host = Committee.ip_from_multi_address(validator_address)
            cmd = CommandMaker.run_gdex_client(
                multiaddr_to_url_data(validator_address),
                multiaddr_to_url_data(relayer_address),
                self.remote_proto_dir + PathMaker.key_file(i),
                rate_share,
                [multiaddr_to_url_data(node['network_address']) for node in committee.json['authorities'].values() if node['network_address'] != validator_address]
            )
            log_file = PathMaker.client_log_file(i, 0)
            self._background_run(host, cmd, log_file)

        # Wait for all transactions to be processed.
        duration = bench_parameters.duration
        for _ in progress_bar(range(20), prefix=f'Running benchmark ({duration} sec):'):
            sleep(ceil(duration / 20))
        self.kill(hosts=hosts, delete_logs=False)

    def _logs(self, committee, faults):
        # Delete local logs (if any).
        cmd = CommandMaker.clean_logs()
        subprocess.run([cmd], shell=True, stderr=subprocess.DEVNULL)

        # Download log files.
        workers_addresses = committee.workers_addresses(faults)
        progress = progress_bar(workers_addresses, prefix='Downloading workers logs:')
        for i, addresses in enumerate(progress):
            for id, address in addresses:
                host = Committee.ip(address)
                c = Connection(host, user='ubuntu', connect_kwargs=self.connect)
                c.get(
                    PathMaker.client_log_file(i, 0),
                    local=PathMaker.client_log_file(i, 0)
                )
                c.get(
                    PathMaker.primary_log_file(i),
                    local=PathMaker.primary_log_file(i)
                )
                c.get(
                    PathMaker.primary_log_file(i),
                    local=PathMaker.worker_log_file(i, 0)
                )
        # Parse logs and return the parser.
        Print.info('Parsing logs and computing performance...')
        return LogParser.process(PathMaker.logs_path(), faults=faults)

    def run(self, bench_parameters_dict, node_parameters_dict, debug=False):
        assert isinstance(debug, bool)
        Print.heading('Starting remote benchmark')
        try:
            bench_parameters = BenchParameters(bench_parameters_dict)
            node_parameters = NodeParameters(node_parameters_dict)
        except ConfigError as e:
            raise BenchError('Invalid nodes or bench parameters', e)

        # Select which hosts to use.
        selected_hosts = self._select_hosts(bench_parameters)
        if not selected_hosts:
            Print.warn('There are not enough instances available')
            return

        # Update nodes.
        try:
            self._update(selected_hosts, bench_parameters.collocate)
        except (GroupException, ExecutionError) as e:
            e = FabricError(e) if isinstance(e, GroupException) else e
            raise BenchError('Failed to update nodes', e)

        # Upload all configuration files.
        try:
            committee = self._config(
                selected_hosts, node_parameters, bench_parameters
            )
        except (subprocess.SubprocessError, GroupException) as e:
            e = FabricError(e) if isinstance(e, GroupException) else e
            raise BenchError('Failed to configure nodes', e)

        # Run benchmarks.
        for n in bench_parameters.nodes:
            committee_copy = deepcopy(committee)
            committee_copy.remove_nodes(committee.size() - n)

            for r in bench_parameters.rate:
                Print.heading(f'\nRunning {n} nodes (input rate: {r:,} tx/s)')

                # Run the benchmark.
                for i in range(bench_parameters.runs):
                    Print.heading(f'Run {i+1}/{bench_parameters.runs}')
                    try:
                        self._run_single(
                            r, committee_copy, bench_parameters, node_parameters, debug
                        )

                        faults = bench_parameters.faults
                        logger = self._logs(committee_copy, faults)
                        logger.print(PathMaker.result_file(
                            faults,
                            n,
                            bench_parameters.workers,
                            bench_parameters.collocate,
                            r,
                            bench_parameters.tx_size,
                        ))
                    except (subprocess.SubprocessError, GroupException, ParseError) as e:
                        self.kill(hosts=selected_hosts)
                        if isinstance(e, GroupException):
                            e = FabricError(e)
                        Print.error(BenchError('Benchmark failed', e))
                        continue
