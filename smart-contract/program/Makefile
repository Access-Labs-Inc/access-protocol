test:
	cargo test-bpf --features no-mint-check no-bond-signer v1-instructions-allowed -- --nocapture --skip functional --skip devnet
	cargo test-bpf  --features no-mint-check no-bond-signer v1-instructions-allowed days-to-sec-10s --test functional -- --nocapture

test_devnet:
	cargo test --test devnet -- --nocapture