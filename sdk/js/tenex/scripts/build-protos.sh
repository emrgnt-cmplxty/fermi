#!/bin/bash

# go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

BASEDIR=$PWD
cd ${BASEDIR}/../
PROTO_DEST=./lib/proto
mkdir -p ${PROTO_DEST}

# Generate proto for transaction

# JavaScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=grpc_js:${PROTO_DEST} \
    --js_out=import_style=commonjs,binary:${PROTO_DEST} \
    --grpc_out=grpc_js:${PROTO_DEST} \
    -I ../../fermi-rs/types/proto \
    ../../fermi-rs/types/proto/transaction.proto

# Typescript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=${PROTO_DEST} \
     -I ../../fermi-rs/types/proto \
    ../../fermi-rs/types/proto/transaction.proto

# Generate proto for bank

# JavaScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=grpc_js:${PROTO_DEST} \
    --js_out=import_style=commonjs,binary:${PROTO_DEST} \
    --grpc_out=grpc_js:${PROTO_DEST} \
    -I ../../fermi-rs/controller/src/bank/proto \
    ../../fermi-rs/controller/src/bank/proto/*.proto

# Typescript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=${PROTO_DEST} \
     -I ../../fermi-rs/controller/src/bank/proto \
    ../../fermi-rs/controller/src/bank/proto/*.proto

# Generate proto for futures

# JavaScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=grpc_js:${PROTO_DEST} \
    --js_out=import_style=commonjs,binary:${PROTO_DEST} \
    --grpc_out=grpc_js:${PROTO_DEST} \
    -I ../../fermi-rs/controller/src/futures/proto \
    ../../fermi-rs/controller/src/futures/proto/*.proto

# TypeScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=${PROTO_DEST} \
     -I ../../fermi-rs/controller/src/futures/proto \
    ../../fermi-rs/controller/src/futures/proto/*.proto
