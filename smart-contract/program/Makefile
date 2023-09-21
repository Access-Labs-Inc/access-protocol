test_solana_program:
	cargo test-bpf --features no-mint-check no-bond-signer v1-instructions-allowed -- --nocapture --skip functional
	cargo test-bpf  --features no-mint-check no-bond-signer v1-instructions-allowed days-to-sec-10s --test functional -- --nocapture