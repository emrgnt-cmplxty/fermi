yarn install

cd sdk/js/fermi
yarn install
yarn build

cd ../tenex
yarn install
yarn build

cd ../../../

cd scripts/futures
yarn install

# deploy markets and execute a trade
yarn deploy
nohup yarn push-prices &
nohup yarn make-markets &

### ### setup the Fermi sandbox
### # cd benchmark
### # pip3 install -r requirements.txt
### ### for instructions on how to install Fabric
### ### https://www.fabfile.org/installing.html
### # nohup fab sandbox
