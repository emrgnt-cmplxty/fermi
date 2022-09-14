import requests
import pandas as pd
from prometheus_client.parser import text_string_to_metric_families
import json

PROTONET_FILE = "../.protonet/.committee.json"

def get_metrics(metrics_to_scrape):
    # fetch committee info from benchmark area
    committee_file = open(PROTONET_FILE)
    committee = json.load(committee_file)

    authorities = committee["authorities"]
    results = {}
    for authority in authorities:
        results[authority] = {}
        authority_info = authorities[authority]
        
        # fetch metrics
        print('fetching data now...')
        ip = authority_info["metrics_address"].split("/")[-4]
        print('ip=', ip)
        port = authority_info["metrics_address"].split("/")[-2]
        print('port=', port)
        result = requests.get("http://%s:%s/metrics" % (ip, port))
        print('result=', result)
        # parse the metrics
        families = list(text_string_to_metric_families(result.text))

        for family in families:
            name, samples = str(family.name), family.samples
            # skip total histogram column or columns that are not in metrics_to_scrape
            if name in metrics_to_scrape:
                # TODO - we may want to maket this more robust
                # histograms tend to have more than 10 samples
                if len(samples) <= 10:
                    results[authority][name] = samples[0].value
                else:
                    total_count = samples[-1].value
                    for sample in samples:
                        # locate the median value, store and break
                        if sample.value >= 0.5 * total_count:
                            results[authority][name] = sample.labels['le']
                            break

    # create dataframe from results
    df = pd.DataFrame(results).T
    return df