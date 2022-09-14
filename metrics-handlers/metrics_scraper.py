import argparse
from flask import Flask
import json
import os
from utils import *

clear = lambda: os.system('clear')

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Process command line input.')
    parser.add_argument('monitor_type', type=str, default='command_line', nargs='?', help='Specify a monitoring type: command line or email')
    parser.add_argument('live_chunk_size', type=int, default=5, nargs='?', help='Specify a the chunk size for live printout')
    args, unknown = parser.parse_known_args()
    assert(args.monitor_type in ['command_line', 'email'])

    metrics_thresholds_file = open("metrics_thresholds.json")
    metrics_thresholds = json.load(metrics_thresholds_file)
    gauge_metrics_to_scrape = list(metrics_thresholds["gauges"].keys())
    histogram_metrics_to_scrape = list(metrics_thresholds["histograms"].keys())

    # begin logic for "command_line" mode
    if args.monitor_type == "command_line":
        display_chunk_size = args.live_chunk_size

        while True:
            df = get_metrics(gauge_metrics_to_scrape + histogram_metrics_to_scrape)
            clear()

            print("Global Consensus Metrics")
            global_metrics = [col for col in df.columns if 'tx_' not in col]
            for chunk in [global_metrics[i:i+display_chunk_size] for i in range(0, len(global_metrics), display_chunk_size)]:
                print(df[chunk])

            print("\nInternal Channel Metrics")
            channel_metrics = [col for col in df.columns if 'tx_' in col]
            for chunk in [channel_metrics[i:i+display_chunk_size] for i in range(0, len(channel_metrics), display_chunk_size)]:
                print(df[chunk])

    # TODO - implement
    elif args.monitor_type == "email":
        pass
        # # check if any of the metrics are above the threshold
        # diff_df = df - df.mean()
        # diff_bools = diff_df.abs() > metrics_thresholds["gauges"]
        # check_result = diff_bools.any(axis=1).any(axis=0)
