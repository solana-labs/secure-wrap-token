#!/bin/bash
# 'anchor test' has a limitation where because it uses --bpf-program to start the test validator, no upgrade authority is possible.
# To work around the Anchor limitation, we start up and load the local test validator separately.

# Register cleanup routines because we will spawn a local validator in the background.
trap "exit" INT TERM
trap "kill 0" EXIT

# Kill any currently running local validator.
pkill solana-test-validator

# Build the program, and update the artifacts metadata to mimic that it was deployed by Anchor. This allows it to be visible in typescript.
anchor build
json -I -f target/idl/secure_wrap_token.json -e "this.metadata = {}; this.metadata.address='NNcz9dDJ5cSxeNy95kn4AosZaQ3jzBzMzrhxyRzDyXQ'"

# Start local validator in the background.
solana-test-validator --reset --quiet --ledger=/tmp/.solana_test_ledger \
	--upgradeable-program NNcz9dDJ5cSxeNy95kn4AosZaQ3jzBzMzrhxyRzDyXQ target/deploy/secure_wrap_token.so D4BhVR2dUGYueMJrm2eBUo3Z9qnuFakrxKRFD5toH9hr \
	&
# Sleep is mandatory because the test validator takes some time to start.
sleep 0.5

# Run anchor test
anchor test --skip-local-validator --skip-build --skip-deploy

