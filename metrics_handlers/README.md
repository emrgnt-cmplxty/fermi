## Overview

This directory contains a monitoring impelmentation which consumes the .committee.json file generated during the benchmark process.



```
    metrics_handlers/
    ├── metrics_scraper.py
    ├── metrics_server.py
    # specifies the gauges to scrape and thresholds at which alerts are triggered   
    ├── metrics_thresholds.json
```


### Use

    # continuously print out metrics
    python3 metrics_scraper.py

    # run serve which returns json of metrics
    python3 metrics_server.py
