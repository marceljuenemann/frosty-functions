#!/usr/bin/env bash

# Owner = first Anvil account
# TODO: Replace with address of the canister.
OWNER=0x5FbDB2315678afecb367f032d93F642f64180aa3

# Minimum payment: 100 gwei
MIN_PAYMENT_WEI=100000000000

# Default Anvil private key for account[0] (public for local testing)
PK=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

forge create contracts/Bridge.sol:FrostyBridge \
  --broadcast \
  --private-key $PK \
  --constructor-args $OWNER $MIN_PAYMENT_WEI

