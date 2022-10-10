[![codecov](https://codecov.io/gh/gdexorg/gdex-core/branch/main/graph/badge.svg?token=1LV5N5F8Q1)](https://codecov.io/gh/gdexorg/gdex-core)
[![Tests](https://github.com/gdexorg/gdex-core/actions/workflows/test.yml/badge.svg)](https://github.com/gdexorg/gdex-core/actions/workflows/test.yml)
[![Coverage](https://github.com/gdexorg/gdex-core/actions/workflows/coverage.yml/badge.svg)](https://github.com/gdexorg/gdex-core/actions/workflows/coverage.yml)
![](https://tokei.rs/b1/github/gdexorg/gdex-core)
# Introducing GDEX-CORE


## Overview 


### How is the repo organized?

    gdex-rs 
    ├── benchmark                  # Tools for running performance benchmarks
    ├── cli                        # Command line interface implementation
    ├── core                       # Core Blockchain logic, like validation
    ├── controller                 # App-specific logic
    ├── node                       # Scripts for running a local node instance
    ├── suite                      # Unit tests & benches
    ├── type                       # Internal type definitions
    └── workspace-hack
