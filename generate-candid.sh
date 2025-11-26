#!/usr/bin/env bash

# Workaround for a bug in cargo-did?!
generate-did frosty-functions-backend
cp target/wasm32-unknown-unknown/release/frosty_functions_backend.wasm target/wasm32-unknown-unknown/release/frosty-functions-backend.wasm 
generate-did frosty-functions-backend
dfx generate
