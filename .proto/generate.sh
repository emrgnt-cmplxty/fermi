../target/release/gdex init-genesis --path $PWD
../target/release/gdex add-validator-genesis --path $PWD --name validator-0 --key-file validator-0.key --network-address /dns/localhost/tcp/62228/http --narwhal-primary-to-primary /dns/localhost/tcp/62230/http --narwhal-worker-to-primary /dns/localhost/tcp/62232/http --narwhal-primary-to-worker /dns/localhost/tcp/62234/http --narwhal-worker-to-worker /dns/localhost/tcp/62236/http --narwhal-consensus-address /dns/localhost/tcp/62238/http
../target/release/gdex add-validator-genesis --path $PWD --name validator-1 --key-file validator-1.key --network-address /dns/localhost/tcp/62240/http --narwhal-primary-to-primary /dns/localhost/tcp/62242/http --narwhal-worker-to-primary /dns/localhost/tcp/62244/http --narwhal-primary-to-worker /dns/localhost/tcp/62246/http --narwhal-worker-to-worker /dns/localhost/tcp/62248/http --narwhal-consensus-address /dns/localhost/tcp/62250/http
../target/release/gdex add-validator-genesis --path $PWD --name validator-2 --key-file validator-2.key --network-address /dns/localhost/tcp/62252/http --narwhal-primary-to-primary /dns/localhost/tcp/62254/http --narwhal-worker-to-primary /dns/localhost/tcp/62256/http --narwhal-primary-to-worker /dns/localhost/tcp/62258/http --narwhal-worker-to-worker /dns/localhost/tcp/62260/http --narwhal-consensus-address /dns/localhost/tcp/62262/http
../target/release/gdex add-validator-genesis --path $PWD --name validator-3 --key-file validator-3.key --network-address /dns/localhost/tcp/62264/http --narwhal-primary-to-primary /dns/localhost/tcp/62266/http --narwhal-worker-to-primary /dns/localhost/tcp/62268/http --narwhal-primary-to-worker /dns/localhost/tcp/62270/http --narwhal-worker-to-worker /dns/localhost/tcp/62272/http --narwhal-consensus-address /dns/localhost/tcp/62274/http
../target/release/gdex add-controllers-genesis --path $PWD
../target/release/gdex build-genesis --path $PWD
../target/release/gdex verify-and-sign-genesis --path $PWD --key-file validator-0.key
../target/release/gdex verify-and-sign-genesis --path $PWD --key-file validator-1.key
../target/release/gdex verify-and-sign-genesis --path $PWD --key-file validator-2.key
../target/release/gdex verify-and-sign-genesis --path $PWD --key-file validator-3.key
../target/release/gdex finalize-genesis --path $PWD
