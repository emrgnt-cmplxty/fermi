import aiohttp
import asyncio
import requests
import pandas as pd
from prometheus_client.parser import text_string_to_metric_families
import json

PROTONET_FILE = "../configs/protonet.json"

# fetch metrics
async def fetch_data(authority_info):
    async with aiohttp.ClientSession() as session:
        ip = authority_info["metrics_address"].split("/")[-4]
        port = authority_info["metrics_address"].split("/")[-2]
        url = "http://{}:{}/metrics".format(ip, port)
        async with session.get(url) as resp:
            result = await resp.text()
    return result

async def get_metrics(metrics_to_scrape):
    # fetch committee info from config file
    committee_file = open(PROTONET_FILE)
    committee = json.load(committee_file)
    authorities = committee["authorities"]

    # fetch metrics
    queries = await asyncio.gather(*[fetch_data(authorities[authority]) for authority in authorities])
    fetch_results = dict(zip(authorities, queries))
    
    # process metrics and save to results
    proc_results = {}
    for authority in authorities:
        proc_results[authority] = {}
        fetch_result = fetch_results[authority]
        # parse the metrics
        families = list(text_string_to_metric_families(fetch_result))

        for family in families:
            name, samples = str(family.name), family.samples
            # skip total histogram column or columns that are not in metrics_to_scrape
            if name in metrics_to_scrape:
                # TODO - we may want to maket this more robust
                # histograms tend to have more than 10 samples
                if len(samples) <= 10:
                    proc_results[authority][name] = samples[0].value
                else:
                    total_count = samples[-1].value
                    for sample in samples:
                        # locate the median value, store and break
                        if sample.value >= 0.5 * total_count:
                            proc_results[authority][name] = sample.labels['le']
                            break

    # create dataframe from results
    df = pd.DataFrame(proc_results).T
    return df