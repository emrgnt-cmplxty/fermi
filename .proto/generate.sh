../target/release/gdex init-genesis --path $PWD
../target/release/gdex generate-keystore $PWD validator-0.key
../target/release/gdex generate-keystore $PWD validator-1.key

#../target/release/gdex add-validator-genesis --path $PWD --name validator-0 --balance 2000000000000 --stake 1000000 --key-file validator-0.key --network-address /dns/localhost/tcp/62228/http --narwhal-primary-to-primary /dns/localhost/tcp/62230/http --narwhal-worker-to-primary /dns/localhost/tcp/62232/http --narwhal-primary-to-worker /dns/localhost/tcp/62234/http --narwhal-worker-to-worker /dns/localhost/tcp/62236/http --narwhal-consensus-address /dns/localhost/tcp/62238/http
#../target/release/gdex add-validator-genesis --path $PWD --name validator-1 --balance 3000000000000 --stake 2000000 --key-file validator-1.key --network-address /dns/localhost/tcp/62340/http --narwhal-primary-to-primary /dns/localhost/tcp/62342/http --narwhal-worker-to-primary /dns/localhost/tcp/62344/http --narwhal-primary-to-worker /dns/localhost/tcp/62346/http --narwhal-worker-to-worker /dns/localhost/tcp/62348/http --narwhal-consensus-address /dns/localhost/tcp/62350/http
#../target/release/gdex add-validator-genesis --path $PWD --name validator-2 --balance 4000000000000 --stake 3000000 --key-file validator-2.key --network-address /dns/localhost/tcp/62452/http --narwhal-primary-to-primary /dns/localhost/tcp/62454/http --narwhal-worker-to-primary /dns/localhost/tcp/62456/http --narwhal-primary-to-worker /dns/localhost/tcp/62458/http --narwhal-worker-to-worker /dns/localhost/tcp/62460/http --narwhal-consensus-address /dns/localhost/tcp/62462/http
#../target/release/gdex add-validator-genesis --path $PWD --name validator-3 --balance 5000000000000 --stake 4000000 --key-file validator-3.key --network-address /dns/localhost/tcp/62564/http --narwhal-primary-to-primary /dns/localhost/tcp/62566/http --narwhal-worker-to-primary /dns/localhost/tcp/62568/http --narwhal-primary-to-worker /dns/localhost/tcp/62570/http --narwhal-worker-to-worker /dns/localhost/tcp/62572/http --narwhal-consensus-address /dns/localhost/tcp/62574/http
#../target/release/gdex add-controllers-genesis --path $PWD
#../target/release/gdex build-genesis --path $PWD
#../target/release/gdex verify-and-sign-genesis --path $PWD --key-file validator-0.key
#../target/release/gdex verify-and-sign-genesis --path $PWD --key-file validator-1.key
#../target/release/gdex verify-and-sign-genesis --path $PWD --key-file validator-2.key
#../target/release/gdex verify-and-sign-genesis --path $PWD --key-file validator-3.key
#../target/release/gdex finalize-genesis --path $PWD