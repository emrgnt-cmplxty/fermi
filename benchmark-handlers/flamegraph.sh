
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

# ---- GDEX Flamegraph ----
# Launch nodes
flamegraph -o node-flamegraph-0.svg -- ./gdex-node -vvv run --db-dir . --genesis-dir  ../.proto/ --key-dir ../.proto/ --validator-name validator-0 --validator-address /dns/localhost/tcp/3003/http --relayer-address /dns/localhost/tcp/3004/http >> logs/primary-0.log
./gdex-node -vvv run --db-dir . --genesis-dir  ../.proto/ --key-dir ../.proto/ --validator-name validator-1 --validator-address /dns/localhost/tcp/3013/http --relayer-address /dns/localhost/tcp/3014/http >> logs/primary-1.log
./gdex-node -vvv run --db-dir . --genesis-dir  ../.proto/ --key-dir ../.proto/ --validator-name validator-2 --validator-address /dns/localhost/tcp/3023/http --relayer-address /dns/localhost/tcp/3024/http >> logs/primary-2.log
./gdex-node -vvv run --db-dir . --genesis-dir  ../.proto/ --key-dir ../.proto/ --validator-name validator-3 --validator-address /dns/localhost/tcp/3033/http --relayer-address /dns/localhost/tcp/3034/http >> logs/primary-3.log

# Launch clients
./benchmark_gdex_client http://localhost:3003 --relayer http://localhost:3004 --validator_key_fpath ../.proto/validator-0.key --rate 12500  --nodes --nodes http://localhost:3013 http://localhost:3023 http://localhost:3033 >> logs/client-0-0.log
./benchmark_gdex_client http://localhost:3013 --relayer http://localhost:3014 --validator_key_fpath ../.proto/validator-1.key --rate 12500  --nodes --nodes http://localhost:3003 http://localhost:3023 http://localhost:3033 >> logs/client-1-0.log
./benchmark_gdex_client http://localhost:3023 --relayer http://localhost:3024 --validator_key_fpath ../.proto/validator-2.key --rate 12500  --nodes --nodes http://localhost:3003 http://localhost:3013 http://localhost:3033 >> logs/client-2-0.log
./benchmark_gdex_client http://localhost:3033 --relayer http://localhost:3034 --validator_key_fpath ../.proto/validator-3.key --rate 12500  --nodes --nodes http://localhost:3003 http://localhost:3013 http://localhost:3023 >> logs/client-3-0.log