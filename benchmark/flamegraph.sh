
# ---- Narwhal Flamegraph ----
# Launch clients
./benchmark_narwhal_client http://127.0.0.1:3003/ --size 512 --rate 6250 --execution advanced --nodes http://127.0.0.1:3003/ http://127.0.0.1:3008/ http://127.0.0.1:3013/ http://127.0.0.1:3018/
./benchmark_narwhal_client http://127.0.0.1:3008/ --size 512 --rate 6250 --execution advanced --nodes http://127.0.0.1:3003/ http://127.0.0.1:3008/ http://127.0.0.1:3013/ http://127.0.0.1:3018/
./benchmark_narwhal_client http://127.0.0.1:3013/ --size 512 --rate 6250 --execution advanced --nodes http://127.0.0.1:3003/ http://127.0.0.1:3008/ http://127.0.0.1:3013/ http://127.0.0.1:3018/
./benchmark_narwhal_client http://127.0.0.1:3018/ --size 512 --rate 6250 --execution advanced --nodes http://127.0.0.1:3003/ http://127.0.0.1:3008/ http://127.0.0.1:3013/ http://127.0.0.1:3018/

# Launch primaries (select 1 primary for flamegraph)
./benchmark-narwhal -vvv run --keys .node-0.json --committee .committee.json --store .db-0 --parameters .parameters.json --execution advanced primary
./benchmark-narwhal -vvv run --keys .node-1.json --committee .committee.json --store .db-1 --parameters .parameters.json --execution advanced primary
./benchmark-narwhal -vvv run --keys .node-2.json --committee .committee.json --store .db-2 --parameters .parameters.json --execution advanced primary
./benchmark-narwhal -vvv run --keys .node-3.json --committee .committee.json --store .db-3 --parameters .parameters.json --execution advanced primary

# Launch workers (or select 1 worker for flamegraph) 
flamegraph -o primary-flamegraph-0.svg -- ./benchmark-narwhal -vvv run --keys .node-0.json --committee .committee.json --store .db-0-0 --parameters .parameters.json --execution advanced worker --id 0
./benchmark-narwhal -vvv run --keys .node-1.json --committee .committee.json --store .db-1-0 --parameters .parameters.json --execution advanced worker --id 0
./benchmark-narwhal -vvv run --keys .node-2.json --committee .committee.json --store .db-2-0 --parameters .parameters.json --execution advanced worker --id 0
./benchmark-narwhal -vvv run --keys .node-3.json --committee .committee.json --store .db-3-0 --parameters .parameters.json --execution advanced worker --id 0