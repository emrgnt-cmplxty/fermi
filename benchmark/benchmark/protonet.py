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
from benchmark.gdex_logs import LogParser, ParseError
from benchmark.instance import InstanceManager
from benchmark.remote import Bench

class FabricError(Exception):
    ''' Wrapper for Fabric exception with a meaningfull error message. '''

    def __init__(self, error):
        assert isinstance(error, GroupException)
        message = list(error.result.values())[-1]
        super().__init__(message)


class ExecutionError(Exception):
    pass


class Protonet(Bench):
    def _run_single(self, rate, committee):

        # Kill any potentially unfinished run and delete logs.
        hosts = committee.ips()
        self.kill(hosts=hosts, delete_logs=True)

        # Run the clients (they will wait for the nodes to be ready).
        # Filter all faulty nodes from the client addresses (or they will wait
        # for the faulty nodes to be online).
        Print.info('Booting nodes...')
        rate_share = ceil(rate / len(committee.json['authorities']))
        for i, name in enumerate(committee.json['authorities'].keys()):
            validator_dict = committee.json['authorities'][name]
            validator_address = validator_dict['network_address'].split('/')
            validator_address[2] = '0.0.0.0'
            validator_address = '/'.join(validator_address)
            relayer_address = validator_dict['relayer_address'].split('/')
            relayer_address[2] = '0.0.0.0'
            relayer_address = '/'.join(relayer_address)

            metrics_address = validator_dict['metrics_address'].split('/')
            metrics_address[2] = '0.0.0.0'
            metrics_address = '/'.join(metrics_address)

            host = Committee.ip_from_multi_address(validator_dict['network_address'])
            cmd = CommandMaker.run_gdex_node(
                self.remote_proto_dir,
                self.remote_proto_dir,
                self.remote_proto_dir + PathMaker.key_file(i),
                name,
                validator_address,
                relayer_address,
                metrics_address
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
            if self.bench_parameters.order_bench:
                cmd = CommandMaker.run_gdex_orderbook_client(
                    multiaddr_to_url_data(validator_address),
                    multiaddr_to_url_data(relayer_address),
                    self.remote_proto_dir + PathMaker.key_file(0),
                    rate_share,
                    [multiaddr_to_url_data(node['network_address']) for node in committee.json['authorities'].values() if node['network_address'] != validator_address]
                )
            else:
                cmd = CommandMaker.run_gdex_client(
                    multiaddr_to_url_data(validator_address),
                    multiaddr_to_url_data(relayer_address),
                    self.remote_proto_dir + PathMaker.key_file(i),
                    rate_share,
                    [multiaddr_to_url_data(node['network_address']) for node in committee.json['authorities'].values() if node['network_address'] != validator_address]
                )
            log_file = PathMaker.client_log_file(i, 0)
            self._background_run(host, cmd, log_file)
        print('Running the protonet')

    def run(self, bench_parameters_dict, node_parameters_dict, debug):
        Print.heading('Starting protonet')

        self.bench_parameters = GDEXBenchParameters(bench_parameters_dict)
        self.node_parameters = NodeParameters(node_parameters_dict)
        self.debug = debug
        self.local_proto_dir = os.getcwd() + self.bench_parameters.key_dir
        self.remote_proto_dir = self.settings.repo_name + self.bench_parameters.key_dir

        # Select which hosts to use.
        selected_hosts = self._select_hosts(self.bench_parameters)
        if not selected_hosts:
            Print.warn('There are not enough instances available')
            return

        # Update nodes.
        try:
            self._update(selected_hosts, self.bench_parameters.collocate)
        except (GroupException, ExecutionError) as e:
            e = FabricError(e) if isinstance(e, GroupException) else e
            raise BenchError('Failed to update nodes', e)

        # Upload all configuration files.
        try:
            committee = self._config(
                selected_hosts, self.node_parameters, self.bench_parameters
            )
        except (subprocess.SubprocessError, GroupException) as e:
            e = FabricError(e) if isinstance(e, GroupException) else e
            raise BenchError('Failed to configure nodes', e)

        # Run protonet
        rate = self.bench_parameters.rate[0]

        try:
            self._run_single(rate, committee)
        except (subprocess.SubprocessError, GroupException, ParseError) as e:
            self.kill(hosts=selected_hosts)
            if isinstance(e, GroupException):
                e = FabricError(e)
            Print.error(BenchError('Protonet crashed', e))
