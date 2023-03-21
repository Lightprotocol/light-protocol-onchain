#!/bin/bash

set -eux

docker rm -f solana-validator || true
docker run -d \
    --name solana-validator \
    --net=host \
    -v $HOME/.config/solana/id.json:/root/.config/solana/id.json \
    -v $(git rev-parse --show-toplevel)/light-system-programs/target/deploy:/usr/local/lib/light-protocol-onchain \
    vadorovsky/solana:audit \
    --reset \
    --limit-ledger-size 500000000 \
    --faucet-port 9002 \
    --rpc-port 8899 \
    --bpf-program J1RRetZ4ujphU75LP8RadjXMf3sA12yC2R44CF7PmU7i /usr/local/lib/light-protocol-onchain/verifier_program_zero.so \
    --bpf-program JA5cjkRJ1euVi9xLWsCJVzsRzEkT8vcC4rqw9sVAo5d6 /usr/local/lib/light-protocol-onchain/merkle_tree_program.so \
    --bpf-program 3KS2k14CmtnuVv2fvYcvdrNgC94Y11WETBpMUGgXyWZL /usr/local/lib/light-protocol-onchain/verifier_program_one.so \
    --bpf-program noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV /usr/local/lib/solana-program-library/spl_noop.so \
    --bpf-program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS /usr/local/lib/light-protocol-onchain/verifier_program.so \
    --quiet

while ! solana balance | grep "500000000 SOL"; do
    sleep 1
done

solana airdrop 100000 ZBUKxVWviAJBy12edp5H6kvhcatGYW3BV4ijbgxpVSq
solana airdrop 100000 ALA2cnz41Wa2v2EYUdkYHsg7VnKsbH1j7secM5aiP8k
solana airdrop 100000 8Ers2bBEWExdrh7KDFTrRbauPbFeEvsHz3UX4vxcK9xY
solana airdrop 10000 BEKmoiPHRUxUPik2WQuKqkoFLLkieyNPrTDup5h8c9S7
