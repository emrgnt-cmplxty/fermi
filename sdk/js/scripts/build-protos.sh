#!/bin/bash

BASEDIR=$PWD
cd ${BASEDIR}/../

PROTO_DEST=./dist/proto

mkdir -p ${PROTO_DEST}
# Generate proto for types
# JavaScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=grpc_js:${PROTO_DEST} \
    --js_out=import_style=commonjs,binary:${PROTO_DEST} \
    --grpc_out=grpc_js:${PROTO_DEST} \
    -I ../../rust-gdex/types/proto \
    ../../rust-gdex/types/proto/*.proto

# TypeScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=${PROTO_DEST} \
     -I ../../rust-gdex/types/proto \
    ../../rust-gdex/types/proto/*.proto


# Repeat for bank controller
# JavaScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=grpc_js:${PROTO_DEST} \
    --js_out=import_style=commonjs,binary:${PROTO_DEST} \
    --grpc_out=grpc_js:${PROTO_DEST} \
    -I ../../rust-gdex/controller/src/bank/proto \
    ../../rust-gdex/controller/src/bank/proto/*.proto

# TypeScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=${PROTO_DEST} \
     -I ../../rust-gdex/controller/src/bank/proto \
    ../../rust-gdex/controller/src/bank/proto/*.proto

# Repeat for spot controller
# JavaScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=grpc_js:${PROTO_DEST} \
    --js_out=import_style=commonjs,binary:${PROTO_DEST} \
    --grpc_out=grpc_js:${PROTO_DEST} \
    -I ../../rust-gdex/controller/src/spot/proto \
    ../../rust-gdex/controller/src/spot/proto/*.proto

# TypeScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=${PROTO_DEST} \
     -I ../../rust-gdex/controller/src/spot/proto \
    ../../rust-gdex/controller/src/spot/proto/*.proto

# Repeat for futures controller
# JavaScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=grpc_js:${PROTO_DEST} \
    --js_out=import_style=commonjs,binary:${PROTO_DEST} \
    --grpc_out=grpc_js:${PROTO_DEST} \
    -I ../../rust-gdex/controller/src/futures/proto \
    ../../rust-gdex/controller/src/futures/proto/*.proto

# TypeScript code generation
yarn run grpc_tools_node_protoc \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    --ts_out=${PROTO_DEST} \
     -I ../../rust-gdex/controller/src/futures/proto \
    ../../rust-gdex/controller/src/futures/proto/*.proto
