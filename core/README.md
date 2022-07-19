## Overview

This directory contains the simplest implementations the elmentary pieces that make up a blockchain.

### How is the module organized?

    core/src
    ├── block                    # Blocks hold transactions and associated meta data
    ├── hash_clock               # The hash clock iteratively hashes an initial input
    ├── transaction              # Transactions specifiy user interactions with the blockchain
    └── vote_cert                # Vote certificates certify a proposed block