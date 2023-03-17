#!/bin/bash -e
../../solana/validator/solana-test-validator     --reset     --limit-ledger-size 500000000     --bpf-program J1RRetZ4ujphU75LP8RadjXMf3sA12yC2R44CF7PmU7i ./target/deploy/verifier_program_zero.so     --bpf-program JA5cjkRJ1euVi9xLWsCJVzsRzEkT8vcC4rqw9sVAo5d6 ./target/deploy/merkle_tree_program.so     --bpf-program 3KS2k14CmtnuVv2fvYcvdrNgC94Y11WETBpMUGgXyWZL ./target/deploy/verifier_program_one.so --bpf-program noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV ../../solana/web3.js/test/fixtures/noop-program/solana_sbf_rust_noop.so --bpf-program DJpbogMSrK94E1zvvJydtkqoE4sknuzmMRoutd6B7TKj ./target/deploy/verifier_program_storage.so --quiet &
sleep 10

#!/bin/bash -e
solana airdrop 100000 ZBUKxVWviAJBy12edp5H6kvhcatGYW3BV4ijbgxpVSq && solana airdrop 100000 ALA2cnz41Wa2v2EYUdkYHsg7VnKsbH1j7secM5aiP8k && solana airdrop 100000 8Ers2bBEWExdrh7KDFTrRbauPbFeEvsHz3UX4vxcK9xY && solana airdrop 10000 BEKmoiPHRUxUPik2WQuKqkoFLLkieyNPrTDup5h8c9S7
cd ../relayer && node lib/index.js
PID=$!
$1;
kill $PID;