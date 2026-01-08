.PHONY: deploy-full-devnet build clean

# Paths
PROGRAM_DIR := smart-contract/program
PROGRAM_SO := $(PROGRAM_DIR)/target/deploy/access_protocol.so

# Build the program
build:
	cd $(PROGRAM_DIR) && cargo build-bpf

# Full devnet deployment (build + deploy + init)
# Usage: make deploy-full-devnet RPC=https://api.devnet.solana.com
deploy-full-devnet:
ifndef RPC
	$(error RPC is required. Usage: make deploy-full-devnet RPC=https://api.devnet.solana.com)
endif
	cd scripts && SOLANA_RPC_PROVIDER_URL=$(RPC) ./deploy-full-devnet.sh

# Deploy with v1 instructions enabled
deploy-full-devnet-v1:
ifndef RPC
	$(error RPC is required. Usage: make deploy-full-devnet-v1 RPC=https://api.devnet.solana.com)
endif
	cd scripts && ALLOW_V1=true SOLANA_RPC_PROVIDER_URL=$(RPC) ./deploy-full-devnet.sh

# Clean build artifacts
clean:
	cd $(PROGRAM_DIR) && cargo clean

# Check if program .so exists
check-program:
	@test -f $(PROGRAM_SO) && echo "✓ $(PROGRAM_SO) exists" || echo "✗ $(PROGRAM_SO) not found - run 'make build'"
