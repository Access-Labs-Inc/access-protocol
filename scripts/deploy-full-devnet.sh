#!/bin/bash
set -o allexport -o notify
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m' # No Color

pwd=$(pwd)
ARTIFACTS_DIR="$pwd/artifacts"

# RPC endpoint (required)
if [ -z "$SOLANA_RPC_PROVIDER_URL" ]; then
  echo -e "${RED}✗ Error: SOLANA_RPC_PROVIDER_URL is required${NC}"
  echo -e "  Usage: SOLANA_RPC_PROVIDER_URL=https://api.devnet.solana.com $0"
  exit 1
fi
export SOLANA_RPC_PROVIDER_URL

# Logging helpers
log_header() {
  echo ""
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${BOLD}${CYAN}  $1${NC}"
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

log_step() {
  echo -e "${YELLOW}▶${NC} $1"
}

log_success() {
  echo -e "${GREEN}✓${NC} $1"
}

log_info() {
  echo -e "${DIM}  $1${NC}"
}

log_error() {
  echo -e "${RED}✗ $1${NC}"
}

# Track start time
START_TIME=$(date +%s)

echo ""
echo -e "${BOLD}${GREEN}╔════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${GREEN}║           ACCESS PROTOCOL - FULL DEVNET DEPLOYMENT                         ║${NC}"
echo -e "${BOLD}${GREEN}║  RPC: ${SOLANA_RPC_PROVIDER_URL}${NC}"
echo -e "${BOLD}${GREEN}╚════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$ARTIFACTS_DIR"

# ─────────────────────────────────────────────────────────────────────────────
log_header "STEP 1/6: Creating SPL Token"
log_step "Running create-devnet-spl-token.ts..."
ts-node create-devnet-spl-token.ts
log_success "SPL Token created"

# ─────────────────────────────────────────────────────────────────────────────
log_header "STEP 2/6: Deploying Smart Contract"
log_step "Building and deploying Access Protocol contract..."
log_info "V1 instructions: ${ALLOW_V1:-disabled}"
./deploy-contract.sh
log_success "Contract deployed"

# ─────────────────────────────────────────────────────────────────────────────
log_header "STEP 3/6: Initializing Central State"
log_step "Creating central state account..."
./init-central-state.sh
log_success "Central state initialized"

# ─────────────────────────────────────────────────────────────────────────────
log_header "STEP 4/6: Minting Initial Token Supply"
log_step "Minting 1,000,000 ACS tokens to token bank..."
log_info "Required for V2 (minting disabled after migration)"
./spl-mint.sh
log_success "Tokens minted"

# ─────────────────────────────────────────────────────────────────────────────
log_header "STEP 5/6: Transferring SPL Token Authority"
log_step "Transferring mint authority to central state..."
./spl-set-authority.sh
log_success "Authority transferred"

# ─────────────────────────────────────────────────────────────────────────────
log_header "STEP 6/6: Migrating to V2"
log_step "Migrating central state to V2 format..."
./migrate-central-state-v2.sh
log_success "Migration complete"

# ─────────────────────────────────────────────────────────────────────────────
# Calculate duration
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
MINUTES=$((DURATION / 60))
SECONDS=$((DURATION % 60))

# Gather deployment info
PROGRAM_PUBKEY=$(solana-keygen pubkey "$ARTIFACTS_DIR/program.json" 2>/dev/null || echo "N/A")
AUTHORITY_PUBKEY=$(solana-keygen pubkey "$ARTIFACTS_DIR/authority.json" 2>/dev/null || echo "N/A")
SPL_AUTHORITY_PUBKEY=$(solana-keygen pubkey "$ARTIFACTS_DIR/spl_authority.json" 2>/dev/null || echo "N/A")
TOKEN_BANK_PUBKEY=$(solana-keygen pubkey "$ARTIFACTS_DIR/token_bank.json" 2>/dev/null || echo "N/A")
MINT_ADDRESS=$(cat "$ARTIFACTS_DIR/mint_address.txt" 2>/dev/null || echo "N/A")
CENTRAL_STATE_PUBKEY=$(cat "$ARTIFACTS_DIR/central_state_pubkey.txt" 2>/dev/null || echo "N/A")

echo ""
echo -e "${BOLD}${GREEN}╔════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${GREEN}║                        DEPLOYMENT COMPLETE                                 ║${NC}"
echo -e "${BOLD}${GREEN}╚════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BOLD}RPC:${NC}            ${SOLANA_RPC_PROVIDER_URL}"
echo -e "${BOLD}Duration:${NC}       ${MINUTES}m ${SECONDS}s"
echo ""
echo -e "${BOLD}${CYAN}─── Program ───────────────────────────────────────────────────────────────────${NC}"
echo -e "${BOLD}Program ID:${NC}     ${PROGRAM_PUBKEY}"
echo -e "${BOLD}Central State:${NC}  ${CENTRAL_STATE_PUBKEY}"
echo ""
echo -e "${BOLD}${CYAN}─── Token ─────────────────────────────────────────────────────────────────────${NC}"
echo -e "${BOLD}Mint Address:${NC}   ${MINT_ADDRESS}"
echo -e "${BOLD}Token Bank:${NC}     ${TOKEN_BANK_PUBKEY}"
echo ""
echo -e "${BOLD}${CYAN}─── Wallets ───────────────────────────────────────────────────────────────────${NC}"
echo -e "${BOLD}Authority:${NC}      ${AUTHORITY_PUBKEY}"
echo -e "${BOLD}SPL Authority:${NC}  ${SPL_AUTHORITY_PUBKEY}"
echo ""
echo -e "${BOLD}${CYAN}─── Artifacts Location ────────────────────────────────────────────────────────${NC}"
echo -e "${DIM}${ARTIFACTS_DIR}/${NC}"
echo ""
ls -la "$ARTIFACTS_DIR" | tail -n +2 | while read line; do
  echo -e "  ${DIM}${line}${NC}"
done
echo ""
echo -e "${BOLD}${CYAN}─── Explorer Links ────────────────────────────────────────────────────────────${NC}"
echo -e "Program:  ${DIM}https://explorer.solana.com/address/${PROGRAM_PUBKEY}?cluster=devnet${NC}"
echo -e "Token:    ${DIM}https://explorer.solana.com/address/${MINT_ADDRESS}?cluster=devnet${NC}"
echo ""
