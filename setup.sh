### setup the Fermi sandbox
cd benchmark
pip3 install -r requirements.txt

### for instructions on how to install Fabric
### https://www.fabfile.org/installing.html

fab sandbox
cd ..

### setup SDKs
cd sdk/js/fermi
yarn install
yarn build

cd ../tenex
yarn install
yarn build

cd ../../../

### setup exchange UI
cd exchange
yarn install
# yarn run start

### deploy DEX
cp benchmark/.committee.json configs/protonet.json
cd scripts/deploy_futures
yarn tsc deployFuturesMarket.ts --esModuleInterop --resolveJsonModule && npx ts-node deployFuturesMarket.ts
